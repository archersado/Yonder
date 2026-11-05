use std::{
    env, fs,
    io::{self, BufReader},
    path::Path,
};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};
use yonder_agent_desktop_driver_spike::{
    BRIDGE_SCHEMA_VERSION, SOURCE_REVISION,
    backend::{Backend, MockBackend},
    clock::monotonic_millis,
    framing::{read_frame, write_frame},
    protocol::{
        ErrorClass, HostRequest, SessionBinding, Status, WorkerResponse, capabilities_sha256,
    },
};

const CONTRACT: &[u8] = include_bytes!(
    "../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/bridge-contract-v1.md"
);
const DEFINITIONS: &[u8] = include_bytes!(
    "../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/sample-definitions-v1.json"
);
const ALLOWLIST: &[u8] = include_bytes!(
    "../../../../openspec/changes/cu-s4-agent-desktop-driver-spike/action-allowlist-v1.json"
);

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn main() -> Result<()> {
    let home = env::var_os("AGENT_DESKTOP_HOME").context("必须设置私有 AGENT_DESKTOP_HOME")?;
    let home = Path::new(&home);
    if !home.is_absolute() {
        bail!("AGENT_DESKTOP_HOME 必须是绝对路径");
    }
    fs::create_dir_all(home)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(home, fs::Permissions::from_mode(0o700))?;
    }
    if env::var_os("AGENT_DESKTOP_TRACE").as_deref() != Some(std::ffi::OsStr::new("off")) {
        bail!("trace 必须显式关闭");
    }
    if env::var_os("AGENT_DESKTOP_SCREENSHOTS").as_deref() != Some(std::ffi::OsStr::new("off")) {
        bail!("screenshots 必须显式关闭");
    }

    let mut backend: Box<dyn Backend> = backend()?;
    let malformed_response = env::args().any(|arg| arg == "--mock-malformed-response");
    let block_dispatch = env::args().any(|arg| arg == "--mock-block-dispatch");
    serve(&mut *backend, malformed_response, block_dispatch, home)
}

fn backend() -> Result<Box<dyn Backend>> {
    if env::args().any(|arg| arg == "--mock") {
        return Ok(Box::new(MockBackend));
    }
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        Ok(Box::new(
            yonder_agent_desktop_driver_spike::backend::native::NativeBackend::new(),
        ))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    bail!("此平台只允许 --mock 契约测试")
}

#[derive(Debug)]
enum Sequence {
    AwaitHello,
    Ready(SessionBinding),
    SnapshotReady(
        SessionBinding,
        String,
        String,
        String,
        yonder_agent_desktop_driver_spike::protocol::TargetIdentity,
    ),
    AwaitObserve(
        SessionBinding,
        String,
        String,
        String,
        yonder_agent_desktop_driver_spike::protocol::TargetIdentity,
    ),
    Observed(SessionBinding),
}

