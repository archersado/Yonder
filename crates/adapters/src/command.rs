use yonder_application::command::{
    CommandCancellation, CommandError, CommandExecution, CommandOutcome, CommandPort,
    CommandRequest,
};

pub struct StructuredCommandAdapter;

#[cfg(not(target_os = "macos"))]
impl CommandPort for StructuredCommandAdapter {
    fn execute(
        &self,
        _: &CommandRequest,
        _: &dyn CommandCancellation,
    ) -> Result<CommandExecution, CommandError> {
        Err(CommandError::UnsupportedPlatform)
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::{
        fs,
        io::Read,
        os::unix::process::CommandExt,
        process::{Child, Command, Stdio},
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        thread,
        time::{Duration, Instant},
    };
    use yonder_application::command::MAX_COMMAND_OUTPUT_BYTES;

    const POLL_INTERVAL: Duration = Duration::from_millis(10);
    const TERM_GRACE: Duration = Duration::from_millis(250);
    const KILL_GRACE: Duration = Duration::from_secs(1);
    const STREAM_GRACE: Duration = Duration::from_secs(1);
    const SIGTERM: i32 = 15;
    const SIGKILL: i32 = 9;

    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }

    #[derive(Default)]
    struct StreamCapture {
        bytes: Vec<u8>,
        truncated: bool,
        failed: bool,
    }

    #[derive(Clone, Copy)]
    enum StopReason {
        Timeout,
        Cancelled,
        OutputLimit,
        UnexpectedDescendant,
    }

    fn signal_group(pid: u32, signal: i32) -> bool {
        let Ok(group) = i32::try_from(pid) else {
            return false;
        };
        // SAFETY: kill is called with a negative, freshly spawned process-group id and a
        // constant signal. No pointers cross the FFI boundary.
        unsafe { kill(-group, signal) == 0 }
    }

    fn group_alive(pid: u32) -> bool {
        let Ok(group) = i32::try_from(pid) else {
            return true;
        };
        // SAFETY: signal 0 only probes a process group and does not mutate memory.
        unsafe { kill(-group, 0) == 0 }
    }

    #[cfg(test)]
    pub(super) fn pid_alive(pid: i32) -> bool {
        // SAFETY: signal 0 only probes a numeric pid and does not mutate memory.
        unsafe { kill(pid, 0) == 0 }
    }

