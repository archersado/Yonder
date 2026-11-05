use std::{
    env, fs,
    io::{BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::mpsc,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use yonder_agent_desktop_driver_spike::{
    BRIDGE_SCHEMA_VERSION, SOURCE_REVISION,
    clock::monotonic_millis,
    containment::{CleanupVerdict as ContainmentVerdict, Containment},
    framing::{read_frame, write_frame},
    protocol::{
        ActionId, HostRequest, Identity, TargetIdentity, WorkerResponse, shutdown_response_matches,
    },
    supervisor::{
        AdmissionSnapshot, ControlState, DispatchResult, DispatchTransport, HostSafetyState,
        OracleSample, ReadOnlyOracle, Supervisor, TransportOutcome, deadline_write,
    },
};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn deadline(offset: u64) -> Result<u64> {
    monotonic_millis()?
        .checked_add(offset)
        .context("deadline overflow")
}
fn identity(request: &str) -> Identity {
    Identity {
        request_id: request.into(),
        attempt_id: "attempt-1".into(),
        worker_instance_id: "worker-1".into(),
        fence: 1,
    }
}
fn state() -> AdmissionSnapshot {
    AdmissionSnapshot {
        fence: 1,
        input_generation: 2,
        observer_ready: true,
        control: ControlState::Running,
    }
}
fn hashes() -> [String; 3] {
    [
        hash(include_bytes!(
            "../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/bridge-contract-v1.md"
        )),
        hash(include_bytes!(
            "../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/sample-definitions-v1.json"
        )),
        hash(include_bytes!(
            "../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/action-allowlist-v1.json"
        )),
    ]
}

struct WorkerSession {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    containment: Box<dyn Containment>,
    state_dir: PathBuf,
    host_safety: HostSafetyState,
    closed: bool,
}

impl WorkerSession {
    fn start(
        worker: &Path,
        malformed: bool,
        block_dispatch: bool,
        host_safety: HostSafetyState,
    ) -> Result<Self> {
        let state_dir = env::temp_dir().join(format!(
            "yonder-agent-desktop-{}-{}",
            std::process::id(),
            monotonic_millis()?
        ));
        fs::create_dir_all(&state_dir)?;
        let mut command = Command::new(worker);
        command.arg("--mock");
        if malformed {
            command.arg("--mock-malformed-response");
        }
        if block_dispatch {
            command.arg("--mock-block-dispatch");
        }
        command
            .env("AGENT_DESKTOP_HOME", &state_dir)
            .env("AGENT_DESKTOP_TRACE", "off")
            .env("AGENT_DESKTOP_SCREENSHOTS", "off")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let mut containment: Box<dyn Containment> =
            Box::new(yonder_agent_desktop_driver_spike::containment::MockContainment);
        #[cfg(target_os = "macos")]
        let mut containment: Box<dyn Containment> =
            Box::new(yonder_agent_desktop_driver_spike::containment::MacProcessGroup::new());
        #[cfg(target_os = "windows")]
        let mut containment: Box<dyn Containment> =
            Box::new(yonder_agent_desktop_driver_spike::containment::WindowsJob::new()?);
        let mut child = containment.spawn(&mut command)?;
        let input = child.stdin.take().context("worker stdin")?;
        let output = BufReader::new(child.stdout.take().context("worker stdout")?);
        Ok(Self {
            child,
            input,
            output,
            containment,
            state_dir,
            host_safety,
            closed: false,
        })
    }

    fn hello(&mut self, supervisor: &mut Supervisor) -> Result<()> {
        let values = hashes();
        let request = HostRequest::Hello {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("hello"),
            input_generation: 2,
            deadline_mono_ms: deadline(10_000)?,
            source_revision: SOURCE_REVISION.into(),
            bridge_contract_sha256: values[0].clone(),
            sample_definitions_sha256: values[1].clone(),
            action_allowlist_sha256: values[2].clone(),
        };
        write_frame(&mut self.input, &request)?;
        let response = self.read_deadline(supervisor, request.deadline_mono_ms())?;
        if !supervisor.accept_hello(
            &response,
            &identity("hello"),
            &[&values[0], &values[1], &values[2]],
        ) {
            bail!("hello 校验失败");
        }
        Ok(())
    }

    fn snapshot(&mut self, supervisor: &mut Supervisor) -> Result<()> {
        let request = HostRequest::Snapshot {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("snapshot"),
            input_generation: 2,
            deadline_mono_ms: deadline(10_000)?,
            sample_id: "ELEMENT-01".into(),
            target_identity: TargetIdentity::fixture("fixture", "O-NATIVE-ELEMENT-IDENTITY-V1"),
        };
        write_frame(&mut self.input, &request)?;
        let response = self.read_deadline(supervisor, request.deadline_mono_ms())?;
        if !supervisor.accept_snapshot(&request, &response, monotonic_millis()?) {
            bail!("snapshot 未返回限定引用");
        }
        Ok(())
    }

    fn read_deadline(
        &mut self,
        supervisor: &mut Supervisor,
        deadline_mono_ms: u64,
    ) -> Result<WorkerResponse> {
        let now = monotonic_millis()?;
        let Some(remaining) = deadline_mono_ms.checked_sub(now) else {
            let evidence = self.containment.terminate_and_probe(&mut self.child);
            supervisor.record_termination(evidence);
            self.closed = true;
            bail!("response read deadline 已过期，Worker 已终止并 quarantine");
        };
        let output = &mut self.output;
        let containment = &mut self.containment;
        let child = &mut self.child;
        let result = std::thread::scope(|scope| {
            let (tx, rx) = mpsc::channel();
            scope.spawn(move || {
                let _ = tx.send(read_frame::<WorkerResponse>(output));
            });
            match rx.recv_timeout(Duration::from_millis(remaining)) {
                Ok(Ok(Some(response))) => Ok(response),
                _ => Err(containment.terminate_and_probe(child)),
            }
        });
        match result {
            Ok(response) => Ok(response),
            Err(evidence) => {
                supervisor.record_termination(evidence);
                self.closed = true;
                bail!("response read 失败或超时，Worker 已终止并 quarantine")
            }
        }
    }

    fn shutdown(mut self, supervisor: &mut Supervisor) -> Result<()> {
        let request = HostRequest::Shutdown {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("shutdown"),
            input_generation: 2,
            deadline_mono_ms: deadline(10_000)?,
        };
        write_frame(&mut self.input, &request)?;
        let response = self.read_deadline(supervisor, request.deadline_mono_ms())?;
        if !shutdown_response_matches(&response, request.identity()) {
            let evidence = self.containment.terminate_and_probe(&mut self.child);
            supervisor.record_termination(evidence);
            self.closed = true;
            bail!("shutdown response kind/schema/identity/status/error 无效");
        }
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) if status.success() => break,
                Ok(Some(_)) | Err(_) => {
                    let evidence = self.containment.terminate_and_probe(&mut self.child);
                    supervisor.record_termination(evidence);
                    self.closed = true;
                    bail!("worker shutdown 非正常退出");
                }
                Ok(None) => {
                    if monotonic_millis()? >= request.deadline_mono_ms() {
                        let evidence = self.containment.terminate_and_probe(&mut self.child);
                        supervisor.record_termination(evidence);
                        self.closed = true;
                        bail!("worker shutdown wait 超时");
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }
        let evidence = self.containment.terminate_and_probe(&mut self.child);
        supervisor.record_termination(evidence);
        self.closed = true;
        fs::remove_dir_all(&self.state_dir).ok();
        Ok(())
    }

    fn terminate_into(mut self, supervisor: &mut Supervisor) -> ContainmentVerdict {
        let evidence = self.containment.terminate_and_probe(&mut self.child);
        let verdict = evidence.verdict();
        supervisor.record_termination(evidence);
        self.closed = true;
        fs::remove_dir_all(&self.state_dir).ok();
        verdict
    }
}

impl Drop for WorkerSession {
    fn drop(&mut self) {
        if !self.closed {
            self.host_safety.quarantine();
        }
    }
}

impl DispatchTransport for WorkerSession {
    fn send_dispatch(&mut self, request: &HostRequest, deadline_mono_ms: u64) -> TransportOutcome {
        let Ok(now) = monotonic_millis() else {
            return TransportOutcome::BeforeWrite;
        };
        let Some(remaining) = deadline_mono_ms.checked_sub(now) else {
            return TransportOutcome::BeforeWrite;
        };
        let containment = &mut self.containment;
        let child = &mut self.child;
        deadline_write(
            &mut self.input,
            request,
            Duration::from_millis(remaining),
            || containment.terminate_and_probe(child),
        )
    }
    fn terminate_and_probe(
        &mut self,
    ) -> yonder_agent_desktop_driver_spike::containment::CleanupEvidence {
        self.closed = true;
        self.containment.terminate_and_probe(&mut self.child)
    }
}

fn dispatch_request(deadline_mono_ms: u64) -> HostRequest {
    HostRequest::Dispatch {
        schema_version: BRIDGE_SCHEMA_VERSION.into(),
        identity: identity("dispatch"),
        input_generation: 2,
        deadline_mono_ms,
        sample_id: "ELEMENT-01".into(),
        action_id: ActionId::ActionTypeFixtureTextQualifiedRef,
        snapshot_id: "snap".into(),
        qualified_ref: "@snap:e1".into(),
    }
}
fn observe_request() -> Result<HostRequest> {
    Ok(HostRequest::Observe {
        schema_version: BRIDGE_SCHEMA_VERSION.into(),
        identity: identity("observe"),
        input_generation: 2,
        deadline_mono_ms: deadline(5_000)?,
        sample_id: "ELEMENT-01".into(),
        predicate_id: "P-FIXTURE-STATE-STABLE-V1".into(),
        oracle_id: "O-NATIVE-ELEMENT-IDENTITY-V1".into(),
        prior_snapshot_id: "snap".into(),
        target_identity: yonder_agent_desktop_driver_spike::protocol::TargetIdentity::fixture(
            "fixture",
            "O-NATIVE-ELEMENT-IDENTITY-V1",
        ),
    })
}

fn main() -> Result<()> {
    let worker = PathBuf::from(
        env::args_os()
            .nth(1)
            .context("用法: harness <worker-path>")?,
    );
    if !worker.is_absolute() || !worker.is_file() {
        bail!("worker 必须是已存在的绝对路径");
    }
    let normal_safety = HostSafetyState::default();
    let mut normal = Supervisor::new(state(), normal_safety.clone());
    let mut normal_worker = WorkerSession::start(&worker, false, false, normal_safety)?;
    normal_worker.hello(&mut normal)?;
    normal_worker.snapshot(&mut normal)?;
    let dispatch = dispatch_request(deadline(10_000)?);
    if normal.dispatch(&state(), &dispatch, &mut normal_worker, monotonic_millis()?)
        != DispatchResult::InFlight
    {
        bail!("正常 dispatch 未进入 in_flight");
    }
    let response = normal_worker.read_deadline(&mut normal, dispatch.deadline_mono_ms())?;
    if normal.response(&response, monotonic_millis()?) != DispatchResult::InFlight {
        bail!("dispatch response 无效");
    }
    let observe = observe_request()?;
    write_frame(&mut normal_worker.input, &observe)?;
    let response = normal_worker.read_deadline(&mut normal, observe.deadline_mono_ms())?;
    struct FixtureOracle {
        state_file: PathBuf,
        sampled: bool,
    }
    impl ReadOnlyOracle for FixtureOracle {
        fn sample(&mut self, target: &TargetIdentity, predicate: &str) -> OracleSample {
            if self.sampled {
                std::thread::sleep(Duration::from_millis(250));
            }
            self.sampled = true;
            let observed = fs::read_to_string(&self.state_file).ok();
            let expected = TargetIdentity::fixture("fixture", "O-NATIVE-ELEMENT-IDENTITY-V1");
            OracleSample {
                identity_match: target == &expected,
                predicate_satisfied: predicate == "P-FIXTURE-STATE-STABLE-V1"
                    && observed.as_deref() == Some("ELEMENT-01"),
                monotonic_ms: monotonic_millis().unwrap_or(0),
            }
        }
    }
    let mut oracle = FixtureOracle {
        state_file: normal_worker.state_dir.join("contract-only-fixture-state"),
        sampled: false,
    };
    if normal.observe(&observe, &response, monotonic_millis()?, &mut oracle)
        != DispatchResult::Observed
    {
        bail!("Observe 未验证");
    }
    normal_worker.shutdown(&mut normal)?;
    if !normal.is_quarantined() {
        bail!("orderly shutdown 未记录保守 cleanup/quarantine");
    }

    let pre_cancel_safety = HostSafetyState::default();
    let mut pre_cancel = Supervisor::new(state(), pre_cancel_safety.clone());
    let mut pre_cancel_worker = WorkerSession::start(&worker, false, false, pre_cancel_safety)?;
    if pre_cancel.cancel(&mut pre_cancel_worker) != DispatchResult::NotDispatched {
        bail!("flush 前取消错误");
    }
    let _ = pre_cancel_worker.terminate_into(&mut pre_cancel);

    let deadline_safety = HostSafetyState::default();
    let mut deadline_supervisor = Supervisor::new(state(), deadline_safety.clone());
    let mut deadline_worker = WorkerSession::start(&worker, false, false, deadline_safety)?;
    deadline_worker.hello(&mut deadline_supervisor)?;
    let now = monotonic_millis()?;
    if deadline_supervisor.dispatch(&state(), &dispatch_request(now), &mut deadline_worker, now)
        != DispatchResult::NotDispatched
    {
        bail!("过期 deadline 未拒绝");
    }
    deadline_worker.shutdown(&mut deadline_supervisor)?;

    let malformed_safety = HostSafetyState::default();
    let mut malformed = Supervisor::new(state(), malformed_safety.clone());
    let blocking_safety = HostSafetyState::default();
    let mut blocking = Supervisor::new(state(), blocking_safety.clone());
    let mut blocking_worker = WorkerSession::start(&worker, false, true, blocking_safety)?;
    blocking_worker.hello(&mut blocking)?;
    blocking_worker.snapshot(&mut blocking)?;
    let blocking_request = dispatch_request(deadline(100)?);
    if blocking.dispatch(
        &state(),
        &blocking_request,
        &mut blocking_worker,
        monotonic_millis()?,
    ) != DispatchResult::InFlight
    {
        bail!("阻塞场景未派发");
    }
    if blocking_worker
        .read_deadline(&mut blocking, blocking_request.deadline_mono_ms())
        .is_ok()
    {
        bail!("阻塞 response read 未按请求 deadline 终止");
    }
    if !blocking.is_quarantined() {
        bail!("deadline kill 未原子回灌 quarantine");
    }

    let mut malformed_worker = WorkerSession::start(&worker, true, false, malformed_safety)?;
    malformed_worker.hello(&mut malformed)?;
    malformed_worker.snapshot(&mut malformed)?;
    let request = dispatch_request(deadline(10_000)?);
    if malformed.dispatch(
        &state(),
        &request,
        &mut malformed_worker,
        monotonic_millis()?,
    ) != DispatchResult::InFlight
    {
        bail!("malformed 场景未派发");
    }
    let bad_response =
        malformed_worker.read_deadline(&mut malformed, request.deadline_mono_ms())?;
    if malformed.response(&bad_response, monotonic_millis()?) != DispatchResult::Unknown {
        bail!("malformed response 未映射 unknown");
    }
    let _ = malformed_worker.terminate_into(&mut malformed);

    let killed_safety = HostSafetyState::default();
    let mut killed = Supervisor::new(state(), killed_safety.clone());
    let mut killed_worker = WorkerSession::start(&worker, false, false, killed_safety.clone())?;
    killed_worker.hello(&mut killed)?;
    killed_worker.snapshot(&mut killed)?;
    let request = dispatch_request(deadline(10_000)?);
    if killed.dispatch(&state(), &request, &mut killed_worker, monotonic_millis()?)
        != DispatchResult::InFlight
    {
        bail!("kill 场景未派发");
    }
    if killed.cancel(&mut killed_worker) != DispatchResult::Unknown {
        bail!("flush 后 kill 未映射 unknown");
    }
    let _cleanup = killed_worker.terminate_into(&mut killed);
    let mut quarantined = Supervisor::new(state(), killed_safety);
    let values = hashes();
    if quarantined.accept_hello(
        &WorkerResponse::HelloResult {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("hello"),
            status: yonder_agent_desktop_driver_spike::protocol::Status::Ok,
            error_class: yonder_agent_desktop_driver_spike::protocol::ErrorClass::None,
            source_revision: SOURCE_REVISION.into(),
            bridge_contract_sha256: values[0].clone(),
            sample_definitions_sha256: values[1].clone(),
            action_allowlist_sha256: values[2].clone(),
            capabilities_sha256: yonder_agent_desktop_driver_spike::protocol::capabilities_sha256(),
        },
        &identity("hello"),
        &[&values[0], &values[1], &values[2]],
    ) {
        bail!("宿主 quarantine 被新 Supervisor 绕过");
    }
    let missed_safety = HostSafetyState::default();
    let mut missed = Supervisor::new(state(), missed_safety.clone());
    let mut missed_worker = WorkerSession::start(&worker, false, false, missed_safety.clone())?;
    missed_worker.hello(&mut missed)?;
    drop(missed_worker);
    if !missed_safety.is_quarantined() {
        bail!("漏记 cleanup 未触发 quarantine");
    }

    std::io::stdout().write_all(b"{\"handshake\":true,\"contract_only_observe_protocol\":true,\"real_action_evidence\":false,\"pre_cancel\":true,\"post_flush_kill\":true,\"blocking_io_deadline_kill\":true,\"deadline\":true,\"malformed_worker\":true,\"quarantine\":true,\"missed_cleanup_quarantine\":true,\"native_evidence\":false}\n")?;
    Ok(())
}