fn serve(
    backend: &mut dyn Backend,
    malformed_response: bool,
    block_dispatch: bool,
    state_home: &Path,
) -> Result<()> {
    let stdin = io::stdin();
    let mut input = BufReader::new(stdin.lock());
    let stdout = io::stdout();
    let mut output = stdout.lock();
    let expected_hashes = [hash(CONTRACT), hash(DEFINITIONS), hash(ALLOWLIST)];
    let mut sequence = Sequence::AwaitHello;
    while let Some(request) = read_frame::<HostRequest>(&mut input).map_err(io::Error::from)? {
        if !request.validate_common() || request.deadline_mono_ms() <= monotonic_millis()? {
            bail!("协议、身份或 deadline 无效");
        }
        match &request {
            HostRequest::Hello {
                identity,
                source_revision,
                bridge_contract_sha256,
                sample_definitions_sha256,
                action_allowlist_sha256,
                ..
            } => {
                if !matches!(sequence, Sequence::AwaitHello)
                    || source_revision != SOURCE_REVISION
                    || [
                        bridge_contract_sha256,
                        sample_definitions_sha256,
                        action_allowlist_sha256,
                    ] != [
                        &expected_hashes[0],
                        &expected_hashes[1],
                        &expected_hashes[2],
                    ]
                {
                    bail!("hello 身份或哈希不匹配");
                }
                sequence = Sequence::Ready(identity.session_binding(request.input_generation()));
                write_frame(
                    &mut output,
                    &WorkerResponse::HelloResult {
                        schema_version: BRIDGE_SCHEMA_VERSION.into(),
                        identity: identity.clone(),
                        status: Status::Ok,
                        error_class: ErrorClass::None,
                        source_revision: SOURCE_REVISION.into(),
                        bridge_contract_sha256: expected_hashes[0].clone(),
                        sample_definitions_sha256: expected_hashes[1].clone(),
                        action_allowlist_sha256: expected_hashes[2].clone(),
                        capabilities_sha256: capabilities_sha256(),
                    },
                )
                .map_err(io::Error::from)?;
            }
            HostRequest::Snapshot {
                identity,
                sample_id,
                target_identity,
                ..
            } => {
                let Sequence::Ready(binding) = &sequence else {
                    bail!("snapshot 顺序无效")
                };
                if !request.matches_session(binding) || !request.frozen_binding_valid() {
                    bail!("snapshot 身份、样本或目标身份错配");
                }
                let reply = backend.snapshot(&request);
                if reply.status != Status::Ok
                    || reply.identity
                        != yonder_agent_desktop_driver_spike::protocol::IdentityVerdict::Match
                    || reply.snapshot_id.is_empty()
                    || reply.qualified_refs.len() != 1
                    || !reply.qualified_refs[0].starts_with(&format!("@{}:", reply.snapshot_id))
                {
                    bail!("snapshot 未建立冻结目标与唯一限定引用");
                }
                let binding = binding.clone();
                sequence = Sequence::SnapshotReady(
                    binding,
                    sample_id.clone(),
                    reply.snapshot_id.clone(),
                    reply.qualified_refs[0].clone(),
                    target_identity.clone(),
                );
                write_frame(
                    &mut output,
                    &WorkerResponse::SnapshotResult {
                        schema_version: BRIDGE_SCHEMA_VERSION.into(),
                        identity: identity.clone(),
                        status: reply.status,
                        error_class: reply.error_class,
                        snapshot_id: reply.snapshot_id,
                        qualified_refs: reply.qualified_refs,
                        target_identity_verdict: reply.identity,
                    },
                )
                .map_err(io::Error::from)?;
            }
            HostRequest::Dispatch { identity, .. } => {
                let Sequence::SnapshotReady(
                    binding,
                    frozen_sample,
                    snapshot_id,
                    qualified_ref,
                    target_identity,
                ) = &sequence
                else {
                    bail!("dispatch 顺序无效")
                };
                let HostRequest::Dispatch {
                    sample_id,
                    snapshot_id: requested_snapshot,
                    qualified_ref: requested_ref,
                    ..
                } = &request
                else {
                    unreachable!()
                };
                if !request.matches_session(binding)
                    || !request.frozen_binding_valid()
                    || sample_id != frozen_sample
                    || requested_snapshot != snapshot_id
                    || requested_ref != qualified_ref
                {
                    bail!("sample/action/ref 错配");
                }
                if malformed_response {
                    write_frame(
                        &mut output,
                        &WorkerResponse::ShutdownResult {
                            schema_version: BRIDGE_SCHEMA_VERSION.into(),
                            identity: identity.clone(),
                            status: Status::Ok,
                            error_class: ErrorClass::None,
                        },
                    )
                    .map_err(io::Error::from)?;
                    bail!("故意发送畸形响应后终止会话");
                }
                if block_dispatch {
                    std::thread::sleep(std::time::Duration::from_secs(30));
                }
                let reply = backend.dispatch(&request);
                if reply.status == Status::Ok && reply.driver_status.starts_with("mock_") {
                    fs::write(
                        state_home.join("contract-only-fixture-state"),
                        frozen_sample.as_bytes(),
                    )?;
                }
                write_frame(
                    &mut output,
                    &WorkerResponse::DispatchResult {
                        schema_version: BRIDGE_SCHEMA_VERSION.into(),
                        identity: identity.clone(),
                        status: reply.status,
                        error_class: reply.error_class,
                        delivery_disposition: reply.disposition,
                        driver_status: reply.driver_status.into(),
                        next_snapshot_hint: None,
                    },
                )
                .map_err(io::Error::from)?;
                sequence = Sequence::AwaitObserve(
                    binding.clone(),
                    frozen_sample.clone(),
                    snapshot_id.clone(),
                    qualified_ref.clone(),
                    target_identity.clone(),
                );
            }
            HostRequest::Observe {
                identity,
                sample_id,
                oracle_id,
                prior_snapshot_id,
                target_identity,
                ..
            } => {
                let Sequence::AwaitObserve(
                    binding,
                    frozen_sample,
                    snapshot_id,
                    _qualified_ref,
                    frozen_target,
                ) = &sequence
                else {
                    bail!("observe 顺序无效")
                };
                if !request.matches_session(binding)
                    || !request.frozen_binding_valid()
                    || sample_id != frozen_sample
                    || oracle_id != &frozen_target.oracle_id
                    || prior_snapshot_id != snapshot_id
                    || target_identity != frozen_target
                {
                    bail!("sample/predicate/oracle 错配");
                }
                let reply = backend.observe(&request);
                let verified = reply.status == Status::Ok
                    && reply.identity
                        == yonder_agent_desktop_driver_spike::protocol::IdentityVerdict::Match
                    && reply.predicate
                        == yonder_agent_desktop_driver_spike::protocol::PredicateVerdict::Satisfied;
                write_frame(
                    &mut output,
                    &WorkerResponse::ObserveResult {
                        schema_version: BRIDGE_SCHEMA_VERSION.into(),
                        identity: identity.clone(),
                        status: if verified {
                            Status::Ok
                        } else {
                            Status::Rejected
                        },
                        error_class: if verified {
                            ErrorClass::None
                        } else {
                            ErrorClass::Observe
                        },
                        snapshot_id: reply.snapshot_id,
                        target_identity_verdict: reply.identity,
                        predicate_verdict: reply.predicate,
                        element_count: reply.element_count,
                    },
                )
                .map_err(io::Error::from)?;
                sequence = Sequence::Observed(binding.clone());
            }
            HostRequest::Shutdown { identity, .. } => {
                let binding = match &sequence {
                    Sequence::Ready(binding) | Sequence::Observed(binding) => binding,
                    _ => bail!("shutdown 只能在副作用前或完整 Observe 后发生"),
                };
                if !request.matches_session(binding) {
                    bail!("shutdown 会话错配")
                }
                write_frame(
                    &mut output,
                    &WorkerResponse::ShutdownResult {
                        schema_version: BRIDGE_SCHEMA_VERSION.into(),
                        identity: identity.clone(),
                        status: Status::Ok,
                        error_class: ErrorClass::None,
                    },
                )
                .map_err(io::Error::from)?;
                break;
            }
        }
    }
    Ok(())
}