    fn bounded_reader(
        mut reader: impl Read + Send + 'static,
        overflow: Arc<AtomicBool>,
    ) -> mpsc::Receiver<StreamCapture> {
        let (sender, receiver) = mpsc::sync_channel(1);
        thread::spawn(move || {
            let mut capture = StreamCapture {
                bytes: Vec::with_capacity(MAX_COMMAND_OUTPUT_BYTES),
                ..StreamCapture::default()
            };
            let mut chunk = [0u8; 8192];
            loop {
                match reader.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(count) => {
                        let remaining = MAX_COMMAND_OUTPUT_BYTES - capture.bytes.len();
                        capture
                            .bytes
                            .extend_from_slice(&chunk[..count.min(remaining)]);
                        if count > remaining {
                            capture.truncated = true;
                            overflow.store(true, Ordering::Release);
                            break;
                        }
                    }
                    Err(_) => {
                        capture.failed = true;
                        break;
                    }
                }
            }
            let _ = sender.send(capture);
        });
        receiver
    }

    fn wait_for_exit(child: &mut Child, deadline: Instant) -> bool {
        loop {
            match child.try_wait() {
                Ok(Some(_)) => return true,
                Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
                _ => return false,
            }
        }
    }

    fn stop_group(child: &mut Child) -> bool {
        let pid = child.id();
        let _ = signal_group(pid, SIGTERM);
        let term_deadline = Instant::now() + TERM_GRACE;
        while Instant::now() < term_deadline {
            let child_done = matches!(child.try_wait(), Ok(Some(_)));
            if child_done && !group_alive(pid) {
                return true;
            }
            thread::sleep(POLL_INTERVAL);
        }
        if group_alive(pid) {
            let _ = signal_group(pid, SIGKILL);
        }
        let child_done = wait_for_exit(child, Instant::now() + KILL_GRACE);
        child_done && !group_alive(pid)
    }

    fn canonical_paths(
        request: &CommandRequest,
    ) -> Result<(std::path::PathBuf, std::path::PathBuf), CommandError> {
        let program = fs::canonicalize(&request.program).map_err(|_| CommandError::InvalidInput)?;
        let cwd = fs::canonicalize(&request.cwd).map_err(|_| CommandError::InvalidInput)?;
        if !program.is_absolute()
            || !cwd.is_absolute()
            || !program.metadata().is_ok_and(|value| value.is_file())
            || !cwd.metadata().is_ok_and(|value| value.is_dir())
        {
            return Err(CommandError::InvalidInput);
        }
        Ok((program, cwd))
    }

    impl CommandPort for StructuredCommandAdapter {
        fn execute(
            &self,
            request: &CommandRequest,
            cancellation: &dyn CommandCancellation,
        ) -> Result<CommandExecution, CommandError> {
            yonder_application::command::validate_request(request)?;
            if cancellation.cancelled() {
                return Ok(CommandExecution {
                    outcome: CommandOutcome::Cancelled,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                    stdout_truncated: false,
                    stderr_truncated: false,
                });
            }
            let (program, cwd) = canonical_paths(request)?;
            let mut command = Command::new(program);
            command
                .args(&request.args)
                .current_dir(cwd)
                .env_clear()
                .envs(&request.env)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .process_group(0);
            let mut child = command.spawn().map_err(|_| CommandError::StartFailed)?;
            let overflow = Arc::new(AtomicBool::new(false));
            let stdout = bounded_reader(
                child.stdout.take().ok_or(CommandError::StartFailed)?,
                Arc::clone(&overflow),
            );
            let stderr = bounded_reader(
                child.stderr.take().ok_or(CommandError::StartFailed)?,
                Arc::clone(&overflow),
            );
            let deadline = Instant::now() + Duration::from_millis(request.timeout_ms);
            let mut known_exit = None;
            let stop_reason = loop {
                if overflow.load(Ordering::Acquire) {
                    break Some(StopReason::OutputLimit);
                }
                if cancellation.cancelled() {
                    break Some(StopReason::Cancelled);
                }
                if Instant::now() >= deadline {
                    break Some(StopReason::Timeout);
                }
                match child.try_wait() {
                    Ok(Some(status)) => {
                        if group_alive(child.id()) {
                            break Some(StopReason::UnexpectedDescendant);
                        }
                        known_exit = status.code();
                        break None;
                    }
                    Ok(None) => thread::sleep(POLL_INTERVAL),
                    Err(_) => break Some(StopReason::UnexpectedDescendant),
                }
            };

            let stopped = stop_reason.is_none_or(|_| stop_group(&mut child));
            let stdout = stdout.recv_timeout(STREAM_GRACE).unwrap_or(StreamCapture {
                failed: true,
                ..StreamCapture::default()
            });
            let stderr = stderr.recv_timeout(STREAM_GRACE).unwrap_or(StreamCapture {
                failed: true,
                ..StreamCapture::default()
            });
            let outcome = if !stopped || stdout.failed || stderr.failed {
                CommandOutcome::Unknown
            } else {
                match stop_reason {
                    Some(StopReason::Timeout) => CommandOutcome::TimedOut,
                    Some(StopReason::Cancelled) => CommandOutcome::Cancelled,
                    Some(StopReason::OutputLimit) => CommandOutcome::OutputLimitExceeded,
                    Some(StopReason::UnexpectedDescendant) => CommandOutcome::Unknown,
                    None => known_exit
                        .map(|exit_code| CommandOutcome::Exited { exit_code })
                        .unwrap_or(CommandOutcome::Unknown),
                }
            };
            Ok(CommandExecution {
                outcome,
                stdout: stdout.bytes,
                stderr: stderr.bytes,
                stdout_truncated: stdout.truncated,
                stderr_truncated: stderr.truncated,
            })
        }
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use std::{
        collections::BTreeMap,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        thread,
        time::Duration,
    };
    use yonder_application::command::{MAX_COMMAND_OUTPUT_BYTES, NeverCancel, execute};

    struct Cancellation(Arc<AtomicBool>);
    impl CommandCancellation for Cancellation {
        fn cancelled(&self) -> bool {
            self.0.load(Ordering::Acquire)
        }
    }

    fn request(program: &str, args: &[&str], timeout_ms: u64) -> CommandRequest {
        CommandRequest {
            program: program.into(),
            args: args.iter().map(|value| (*value).into()).collect(),
            cwd: "/tmp".into(),
            env: BTreeMap::new(),
            timeout_ms,
        }
    }

    fn parse_pid(bytes: &[u8]) -> i32 {
        String::from_utf8_lossy(bytes).trim().parse().unwrap()
    }

    fn process_alive(pid: i32) -> bool {
        macos::pid_alive(pid)
    }

    #[test]
    fn literal_arguments_and_explicit_environment_are_not_shell_expanded() {
        let marker = format!("/tmp/yonder-command-never-created-{}", std::process::id());
        let literal = format!("$(touch {marker})");
        let result = execute(
            &StructuredCommandAdapter,
            &request("/usr/bin/printf", &["%s", &literal], 1000),
            &NeverCancel,
        )
        .unwrap();
        assert_eq!(result.outcome, CommandOutcome::Exited { exit_code: 0 });
        assert_eq!(result.stdout, literal.as_bytes());
        assert!(!std::path::Path::new(&marker).exists());

        let mut env_request = request("/usr/bin/env", &[], 1000);
        env_request
            .env
            .insert("YONDER_ONLY".into(), "visible".into());
        let result = execute(&StructuredCommandAdapter, &env_request, &NeverCancel).unwrap();
        assert_eq!(result.stdout, b"YONDER_ONLY=visible\n");
    }

    #[test]
    fn exit_start_and_input_failures_are_distinct() {
        let nonzero = execute(
            &StructuredCommandAdapter,
            &request("/usr/bin/false", &[], 1000),
            &NeverCancel,
        )
        .unwrap();
        assert_eq!(nonzero.outcome, CommandOutcome::Exited { exit_code: 1 });
        let missing = request("/tmp/yonder-command-does-not-exist", &[], 1000);
        assert_eq!(
            execute(&StructuredCommandAdapter, &missing, &NeverCancel),
            Err(CommandError::InvalidInput)
        );
        let directory = request("/tmp", &[], 1000);
        assert_eq!(
            execute(&StructuredCommandAdapter, &directory, &NeverCancel),
            Err(CommandError::InvalidInput)
        );

        use std::os::unix::fs::PermissionsExt;
        let blocked = format!("/tmp/yonder-command-not-executable-{}", std::process::id());
        std::fs::write(&blocked, b"not executable").unwrap();
        std::fs::set_permissions(&blocked, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert_eq!(
            execute(
                &StructuredCommandAdapter,
                &request(&blocked, &[], 1000),
                &NeverCancel
            ),
            Err(CommandError::StartFailed)
        );
        std::fs::remove_file(blocked).unwrap();
    }

    #[test]
    fn timeout_and_cancellation_stop_parent_and_descendant() {
        let timeout = execute(
            &StructuredCommandAdapter,
            &request("/bin/sh", &["-c", "/bin/sleep 30 & echo $!; wait"], 150),
            &NeverCancel,
        )
        .unwrap();
        assert_eq!(timeout.outcome, CommandOutcome::TimedOut);
        assert!(!process_alive(parse_pid(&timeout.stdout)));

        let flag = Arc::new(AtomicBool::new(false));
        let setter = Arc::clone(&flag);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(100));
            setter.store(true, Ordering::Release);
        });
        let cancelled = execute(
            &StructuredCommandAdapter,
            &request("/bin/sh", &["-c", "/bin/sleep 30 & echo $!; wait"], 5000),
            &Cancellation(flag),
        )
        .unwrap();
        assert_eq!(cancelled.outcome, CommandOutcome::Cancelled);
        assert!(!process_alive(parse_pid(&cancelled.stdout)));
    }

    #[test]
    fn either_output_stream_overflow_stops_the_group_with_a_bounded_prefix() {
        for (script, stderr) in [
            ("while :; do echo stdout; done", false),
            ("while :; do echo stderr >&2; done", true),
        ] {
            let result = execute(
                &StructuredCommandAdapter,
                &request("/bin/sh", &["-c", script], 5000),
                &NeverCancel,
            )
            .unwrap();
            assert_eq!(result.outcome, CommandOutcome::OutputLimitExceeded);
            let (bytes, truncated) = if stderr {
                (&result.stderr, result.stderr_truncated)
            } else {
                (&result.stdout, result.stdout_truncated)
            };
            assert_eq!(bytes.len(), MAX_COMMAND_OUTPUT_BYTES);
            assert!(truncated);
        }
    }

    #[test]
    fn a_background_descendant_prevents_a_false_success() {
        let result = execute(
            &StructuredCommandAdapter,
            &request("/bin/sh", &["-c", "/bin/sleep 30 & echo $!"], 1000),
            &NeverCancel,
        )
        .unwrap();
        assert_eq!(result.outcome, CommandOutcome::Unknown);
        assert!(!process_alive(parse_pid(&result.stdout)));
    }
}
