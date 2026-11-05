use std::{
    error::Error,
    io::{BufReader, Write},
    process::{Child, Command, Stdio},
};

use sha2::{Digest, Sha256};
use tempfile::TempDir;
use yonder_agent_desktop_driver_spike::{
    BRIDGE_SCHEMA_VERSION, SOURCE_REVISION,
    clock::monotonic_millis,
    framing::{read_frame, write_frame},
    protocol::{
        ActionId, HostRequest, Identity, Status, TargetIdentity, WorkerResponse,
        hello_identity_matches,
    },
};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn start() -> Result<(Child, TempDir), Box<dyn Error>> {
    let home = TempDir::new()?;
    let child = Command::new(env!("CARGO_BIN_EXE_worker"))
        .arg("--mock")
        .env("AGENT_DESKTOP_HOME", home.path())
        .env("AGENT_DESKTOP_TRACE", "off")
        .env("AGENT_DESKTOP_SCREENSHOTS", "off")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    Ok((child, home))
}

fn identity(request: &str) -> Identity {
    Identity {
        request_id: request.into(),
        attempt_id: "attempt".into(),
        worker_instance_id: "worker".into(),
        fence: 9,
    }
}

fn deadline() -> Result<u64, Box<dyn Error>> {
    Ok(monotonic_millis()?
        .checked_add(10_000)
        .ok_or("deadline overflow")?)
}

fn hashes() -> [String; 3] {
    [
        hash(include_bytes!(
            "../../../openspec/changes/cu-s4-agent-desktop-driver-spike/bridge-contract-v1.md"
        )),
        hash(include_bytes!(
            "../../../openspec/changes/cu-s4-agent-desktop-driver-spike/sample-definitions-v1.json"
        )),
        hash(include_bytes!(
            "../../../openspec/changes/cu-s4-agent-desktop-driver-spike/action-allowlist-v1.json"
        )),
    ]
}

