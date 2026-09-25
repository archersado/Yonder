use std::collections::BTreeMap;

pub const MAX_COMMAND_PATH_BYTES: usize = 4096;
pub const MAX_COMMAND_ARGS: usize = 128;
pub const MAX_COMMAND_ARG_BYTES: usize = 4096;
pub const MAX_COMMAND_ARGS_BYTES: usize = 32 * 1024;
pub const MAX_COMMAND_ENV: usize = 64;
pub const MAX_COMMAND_ENV_KEY_BYTES: usize = 128;
pub const MAX_COMMAND_ENV_VALUE_BYTES: usize = 4096;
pub const MAX_COMMAND_ENV_BYTES: usize = 16 * 1024;
pub const MAX_COMMAND_OUTPUT_BYTES: usize = 64 * 1024;
pub const MIN_COMMAND_TIMEOUT_MS: u64 = 100;
pub const MAX_COMMAND_TIMEOUT_MS: u64 = 300_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandRequest {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub env: BTreeMap<String, String>,
    pub timeout_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandOutcome {
    Exited { exit_code: i32 },
    TimedOut,
    Cancelled,
    OutputLimitExceeded,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandExecution {
    pub outcome: CommandOutcome,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandError {
    InvalidInput,
    UnsupportedPlatform,
    StartFailed,
}

pub trait CommandCancellation: Send + Sync {
    fn cancelled(&self) -> bool;
}

pub struct NeverCancel;

impl CommandCancellation for NeverCancel {
    fn cancelled(&self) -> bool {
        false
    }
}

pub trait CommandPort {
    fn execute(
        &self,
        request: &CommandRequest,
        cancellation: &dyn CommandCancellation,
    ) -> Result<CommandExecution, CommandError>;
}

fn valid_env_key(value: &str) -> bool {
    !value.is_empty()
        && value.as_bytes().len() <= MAX_COMMAND_ENV_KEY_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

pub fn validate_request(request: &CommandRequest) -> Result<(), CommandError> {
    let valid_path = |value: &str| {
        !value.is_empty()
            && value.as_bytes().len() <= MAX_COMMAND_PATH_BYTES
            && !value.contains('\0')
            && std::path::Path::new(value).is_absolute()
    };
    let args_bytes = request.args.iter().try_fold(0usize, |total, value| {
        total.checked_add(value.as_bytes().len())
    });
    let env_bytes = request.env.iter().try_fold(0usize, |total, (key, value)| {
        total
            .checked_add(key.as_bytes().len())?
            .checked_add(value.as_bytes().len())
    });
    if !valid_path(&request.program)
        || !valid_path(&request.cwd)
        || request.args.len() > MAX_COMMAND_ARGS
        || request
            .args
            .iter()
            .any(|value| value.as_bytes().len() > MAX_COMMAND_ARG_BYTES || value.contains('\0'))
        || args_bytes.is_none_or(|bytes| bytes > MAX_COMMAND_ARGS_BYTES)
        || request.env.len() > MAX_COMMAND_ENV
        || request.env.iter().any(|(key, value)| {
            !valid_env_key(key)
                || value.as_bytes().len() > MAX_COMMAND_ENV_VALUE_BYTES
                || value.contains('\0')
        })
        || env_bytes.is_none_or(|bytes| bytes > MAX_COMMAND_ENV_BYTES)
        || !(MIN_COMMAND_TIMEOUT_MS..=MAX_COMMAND_TIMEOUT_MS).contains(&request.timeout_ms)
    {
        return Err(CommandError::InvalidInput);
    }
    Ok(())
}

/// 只执行已经过可信上层授权的结构化请求；本用例不创建任务或风险确认事实。
pub fn execute(
    port: &dyn CommandPort,
    request: &CommandRequest,
    cancellation: &dyn CommandCancellation,
) -> Result<CommandExecution, CommandError> {
    validate_request(request)?;
    if cancellation.cancelled() {
        return Ok(CommandExecution {
            outcome: CommandOutcome::Cancelled,
            stdout: Vec::new(),
            stderr: Vec::new(),
            stdout_truncated: false,
            stderr_truncated: false,
        });
    }
    port.execute(request, cancellation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Port(AtomicUsize);
    impl CommandPort for Port {
        fn execute(
            &self,
            _: &CommandRequest,
            _: &dyn CommandCancellation,
        ) -> Result<CommandExecution, CommandError> {
            self.0.fetch_add(1, Ordering::Relaxed);
            Ok(CommandExecution {
                outcome: CommandOutcome::Exited { exit_code: 0 },
                stdout: Vec::new(),
                stderr: Vec::new(),
                stdout_truncated: false,
                stderr_truncated: false,
            })
        }
    }

    struct Cancelled;
    impl CommandCancellation for Cancelled {
        fn cancelled(&self) -> bool {
            true
        }
    }

    fn request() -> CommandRequest {
        CommandRequest {
            program: "/usr/bin/printf".into(),
            args: vec!["literal".into()],
            cwd: "/tmp".into(),
            env: BTreeMap::new(),
            timeout_ms: 1000,
        }
    }

    #[test]
    fn validation_happens_before_the_port() {
        let port = Port(AtomicUsize::new(0));
        let mut value = request();
        value.program = "relative".into();
        assert_eq!(
            execute(&port, &value, &NeverCancel),
            Err(CommandError::InvalidInput)
        );
        value = request();
        value.args = vec!["x".repeat(MAX_COMMAND_ARG_BYTES + 1)];
        assert_eq!(
            execute(&port, &value, &NeverCancel),
            Err(CommandError::InvalidInput)
        );
        value = request();
        value.env.insert("BAD-KEY".into(), "value".into());
        assert_eq!(
            execute(&port, &value, &NeverCancel),
            Err(CommandError::InvalidInput)
        );
        value = request();
        value.timeout_ms = MIN_COMMAND_TIMEOUT_MS - 1;
        assert_eq!(
            execute(&port, &value, &NeverCancel),
            Err(CommandError::InvalidInput)
        );
        assert_eq!(port.0.load(Ordering::Relaxed), 0);
        assert!(execute(&port, &request(), &NeverCancel).is_ok());
        assert_eq!(port.0.load(Ordering::Relaxed), 1);
        assert!(matches!(
            execute(&port, &request(), &Cancelled),
            Ok(CommandExecution {
                outcome: CommandOutcome::Cancelled,
                ..
            })
        ));
        assert_eq!(port.0.load(Ordering::Relaxed), 1);
    }
}
