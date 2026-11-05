use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{BRIDGE_SCHEMA_VERSION, SOURCE_REVISION};

pub const MAX_FRAME_BYTES: usize = 65_536;
pub const MAX_ID_BYTES: usize = 128;
pub const CAPABILITIES_DESCRIPTOR: &[u8] = b"snapshot|qualified-ref|observe|shutdown";

pub fn capabilities_sha256() -> String {
    format!("{:x}", Sha256::digest(CAPABILITIES_DESCRIPTOR))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub request_id: String,
    pub attempt_id: String,
    pub worker_instance_id: String,
    pub fence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionBinding {
    pub attempt_id: String,
    pub worker_instance_id: String,
    pub fence: u64,
    pub input_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TargetIdentity {
    pub platform: String,
    pub fixture_id: String,
    pub process_id: u32,
    pub native_window_id: String,
    pub process_start_sha256: String,
    pub window_token_sha256: String,
    pub topology_generation: u64,
    pub oracle_id: String,
}

impl TargetIdentity {
    pub fn fixture(fixture_id: &str, oracle_id: &str) -> Self {
        let platform = std::env::consts::OS.to_owned();
        let process_id = std::process::id();
        let native_window_id = format!("fixture:{fixture_id}:{process_id}");
        let topology_generation = 1;
        Self {
            process_start_sha256: format!(
                "{:x}",
                Sha256::digest(
                    format!("{platform}:{fixture_id}:{process_id}:process-start-v1").as_bytes(),
                )
            ),
            window_token_sha256: format!(
                "{:x}",
                Sha256::digest(
                    format!("{native_window_id}:{topology_generation}:{oracle_id}").as_bytes(),
                )
            ),
            platform,
            fixture_id: fixture_id.into(),
            process_id,
            native_window_id,
            topology_generation,
            oracle_id: oracle_id.into(),
        }
    }
    pub fn is_valid(&self) -> bool {
        let valid_hash = |value: &str| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        };
        !self.platform.is_empty()
            && !self.fixture_id.is_empty()
            && self.process_id != 0
            && [&self.native_window_id, &self.oracle_id]
                .into_iter()
                .all(|value| !value.is_empty() && value.len() <= MAX_ID_BYTES)
            && valid_hash(&self.process_start_sha256)
            && valid_hash(&self.window_token_sha256)
    }
}

impl Identity {
    pub fn is_valid(&self) -> bool {
        [&self.request_id, &self.attempt_id, &self.worker_instance_id]
            .into_iter()
            .all(|value| !value.is_empty() && value.len() <= MAX_ID_BYTES)
    }

    pub fn session_binding(&self, input_generation: u64) -> SessionBinding {
        SessionBinding {
            attempt_id: self.attempt_id.clone(),
            worker_instance_id: self.worker_instance_id.clone(),
            fence: self.fence,
            input_generation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING-KEBAB-CASE")]
pub enum ActionId {
    ActionNone,
    ActionClickQualifiedRef,
    ActionTypeFixtureTextQualifiedRef,
    ActionFocusWindowQualifiedRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleBinding {
    pub action_id: ActionId,
    pub predicate_id: &'static str,
    pub oracle_id: &'static str,
}

pub fn sample_binding(sample_id: &str) -> Option<SampleBinding> {
    let binding = match sample_id {
        "IDENTITY-01" => (
            ActionId::ActionClickQualifiedRef,
            "P-TARGET-UNCHANGED-V1",
            "O-NATIVE-WINDOW-IDENTITY-V1",
        ),
        "IDENTITY-02" => (
            ActionId::ActionClickQualifiedRef,
            "P-OLD-TARGET-REJECTED-V1",
            "O-NATIVE-WINDOW-IDENTITY-V1",
        ),
        "ELEMENT-01" => (
            ActionId::ActionTypeFixtureTextQualifiedRef,
            "P-FIXTURE-STATE-STABLE-V1",
            "O-NATIVE-ELEMENT-IDENTITY-V1",
        ),
        "APPSET-01" => (
            ActionId::ActionTypeFixtureTextQualifiedRef,
            "P-APPSET-STATE-STABLE-V1",
            "O-FIXED-APP-WINDOW-V1",
        ),
        "APPSET-02" => (
            ActionId::ActionClickQualifiedRef,
            "P-APPSET-STATE-STABLE-V1",
            "O-FIXED-APP-WINDOW-V1",
        ),
        "FOCUS-01" => (
            ActionId::ActionFocusWindowQualifiedRef,
            "P-TARGET-VISIBLE-V1",
            "O-NATIVE-WINDOW-IDENTITY-V1",
        ),
        "INPUT-01" => (
            ActionId::ActionClickQualifiedRef,
            "P-USER-PREEMPTED-V1",
            "O-INPUT-GENERATION-V1",
        ),
        "INPUT-02" => (
            ActionId::ActionClickQualifiedRef,
            "P-DRIVER-INPUT-ATTRIBUTED-V1",
            "O-INPUT-GENERATION-V1",
        ),
        "INPUT-03" => (
            ActionId::ActionNone,
            "P-OBSERVER-FAIL-CLOSED-V1",
            "O-INPUT-GENERATION-V1",
        ),
        "CANCEL-01" => (
            ActionId::ActionClickQualifiedRef,
            "P-NOT-DISPATCHED-V1",
            "O-DISPATCH-BOUNDARY-V1",
        ),
        "CANCEL-02" => (
            ActionId::ActionClickQualifiedRef,
            "P-INFLIGHT-UNKNOWN-V1",
            "O-WORKER-CONTAINMENT-V1",
        ),
        "FAULT-01" => (
            ActionId::ActionClickQualifiedRef,
            "P-FAULT-UNKNOWN-V1",
            "O-DISPATCH-BOUNDARY-V1",
        ),
        "DISPLAY-01" => (
            ActionId::ActionClickQualifiedRef,
            "P-REF-DISPLAY-INDEPENDENT-V1",
            "O-DISPLAY-TOPOLOGY-V1",
        ),
        "PERMISSION-01" => (
            ActionId::ActionClickQualifiedRef,
            "P-PERMISSION-REJECTED-V1",
            "O-PLATFORM-PERMISSION-V1",
        ),
        "DIST-01" => (
            ActionId::ActionNone,
            "P-DISTRIBUTION-READY-V1",
            "O-BUILD-MANIFEST-V1",
        ),
        _ => return None,
    };
    Some(SampleBinding {
        action_id: binding.0,
        predicate_id: binding.1,
        oracle_id: binding.2,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum HostRequest {
    Hello {
        schema_version: String,
        #[serde(flatten)]
        identity: Identity,
        input_generation: u64,
        deadline_mono_ms: u64,
        source_revision: String,
        bridge_contract_sha256: String,
        sample_definitions_sha256: String,
        action_allowlist_sha256: String,
    },
    Snapshot {
        schema_version: String,
        #[serde(flatten)]
        identity: Identity,
        input_generation: u64,
        deadline_mono_ms: u64,
        sample_id: String,
        target_identity: TargetIdentity,
    },
    Dispatch {
        schema_version: String,
        #[serde(flatten)]
        identity: Identity,
        input_generation: u64,
        deadline_mono_ms: u64,
        sample_id: String,
        action_id: ActionId,
        snapshot_id: String,
        qualified_ref: String,
    },
    Observe {
        schema_version: String,
        #[serde(flatten)]
        identity: Identity,
        input_generation: u64,
        deadline_mono_ms: u64,
        sample_id: String,
        predicate_id: String,
        oracle_id: String,
        prior_snapshot_id: String,
        target_identity: TargetIdentity,
    },
    Shutdown {
        schema_version: String,
        #[serde(flatten)]
        identity: Identity,
        input_generation: u64,
        deadline_mono_ms: u64,
    },
}

impl HostRequest {
    pub fn identity(&self) -> &Identity {
        match self {
            Self::Hello { identity, .. }
            | Self::Snapshot { identity, .. }
            | Self::Dispatch { identity, .. }
            | Self::Observe { identity, .. }
            | Self::Shutdown { identity, .. } => identity,
        }
    }
    pub fn schema_version(&self) -> &str {
        match self {
            Self::Hello { schema_version, .. }
            | Self::Snapshot { schema_version, .. }
            | Self::Dispatch { schema_version, .. }
            | Self::Observe { schema_version, .. }
            | Self::Shutdown { schema_version, .. } => schema_version,
        }
    }
    pub fn input_generation(&self) -> u64 {
        match self {
            Self::Hello {
                input_generation, ..
            }
            | Self::Snapshot {
                input_generation, ..
            }
            | Self::Dispatch {
                input_generation, ..
            }
            | Self::Observe {
                input_generation, ..
            }
            | Self::Shutdown {
                input_generation, ..
            } => *input_generation,
        }
    }
    pub fn deadline_mono_ms(&self) -> u64 {
        match self {
            Self::Hello {
                deadline_mono_ms, ..
            }
            | Self::Snapshot {
                deadline_mono_ms, ..
            }
            | Self::Dispatch {
                deadline_mono_ms, ..
            }
            | Self::Observe {
                deadline_mono_ms, ..
            }
            | Self::Shutdown {
                deadline_mono_ms, ..
            } => *deadline_mono_ms,
        }
    }
    pub fn validate_common(&self) -> bool {
        self.schema_version() == BRIDGE_SCHEMA_VERSION && self.identity().is_valid()
    }
    pub fn matches_session(&self, session: &SessionBinding) -> bool {
        self.identity().attempt_id == session.attempt_id
            && self.identity().worker_instance_id == session.worker_instance_id
            && self.identity().fence == session.fence
            && self.input_generation() == session.input_generation
    }
    pub fn frozen_binding_valid(&self) -> bool {
        match self {
            Self::Snapshot {
                sample_id,
                target_identity,
                ..
            } => {
                sample_binding(sample_id)
                    .is_some_and(|binding| binding.oracle_id == target_identity.oracle_id)
                    && target_identity.is_valid()
            }
            Self::Dispatch {
                sample_id,
                action_id,
                snapshot_id,
                qualified_ref,
                ..
            } => {
                let Some(binding) = sample_binding(sample_id) else {
                    return false;
                };
                let qualified = qualified_ref.strip_prefix('@').is_some_and(|value| {
                    value.starts_with(snapshot_id) && value[snapshot_id.len()..].starts_with(':')
                });
                binding.action_id == *action_id
                    && !snapshot_id.is_empty()
                    && (*action_id == ActionId::ActionNone || qualified)
            }
            Self::Observe {
                sample_id,
                predicate_id,
                oracle_id,
                prior_snapshot_id,
                target_identity,
                ..
            } => {
                sample_binding(sample_id).is_some_and(|binding| {
                    binding.predicate_id == predicate_id && binding.oracle_id == oracle_id
                }) && !prior_snapshot_id.is_empty()
                    && target_identity.is_valid()
            }
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Rejected,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorClass {
    None,
    Schema,
    Identity,
    Deadline,
    Permission,
    Stale,
    Ambiguous,
    Unsupported,
    Driver,
    Observe,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryDisposition {
    NotDelivered,
    DeliveredVerified,
    DeliveredUnverified,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum IdentityVerdict {
    Match,
    Mismatch,
    Ambiguous,
    Stale,
    Unavailable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PredicateVerdict {
    Satisfied,
    Unsatisfied,
    Unstable,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum WorkerResponse {
    HelloResult {
        schema_version: String,
        #[serde(flatten)]
        identity: Identity,
        status: Status,
        error_class: ErrorClass,
        source_revision: String,
        bridge_contract_sha256: String,
        sample_definitions_sha256: String,
        action_allowlist_sha256: String,
        capabilities_sha256: String,
    },
    SnapshotResult {
        schema_version: String,
        #[serde(flatten)]
        identity: Identity,
        status: Status,
        error_class: ErrorClass,
        snapshot_id: String,
        qualified_refs: Vec<String>,
        target_identity_verdict: IdentityVerdict,
    },
    DispatchResult {
        schema_version: String,
        #[serde(flatten)]
        identity: Identity,
        status: Status,
        error_class: ErrorClass,
        delivery_disposition: DeliveryDisposition,
        driver_status: String,
        next_snapshot_hint: Option<String>,
    },
    ObserveResult {
        schema_version: String,
        #[serde(flatten)]
        identity: Identity,
        status: Status,
        error_class: ErrorClass,
        snapshot_id: String,
        target_identity_verdict: IdentityVerdict,
        predicate_verdict: PredicateVerdict,
        element_count: u16,
    },
    ShutdownResult {
        schema_version: String,
        #[serde(flatten)]
        identity: Identity,
        status: Status,
        error_class: ErrorClass,
    },
}

impl WorkerResponse {
    pub fn identity(&self) -> &Identity {
        match self {
            Self::HelloResult { identity, .. }
            | Self::SnapshotResult { identity, .. }
            | Self::DispatchResult { identity, .. }
            | Self::ObserveResult { identity, .. }
            | Self::ShutdownResult { identity, .. } => identity,
        }
    }
    pub fn schema_version(&self) -> &str {
        match self {
            Self::HelloResult { schema_version, .. }
            | Self::SnapshotResult { schema_version, .. }
            | Self::DispatchResult { schema_version, .. }
            | Self::ObserveResult { schema_version, .. }
            | Self::ShutdownResult { schema_version, .. } => schema_version,
        }
    }
}

pub fn hello_identity_matches(
    response: &WorkerResponse,
    expected: &Identity,
    hashes: &[&str; 3],
) -> bool {
    matches!(response, WorkerResponse::HelloResult {
        schema_version, identity, status: Status::Ok, error_class: ErrorClass::None,
        source_revision, bridge_contract_sha256, sample_definitions_sha256, action_allowlist_sha256,
        capabilities_sha256: actual_capabilities
    } if schema_version == BRIDGE_SCHEMA_VERSION && identity == expected && source_revision == SOURCE_REVISION
        && bridge_contract_sha256 == hashes[0] && sample_definitions_sha256 == hashes[1]
        && action_allowlist_sha256 == hashes[2] && actual_capabilities == &capabilities_sha256())
}

pub fn shutdown_response_matches(response: &WorkerResponse, expected: &Identity) -> bool {
    matches!(response, WorkerResponse::ShutdownResult {
        schema_version, identity, status: Status::Ok, error_class: ErrorClass::None,
    } if schema_version == BRIDGE_SCHEMA_VERSION && identity == expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> Identity {
        Identity {
            request_id: "r".into(),
            attempt_id: "a".into(),
            worker_instance_id: "w".into(),
            fence: 1,
        }
    }

    #[test]
    fn dispatch_ref_and_observe_definition_are_frozen_together() {
        let valid = HostRequest::Dispatch {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity(),
            input_generation: 2,
            deadline_mono_ms: 10,
            sample_id: "ELEMENT-01".into(),
            action_id: ActionId::ActionTypeFixtureTextQualifiedRef,
            snapshot_id: "snap".into(),
            qualified_ref: "@snap:e1".into(),
        };
        assert!(valid.frozen_binding_valid());
        let mut wrong_ref = valid.clone();
        if let HostRequest::Dispatch { qualified_ref, .. } = &mut wrong_ref {
            *qualified_ref = "@other:e1".into();
        }
        assert!(!wrong_ref.frozen_binding_valid());
        let wrong_oracle = HostRequest::Observe {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity(),
            input_generation: 2,
            deadline_mono_ms: 10,
            sample_id: "ELEMENT-01".into(),
            predicate_id: "P-FIXTURE-STATE-STABLE-V1".into(),
            oracle_id: "O-NATIVE-WINDOW-IDENTITY-V1".into(),
            prior_snapshot_id: "snap".into(),
            target_identity: TargetIdentity::fixture("fixture", "O-NATIVE-WINDOW-IDENTITY-V1"),
        };
        assert!(!wrong_oracle.frozen_binding_valid());
    }

    #[test]
    fn shutdown_response_requires_exact_kind_identity_and_success() {
        let expected = identity();
        let valid = WorkerResponse::ShutdownResult {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: expected.clone(),
            status: Status::Ok,
            error_class: ErrorClass::None,
        };
        assert!(shutdown_response_matches(&valid, &expected));
        let wrong = WorkerResponse::ShutdownResult {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: Identity {
                request_id: "other".into(),
                ..expected.clone()
            },
            status: Status::Ok,
            error_class: ErrorClass::None,
        };
        assert!(!shutdown_response_matches(&wrong, &expected));
        assert!(!shutdown_response_matches(
            &WorkerResponse::DispatchResult {
                schema_version: BRIDGE_SCHEMA_VERSION.into(),
                identity: expected.clone(),
                status: Status::Ok,
                error_class: ErrorClass::None,
                delivery_disposition: DeliveryDisposition::DeliveredVerified,
                driver_status: "x".into(),
                next_snapshot_hint: None,
            },
            &expected
        ));
    }
}