#[test]
fn handshake_dispatch_observe_shutdown_round_trip() -> Result<(), Box<dyn Error>> {
    let (mut child, _home) = start()?;
    let mut input = child.stdin.take().ok_or("stdin")?;
    let mut output = BufReader::new(child.stdout.take().ok_or("stdout")?);
    let h = hashes();
    let hello_id = identity("hello");
    write_frame(
        &mut input,
        &HostRequest::Hello {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: hello_id.clone(),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            source_revision: SOURCE_REVISION.into(),
            bridge_contract_sha256: h[0].clone(),
            sample_definitions_sha256: h[1].clone(),
            action_allowlist_sha256: h[2].clone(),
        },
    )?;
    let response: WorkerResponse = read_frame(&mut output)?.ok_or("hello eof")?;
    assert!(hello_identity_matches(
        &response,
        &hello_id,
        &[&h[0], &h[1], &h[2]]
    ));

    let target = TargetIdentity::fixture("fixture", "O-NATIVE-ELEMENT-IDENTITY-V1");
    write_frame(
        &mut input,
        &HostRequest::Snapshot {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("snapshot"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            sample_id: "ELEMENT-01".into(),
            target_identity: target.clone(),
        },
    )?;
    let response: WorkerResponse = read_frame(&mut output)?.ok_or("snapshot eof")?;
    assert!(matches!(
        response,
        WorkerResponse::SnapshotResult {
            status: Status::Ok,
            ..
        }
    ));

    write_frame(
        &mut input,
        &HostRequest::Dispatch {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("dispatch"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            sample_id: "ELEMENT-01".into(),
            action_id: ActionId::ActionTypeFixtureTextQualifiedRef,
            snapshot_id: "snap".into(),
            qualified_ref: "@snap:e1".into(),
        },
    )?;
    let response: WorkerResponse = read_frame(&mut output)?.ok_or("dispatch eof")?;
    assert!(matches!(
        response,
        WorkerResponse::DispatchResult {
            status: Status::Ok,
            ..
        }
    ));

    write_frame(
        &mut input,
        &HostRequest::Observe {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("observe"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            sample_id: "ELEMENT-01".into(),
            predicate_id: "P-FIXTURE-STATE-STABLE-V1".into(),
            oracle_id: "O-NATIVE-ELEMENT-IDENTITY-V1".into(),
            prior_snapshot_id: "snap".into(),
            target_identity: target,
        },
    )?;
    let response: WorkerResponse = read_frame(&mut output)?.ok_or("observe eof")?;
    assert!(matches!(
        response,
        WorkerResponse::ObserveResult {
            status: Status::Ok,
            ..
        }
    ));

    write_frame(
        &mut input,
        &HostRequest::Shutdown {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("shutdown"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
        },
    )?;
    let _: WorkerResponse = read_frame(&mut output)?.ok_or("shutdown eof")?;
    drop(input);
    assert!(child.wait()?.success());
    Ok(())
}

#[test]
fn dispatch_before_snapshot_is_terminal() -> Result<(), Box<dyn Error>> {
    let (mut child, _home) = start()?;
    let mut input = child.stdin.take().ok_or("stdin")?;
    let mut output = BufReader::new(child.stdout.take().ok_or("stdout")?);
    let h = hashes();
    write_frame(
        &mut input,
        &HostRequest::Hello {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("hello"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            source_revision: SOURCE_REVISION.into(),
            bridge_contract_sha256: h[0].clone(),
            sample_definitions_sha256: h[1].clone(),
            action_allowlist_sha256: h[2].clone(),
        },
    )?;
    let _: WorkerResponse = read_frame(&mut output)?.ok_or("hello eof")?;
    write_frame(
        &mut input,
        &HostRequest::Dispatch {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("dispatch"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            sample_id: "ELEMENT-01".into(),
            action_id: ActionId::ActionTypeFixtureTextQualifiedRef,
            snapshot_id: "snap".into(),
            qualified_ref: "@snap:e1".into(),
        },
    )?;
    drop(input);
    assert!(read_frame::<WorkerResponse>(&mut output)?.is_none());
    assert!(!child.wait()?.success());
    Ok(())
}

#[test]
fn dispatch_sample_cannot_drift_from_snapshot_sample() -> Result<(), Box<dyn Error>> {
    let (mut child, _home) = start()?;
    let mut input = child.stdin.take().ok_or("stdin")?;
    let mut output = BufReader::new(child.stdout.take().ok_or("stdout")?);
    let h = hashes();
    write_frame(
        &mut input,
        &HostRequest::Hello {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("hello"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            source_revision: SOURCE_REVISION.into(),
            bridge_contract_sha256: h[0].clone(),
            sample_definitions_sha256: h[1].clone(),
            action_allowlist_sha256: h[2].clone(),
        },
    )?;
    let _: WorkerResponse = read_frame(&mut output)?.ok_or("hello eof")?;
    write_frame(
        &mut input,
        &HostRequest::Snapshot {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("snapshot"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            sample_id: "ELEMENT-01".into(),
            target_identity: TargetIdentity::fixture("fixture", "O-NATIVE-ELEMENT-IDENTITY-V1"),
        },
    )?;
    let _: WorkerResponse = read_frame(&mut output)?.ok_or("snapshot eof")?;
    write_frame(
        &mut input,
        &HostRequest::Dispatch {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("dispatch"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            sample_id: "IDENTITY-01".into(),
            action_id: ActionId::ActionClickQualifiedRef,
            snapshot_id: "snap".into(),
            qualified_ref: "@snap:e1".into(),
        },
    )?;
    drop(input);
    assert!(read_frame::<WorkerResponse>(&mut output)?.is_none());
    assert!(!child.wait()?.success());
    Ok(())
}

#[test]
fn malformed_or_oversize_input_closes_worker() -> Result<(), Box<dyn Error>> {
    for bytes in [b"not-json\n".to_vec(), vec![b'x'; 65_537]] {
        let (mut child, _home) = start()?;
        child.stdin.as_mut().ok_or("stdin")?.write_all(&bytes)?;
        drop(child.stdin.take());
        assert!(!child.wait()?.success());
    }
    Ok(())
}

#[test]
fn privacy_flags_are_required_not_defaulted() -> Result<(), Box<dyn Error>> {
    let home = TempDir::new()?;
    let status = Command::new(env!("CARGO_BIN_EXE_worker"))
        .arg("--mock")
        .env("AGENT_DESKTOP_HOME", home.path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    assert!(!status.success());
    Ok(())
}

#[cfg(target_os = "windows")]
#[test]
fn native_snapshot_rejection_is_reported_before_worker_exits() -> Result<(), Box<dyn Error>> {
    use yonder_agent_desktop_driver_spike::protocol::{ErrorClass, IdentityVerdict};

    let home = TempDir::new()?;
    let mut child = Command::new(env!("CARGO_BIN_EXE_worker"))
        .env("AGENT_DESKTOP_HOME", home.path().join("native-home"))
        .env("AGENT_DESKTOP_TRACE", "off")
        .env("AGENT_DESKTOP_SCREENSHOTS", "off")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let mut input = child.stdin.take().ok_or("stdin")?;
    let mut output = BufReader::new(child.stdout.take().ok_or("stdout")?);
    let h = hashes();
    write_frame(
        &mut input,
        &HostRequest::Hello {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("hello"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            source_revision: SOURCE_REVISION.into(),
            bridge_contract_sha256: h[0].clone(),
            sample_definitions_sha256: h[1].clone(),
            action_allowlist_sha256: h[2].clone(),
        },
    )?;
    assert!(matches!(
        read_frame::<WorkerResponse>(&mut output)?.ok_or("hello eof")?,
        WorkerResponse::HelloResult {
            status: Status::Ok,
            ..
        }
    ));
    write_frame(
        &mut input,
        &HostRequest::Snapshot {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("snapshot"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            sample_id: "ELEMENT-01".into(),
            target_identity: TargetIdentity::fixture("fixture", "O-NATIVE-ELEMENT-IDENTITY-V1"),
        },
    )?;
    assert!(matches!(
        read_frame::<WorkerResponse>(&mut output)?.ok_or("snapshot eof")?,
        WorkerResponse::SnapshotResult {
            status: Status::Rejected,
            error_class: ErrorClass::Unsupported,
            target_identity_verdict: IdentityVerdict::Unavailable,
            snapshot_id,
            qualified_refs,
            ..
        } if snapshot_id.is_empty() && qualified_refs.is_empty()
    ));
    drop(input);
    assert!(read_frame::<WorkerResponse>(&mut output)?.is_none());
    assert!(!child.wait()?.success());
    Ok(())
}

#[cfg(target_os = "windows")]
fn native_identity01_request(
    target_identity: TargetIdentity,
) -> Result<(WorkerResponse, bool), Box<dyn Error>> {
    let home = TempDir::new()?;
    let mut worker = Command::new(env!("CARGO_BIN_EXE_worker"))
        .env("AGENT_DESKTOP_HOME", home.path().join("native-home"))
        .env("AGENT_DESKTOP_TRACE", "off")
        .env("AGENT_DESKTOP_SCREENSHOTS", "off")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    let mut input = worker.stdin.take().ok_or("worker stdin")?;
    let mut output = BufReader::new(worker.stdout.take().ok_or("worker stdout")?);
    let h = hashes();
    write_frame(
        &mut input,
        &HostRequest::Hello {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("hello"),
            input_generation: 2,
            deadline_mono_ms: deadline()?,
            source_revision: SOURCE_REVISION.into(),
            bridge_contract_sha256: h[0].clone(),
            sample_definitions_sha256: h[1].clone(),
            action_allowlist_sha256: h[2].clone(),
        },
    )?;
    assert!(matches!(
        read_frame::<WorkerResponse>(&mut output)?.ok_or("hello eof")?,
        WorkerResponse::HelloResult {
            status: Status::Ok,
            ..
        }
    ));
    write_frame(
        &mut input,
        &HostRequest::Snapshot {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("snapshot"),
            input_generation: 2,
            deadline_mono_ms: monotonic_millis()?.checked_add(20_000).ok_or("deadline")?,
            sample_id: "IDENTITY-01".into(),
            target_identity,
        },
    )?;
    let response = read_frame(&mut output)?.ok_or("snapshot eof")?;
    drop(input);
    let success = worker.wait()?.success();
    Ok((response, success))
}

#[cfg(target_os = "windows")]
#[test]
fn native_identity01_snapshot_uses_frozen_fixture_window() -> Result<(), Box<dyn Error>> {
    use std::{io::BufRead, path::Path};
    use yonder_agent_desktop_driver_spike::host_oracle::windows::capture_identity01_window;
    use yonder_agent_desktop_driver_spike::protocol::ErrorClass;

    struct Fixture(Child);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/identity01-windows.ps1");
    let mut fixture = Fixture(
        Command::new("powershell.exe")
            .args(["-Sta", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?,
    );
    let mut fixture_output = BufReader::new(fixture.0.stdout.take().ok_or("fixture stdout")?);
    let mut line = String::new();
    if fixture_output.read_line(&mut line)? == 0 {
        return Err("fixture startup eof".into());
    }
    let started: serde_json::Value = serde_json::from_str(&line)?;
    let pid = u32::try_from(started["pid"].as_u64().ok_or("fixture pid")?)?;
    let target_id = started["target_window_id"]
        .as_str()
        .ok_or("fixture target")?;
    let decoy_id = started["decoy_window_id"].as_str().ok_or("fixture decoy")?;
    let target = capture_identity01_window(pid, target_id)?;
    let decoy = capture_identity01_window(pid, decoy_id)?;
    assert_eq!((target.role, decoy.role), (1, 2));
    assert_eq!((target.click_count, decoy.click_count), (1, 1));
    assert_ne!(target.window_token_sha256, decoy.window_token_sha256);

    let identity = TargetIdentity {
        platform: "windows".into(),
        fixture_id: "identity-01".into(),
        process_id: pid,
        native_window_id: target_id.into(),
        process_start_sha256: target.window.process_start_sha256,
        window_token_sha256: target.window_token_sha256,
        topology_generation: 1,
        oracle_id: "O-NATIVE-WINDOW-IDENTITY-V1".into(),
    };
    let (response, success) = native_identity01_request(identity.clone())?;
    assert!(
        matches!(
            &response,
            WorkerResponse::SnapshotResult {
                status: Status::Ok,
                qualified_refs,
                ..
            } if qualified_refs.len() == 1
        ),
        "{response:?}"
    );
    assert!(success);
    let mut wrong_generation = identity.clone();
    wrong_generation.window_token_sha256 = decoy.window_token_sha256.clone();
    let (response, success) = native_identity01_request(wrong_generation)?;
    assert!(matches!(
        response,
        WorkerResponse::SnapshotResult {
            status: Status::Rejected,
            error_class: ErrorClass::Identity,
            snapshot_id,
            qualified_refs,
            ..
        } if snapshot_id.is_empty() && qualified_refs.is_empty()
    ));
    assert!(!success);
    let mut decoy_identity = identity;
    decoy_identity.native_window_id = decoy_id.into();
    decoy_identity.window_token_sha256 = decoy.window_token_sha256;
    let (response, success) = native_identity01_request(decoy_identity)?;
    assert!(matches!(
        response,
        WorkerResponse::SnapshotResult {
            status: Status::Rejected,
            error_class: ErrorClass::Identity,
            snapshot_id,
            qualified_refs,
            ..
        } if snapshot_id.is_empty() && qualified_refs.is_empty()
    ));
    assert!(!success);
    let after_target = capture_identity01_window(pid, target_id)?;
    let after_decoy = capture_identity01_window(pid, decoy_id)?;
    assert_eq!((after_target.click_count, after_decoy.click_count), (1, 1));
    Ok(())
}
