use std::{
    io::Write,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    time::Duration,
};

use crate::{
    BRIDGE_SCHEMA_VERSION,
    containment::{CleanupEvidence, CleanupVerdict},
    framing::{FrameWriteError, write_frame},
    protocol::{
        DeliveryDisposition, ErrorClass, HostRequest, Identity, IdentityVerdict, SessionBinding,
        Status, TargetIdentity, WorkerResponse, hello_identity_matches,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchBoundary {
    NotDispatched,
    FrameFlushed,
    ResponseReceived,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlState {
    Running,
    Preempted,
    CancelledBeforeDispatch,
    Quarantined,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchResult {
    NotDispatched,
    InFlight,
    Unknown,
    Observed,
}
#[derive(Debug)]
pub enum TransportOutcome {
    Flushed,
    BeforeWrite,
    DeliveryUnknown(CleanupEvidence),
}

/// 派发 transport 必须自行实施 write/flush deadline，并在不确定交付前 kill+probe。
pub trait DispatchTransport {
    fn send_dispatch(&mut self, request: &HostRequest, deadline_mono_ms: u64) -> TransportOutcome;
    fn terminate_and_probe(&mut self) -> CleanupEvidence;
}

pub fn deadline_write(
    writer: &mut (impl Write + Send),
    request: &HostRequest,
    timeout: Duration,
    kill_and_probe: impl FnOnce() -> CleanupEvidence,
) -> TransportOutcome {
    std::thread::scope(|scope| {
        let (tx, rx) = mpsc::channel();
        scope.spawn(move || {
            let _ = tx.send(write_frame(writer, request));
        });
        match rx.recv_timeout(timeout) {
            Ok(Ok(())) => TransportOutcome::Flushed,
            Ok(Err(FrameWriteError::BeforeWrite(_))) => TransportOutcome::BeforeWrite,
            Ok(Err(FrameWriteError::DeliveryUnknown(_)))
            | Err(mpsc::RecvTimeoutError::Timeout)
            | Err(mpsc::RecvTimeoutError::Disconnected) => {
                TransportOutcome::DeliveryUnknown(kill_and_probe())
            }
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionSnapshot {
    pub fence: u64,
    pub input_generation: u64,
    pub observer_ready: bool,
    pub control: ControlState,
}

#[derive(Debug, Clone)]
pub struct AdmissionGate(Arc<Mutex<AdmissionSnapshot>>);

impl AdmissionGate {
    pub fn new(snapshot: AdmissionSnapshot) -> Self {
        Self(Arc::new(Mutex::new(snapshot)))
    }
    pub fn snapshot(&self) -> AdmissionSnapshot {
        match self.0.lock() {
            Ok(value) => value.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
    pub fn update(&self, update: impl FnOnce(&mut AdmissionSnapshot)) {
        if let Ok(mut value) = self.0.lock() {
            update(&mut value);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingDispatch {
    sample_id: String,
    snapshot_id: String,
    qualified_ref: String,
    target_identity: TargetIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SequenceState {
    AwaitHello,
    Ready,
    SnapshotReady,
    Dispatched,
    Response,
    Terminal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OracleSample {
    pub identity_match: bool,
    pub predicate_satisfied: bool,
    pub monotonic_ms: u64,
}

pub trait ReadOnlyOracle {
    fn sample(&mut self, target: &TargetIdentity, predicate_id: &str) -> OracleSample;
}

#[derive(Debug, Clone, Default)]
pub struct HostSafetyState {
    quarantined: Arc<AtomicBool>,
    max_generation: Arc<AtomicU64>,
}

impl HostSafetyState {
    pub fn is_quarantined(&self) -> bool {
        self.quarantined.load(Ordering::Acquire)
    }
    pub fn quarantine(&self) {
        self.quarantined.store(true, Ordering::Release);
    }
    fn accept_generation(&self, generation: u64) -> bool {
        let mut current = self.max_generation.load(Ordering::Acquire);
        loop {
            if generation < current {
                return false;
            }
            if generation == current {
                return true;
            }
            match self.max_generation.compare_exchange_weak(
                current,
                generation,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }
}

#[derive(Debug)]
pub struct Supervisor {
    admitted: AdmissionSnapshot,
    gate: AdmissionGate,
    host_safety: HostSafetyState,
    boundary: DispatchBoundary,
    session: Option<SessionBinding>,
    expected_dispatch: Option<Identity>,
    dispatch_deadline: Option<u64>,
    dispatch_response_valid: bool,
    sequence: SequenceState,
    snapshot_binding: Option<PendingDispatch>,
    pending: Option<PendingDispatch>,
}

impl Supervisor {
    pub fn new(admitted: AdmissionSnapshot, host_safety: HostSafetyState) -> Self {
        Self::new_with_gate(AdmissionGate::new(admitted.clone()), admitted, host_safety)
    }

    pub fn new_with_gate(
        gate: AdmissionGate,
        admitted: AdmissionSnapshot,
        host_safety: HostSafetyState,
    ) -> Self {
        Self {
            admitted,
            gate,
            host_safety,
            boundary: DispatchBoundary::NotDispatched,
            session: None,
            expected_dispatch: None,
            dispatch_deadline: None,
            dispatch_response_valid: false,
            sequence: SequenceState::AwaitHello,
            snapshot_binding: None,
            pending: None,
        }
    }

    pub fn admission_gate(&self) -> AdmissionGate {
        self.gate.clone()
    }

    pub fn accept_hello(
        &mut self,
        response: &WorkerResponse,
        expected: &Identity,
        hashes: &[&str; 3],
    ) -> bool {
        let valid = self.sequence == SequenceState::AwaitHello
            && !self.host_safety.is_quarantined()
            && expected.fence == self.admitted.fence
            && hello_identity_matches(response, expected, hashes)
            && self
                .host_safety
                .accept_generation(self.admitted.input_generation);
        self.session = valid.then(|| expected.session_binding(self.admitted.input_generation));
        self.sequence = if valid {
            SequenceState::Ready
        } else {
            SequenceState::Terminal
        };
        valid
    }

    pub fn accept_snapshot(
        &mut self,
        request: &HostRequest,
        response: &WorkerResponse,
        now_mono_ms: u64,
    ) -> bool {
        let valid = self.sequence == SequenceState::Ready
            && request.validate_common()
            && request.frozen_binding_valid()
            && request.deadline_mono_ms() > now_mono_ms
            && self
                .session
                .as_ref()
                .is_some_and(|s| request.matches_session(s))
            && response.schema_version() == BRIDGE_SCHEMA_VERSION
            && response.identity() == request.identity()
            && matches!(response, WorkerResponse::SnapshotResult {
                status: Status::Ok, error_class: ErrorClass::None,
                target_identity_verdict: IdentityVerdict::Match, snapshot_id, qualified_refs, ..
            } if !snapshot_id.is_empty() && qualified_refs.len() == 1
                && qualified_refs[0].starts_with(&format!("@{snapshot_id}:")));
        if valid {
            let HostRequest::Snapshot {
                sample_id,
                target_identity,
                ..
            } = request
            else {
                unreachable!()
            };
            let WorkerResponse::SnapshotResult {
                snapshot_id,
                qualified_refs,
                ..
            } = response
            else {
                unreachable!()
            };
            self.snapshot_binding = Some(PendingDispatch {
                sample_id: sample_id.clone(),
                snapshot_id: snapshot_id.clone(),
                qualified_ref: qualified_refs[0].clone(),
                target_identity: target_identity.clone(),
            });
            self.sequence = SequenceState::SnapshotReady;
        } else {
            self.sequence = SequenceState::Terminal;
        }
        valid
    }

    pub fn dispatch(
        &mut self,
        current: &AdmissionSnapshot,
        request: &HostRequest,
        transport: &mut impl DispatchTransport,
        now_mono_ms: u64,
    ) -> DispatchResult {
        let Some(session) = &self.session else {
            self.sequence = SequenceState::Terminal;
            return DispatchResult::NotDispatched;
        };
        let Ok(mut gate) = self.gate.0.lock() else {
            self.sequence = SequenceState::Terminal;
            return DispatchResult::NotDispatched;
        };
        if self.host_safety.is_quarantined()
            || self.sequence != SequenceState::SnapshotReady
            || !request.validate_common()
            || !request.frozen_binding_valid()
            || !request.matches_session(session)
            || self.admitted != *current
            || *gate != *current
            || current.control != ControlState::Running
            || !current.observer_ready
            || request.deadline_mono_ms() <= now_mono_ms
        {
            self.sequence = SequenceState::Terminal;
            return DispatchResult::NotDispatched;
        }
        let HostRequest::Dispatch {
            sample_id,
            snapshot_id,
            qualified_ref,
            ..
        } = request
        else {
            self.sequence = SequenceState::Terminal;
            return DispatchResult::NotDispatched;
        };
        let Some(frozen) = self.snapshot_binding.as_ref() else {
            self.sequence = SequenceState::Terminal;
            return DispatchResult::NotDispatched;
        };
        if frozen.sample_id != *sample_id
            || frozen.snapshot_id != *snapshot_id
            || frozen.qualified_ref != *qualified_ref
        {
            self.sequence = SequenceState::Terminal;
            return DispatchResult::NotDispatched;
        }
        self.pending = Some(frozen.clone());
        self.expected_dispatch = Some(request.identity().clone());
        self.dispatch_deadline = Some(request.deadline_mono_ms());
        match transport.send_dispatch(request, request.deadline_mono_ms()) {
            TransportOutcome::Flushed => {
                self.boundary = DispatchBoundary::FrameFlushed;
                self.sequence = SequenceState::Dispatched;
                DispatchResult::InFlight
            }
            TransportOutcome::BeforeWrite => {
                self.sequence = SequenceState::Terminal;
                DispatchResult::NotDispatched
            }
            TransportOutcome::DeliveryUnknown(cleanup) => {
                self.boundary = DispatchBoundary::FrameFlushed;
                self.sequence = SequenceState::Terminal;
                if matches!(
                    cleanup.verdict(),
                    CleanupVerdict::Residual | CleanupVerdict::Unproven
                ) {
                    self.host_safety.quarantine();
                    self.admitted.control = ControlState::Quarantined;
                    gate.control = ControlState::Quarantined;
                }
                DispatchResult::Unknown
            }
        }
    }

    pub fn cancel(&mut self, transport: &mut impl DispatchTransport) -> DispatchResult {
        let Ok(mut gate) = self.gate.0.lock() else {
            self.sequence = SequenceState::Terminal;
            return DispatchResult::Unknown;
        };
        match self.boundary {
            DispatchBoundary::NotDispatched => {
                self.admitted.control = ControlState::CancelledBeforeDispatch;
                gate.control = ControlState::CancelledBeforeDispatch;
                self.sequence = SequenceState::Terminal;
                DispatchResult::NotDispatched
            }
            DispatchBoundary::FrameFlushed | DispatchBoundary::ResponseReceived => {
                self.sequence = SequenceState::Terminal;
                let cleanup = transport.terminate_and_probe();
                if matches!(
                    cleanup.verdict(),
                    CleanupVerdict::Residual | CleanupVerdict::Unproven
                ) {
                    self.host_safety.quarantine();
                    self.admitted.control = ControlState::Quarantined;
                    gate.control = ControlState::Quarantined;
                }
                DispatchResult::Unknown
            }
        }
    }

    pub fn response(&mut self, response: &WorkerResponse, now_mono_ms: u64) -> DispatchResult {
        let valid = self.sequence == SequenceState::Dispatched
            && self.boundary == DispatchBoundary::FrameFlushed
            && self
                .dispatch_deadline
                .is_some_and(|deadline| deadline > now_mono_ms)
            && self
                .expected_dispatch
                .as_ref()
                .is_some_and(|expected| response.identity() == expected)
            && response.schema_version() == BRIDGE_SCHEMA_VERSION
            && matches!(
                response,
                WorkerResponse::DispatchResult {
                    status: Status::Ok,
                    error_class: ErrorClass::None,
                    delivery_disposition: DeliveryDisposition::DeliveredVerified,
                    ..
                }
            );
        if !valid {
            self.sequence = SequenceState::Terminal;
            return DispatchResult::Unknown;
        }
        self.boundary = DispatchBoundary::ResponseReceived;
        self.dispatch_response_valid = true;
        self.sequence = SequenceState::Response;
        DispatchResult::InFlight
    }

    pub fn observe(
        &mut self,
        request: &HostRequest,
        response: &WorkerResponse,
        now_mono_ms: u64,
        oracle: &mut impl ReadOnlyOracle,
    ) -> DispatchResult {
        let pending_matches = self.pending.as_ref().is_some_and(|pending| {
            matches!(request,
            HostRequest::Observe { sample_id, prior_snapshot_id, target_identity, .. }
            if sample_id == &pending.sample_id && prior_snapshot_id == &pending.snapshot_id
                && target_identity == &pending.target_identity)
        });
        let worker_candidate_valid = self.sequence == SequenceState::Response
            && self.dispatch_response_valid
            && self.boundary == DispatchBoundary::ResponseReceived
            && request.validate_common()
            && request.frozen_binding_valid()
            && self
                .session
                .as_ref()
                .is_some_and(|session| request.matches_session(session))
            && request.deadline_mono_ms() > now_mono_ms
            && response.schema_version() == BRIDGE_SCHEMA_VERSION
            && response.identity() == request.identity()
            && pending_matches
            && matches!(
                response,
                WorkerResponse::ObserveResult {
                    status: Status::Ok,
                    error_class: ErrorClass::None,
                    snapshot_id,
                    target_identity_verdict: IdentityVerdict::Match,
                    predicate_verdict: crate::protocol::PredicateVerdict::Satisfied,
                    ..
                } if !snapshot_id.is_empty()
            );
        let stable = if worker_candidate_valid {
            let HostRequest::Observe {
                target_identity,
                predicate_id,
                ..
            } = request
            else {
                unreachable!()
            };
            let first = oracle.sample(target_identity, predicate_id);
            let second = oracle.sample(target_identity, predicate_id);
            first.identity_match
                && first.predicate_satisfied
                && second.identity_match
                && second.predicate_satisfied
                && second.monotonic_ms >= first.monotonic_ms + 250
                && second.monotonic_ms <= first.monotonic_ms + 500
        } else {
            false
        };
        self.sequence = SequenceState::Terminal;
        if stable {
            DispatchResult::Observed
        } else {
            DispatchResult::Unknown
        }
    }

    fn apply_cleanup(&mut self, evidence: CleanupEvidence) {
        if matches!(
            evidence.verdict(),
            CleanupVerdict::Residual | CleanupVerdict::Unproven
        ) {
            self.host_safety.quarantine();
            self.admitted.control = ControlState::Quarantined;
            self.gate
                .update(|state| state.control = ControlState::Quarantined);
        }
    }
    pub fn record_termination(&mut self, evidence: CleanupEvidence) {
        self.apply_cleanup(evidence);
    }
    pub fn boundary(&self) -> DispatchBoundary {
        self.boundary
    }
    pub fn is_quarantined(&self) -> bool {
        self.host_safety.is_quarantined()
    }
    pub fn retry_count(&self) -> u8 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        SOURCE_REVISION,
        framing::{FrameWriteError, write_frame},
        protocol::{ActionId, PredicateVerdict, TargetIdentity, capabilities_sha256},
    };
    use std::io;
    use std::io::Write;

    struct TestTransport<W>(W);
    impl<W: Write> DispatchTransport for TestTransport<W> {
        fn send_dispatch(&mut self, request: &HostRequest, _deadline: u64) -> TransportOutcome {
            match write_frame(&mut self.0, request) {
                Ok(()) => TransportOutcome::Flushed,
                Err(FrameWriteError::BeforeWrite(_)) => TransportOutcome::BeforeWrite,
                Err(FrameWriteError::DeliveryUnknown(_)) => {
                    TransportOutcome::DeliveryUnknown(CleanupEvidence::unproven())
                }
            }
        }
        fn terminate_and_probe(&mut self) -> CleanupEvidence {
            CleanupEvidence::unproven()
        }
    }

    fn identity(request: &str) -> Identity {
        Identity {
            request_id: request.into(),
            attempt_id: "a".into(),
            worker_instance_id: "w".into(),
            fence: 7,
        }
    }
    fn hello() -> WorkerResponse {
        WorkerResponse::HelloResult {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("hello"),
            status: Status::Ok,
            error_class: ErrorClass::None,
            source_revision: SOURCE_REVISION.into(),
            bridge_contract_sha256: "b".into(),
            sample_definitions_sha256: "s".into(),
            action_allowlist_sha256: "a".into(),
            capabilities_sha256: capabilities_sha256(),
        }
    }
    fn request(deadline: u64) -> HostRequest {
        HostRequest::Dispatch {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("dispatch"),
            input_generation: 3,
            deadline_mono_ms: deadline,
            sample_id: "CANCEL-02".into(),
            action_id: ActionId::ActionClickQualifiedRef,
            snapshot_id: "snapshot".into(),
            qualified_ref: "@snapshot:e1".into(),
        }
    }
    fn snapshot_request(deadline: u64) -> HostRequest {
        HostRequest::Snapshot {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("snapshot"),
            input_generation: 3,
            deadline_mono_ms: deadline,
            sample_id: "CANCEL-02".into(),
            target_identity: TargetIdentity::fixture("fixture", "O-WORKER-CONTAINMENT-V1"),
        }
    }
    fn snapshot_response() -> WorkerResponse {
        WorkerResponse::SnapshotResult {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("snapshot"),
            status: Status::Ok,
            error_class: ErrorClass::None,
            snapshot_id: "snapshot".into(),
            qualified_refs: vec!["@snapshot:e1".into()],
            target_identity_verdict: IdentityVerdict::Match,
        }
    }
    fn observe_request(deadline: u64) -> HostRequest {
        HostRequest::Observe {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("observe"),
            input_generation: 3,
            deadline_mono_ms: deadline,
            sample_id: "CANCEL-02".into(),
            predicate_id: "P-INFLIGHT-UNKNOWN-V1".into(),
            oracle_id: "O-WORKER-CONTAINMENT-V1".into(),
            prior_snapshot_id: "snapshot".into(),
            target_identity: TargetIdentity::fixture("fixture", "O-WORKER-CONTAINMENT-V1"),
        }
    }
    fn dispatch_response(status: Status, disposition: DeliveryDisposition) -> WorkerResponse {
        WorkerResponse::DispatchResult {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("dispatch"),
            status,
            error_class: if status == Status::Ok {
                ErrorClass::None
            } else {
                ErrorClass::Driver
            },
            delivery_disposition: disposition,
            driver_status: "mock".into(),
            next_snapshot_hint: None,
        }
    }
    fn observe_response() -> WorkerResponse {
        WorkerResponse::ObserveResult {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("observe"),
            status: Status::Ok,
            error_class: ErrorClass::None,
            snapshot_id: "fresh".into(),
            target_identity_verdict: IdentityVerdict::Match,
            predicate_verdict: PredicateVerdict::Satisfied,
            element_count: 1,
        }
    }
    fn state() -> AdmissionSnapshot {
        AdmissionSnapshot {
            fence: 7,
            input_generation: 3,
            observer_ready: true,
            control: ControlState::Running,
        }
    }
    fn ready(safety: HostSafetyState) -> Supervisor {
        let mut supervisor = Supervisor::new(state(), safety);
        assert!(supervisor.accept_hello(&hello(), &identity("hello"), &["b", "s", "a"]));
        assert!(supervisor.accept_snapshot(&snapshot_request(100), &snapshot_response(), 1));
        supervisor
    }

    #[test]
    fn rejected_snapshot_never_admits_dispatch() {
        let mut supervisor = Supervisor::new(state(), HostSafetyState::default());
        assert!(supervisor.accept_hello(&hello(), &identity("hello"), &["b", "s", "a"]));
        let rejected = WorkerResponse::SnapshotResult {
            schema_version: BRIDGE_SCHEMA_VERSION.into(),
            identity: identity("snapshot"),
            status: Status::Rejected,
            error_class: ErrorClass::Unsupported,
            snapshot_id: String::new(),
            qualified_refs: Vec::new(),
            target_identity_verdict: IdentityVerdict::Unavailable,
        };
        assert!(!supervisor.accept_snapshot(&snapshot_request(100), &rejected, 1));
        assert_eq!(
            supervisor.dispatch(&state(), &request(100), &mut TestTransport(Vec::new()), 2),
            DispatchResult::NotDispatched
        );
    }

    struct StableOracle(u64);
    impl ReadOnlyOracle for StableOracle {
        fn sample(&mut self, _target: &TargetIdentity, _predicate_id: &str) -> OracleSample {
            let result = OracleSample {
                identity_match: true,
                predicate_satisfied: true,
                monotonic_ms: self.0,
            };
            self.0 += 250;
            result
        }
    }

    struct FlushFailure {
        wrote: bool,
    }
    impl Write for FlushFailure {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.wrote = !bytes.is_empty();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("flush"))
        }
    }

    #[test]
    fn cancel_and_flush_failure_are_linearized_conservatively() {
        let mut before = Supervisor::new(state(), HostSafetyState::default());
        assert_eq!(
            before.cancel(&mut TestTransport(Vec::new())),
            DispatchResult::NotDispatched
        );
        let mut after = ready(HostSafetyState::default());
        assert_eq!(
            after.dispatch(&state(), &request(100), &mut TestTransport(Vec::new()), 1),
            DispatchResult::InFlight
        );
        assert_eq!(
            after.cancel(&mut TestTransport(Vec::new())),
            DispatchResult::Unknown
        );
        assert_eq!(
            after.response(
                &dispatch_response(Status::Ok, DeliveryDisposition::DeliveredVerified),
                2
            ),
            DispatchResult::Unknown
        );
        let mut failed = ready(HostSafetyState::default());
        let mut writer = TestTransport(FlushFailure { wrote: false });
        assert_eq!(
            failed.dispatch(&state(), &request(100), &mut writer, 1),
            DispatchResult::Unknown
        );
        assert!(writer.0.wrote);
        assert_eq!(failed.boundary(), DispatchBoundary::FrameFlushed);
    }

    #[test]
    fn no_session_and_before_write_failures_are_terminal() {
        struct RejectTransport;
        impl DispatchTransport for RejectTransport {
            fn send_dispatch(
                &mut self,
                _request: &HostRequest,
                _deadline: u64,
            ) -> TransportOutcome {
                TransportOutcome::BeforeWrite
            }
            fn terminate_and_probe(&mut self) -> CleanupEvidence {
                CleanupEvidence::unproven()
            }
        }
        let mut no_session = Supervisor::new(state(), HostSafetyState::default());
        assert_eq!(
            no_session.dispatch(&state(), &request(100), &mut RejectTransport, 1),
            DispatchResult::NotDispatched
        );
        assert!(!no_session.accept_hello(&hello(), &identity("hello"), &["b", "s", "a"]));

        let mut before_write = ready(HostSafetyState::default());
        assert_eq!(
            before_write.dispatch(&state(), &request(100), &mut RejectTransport, 1),
            DispatchResult::NotDispatched
        );
        assert_eq!(
            before_write.dispatch(&state(), &request(100), &mut RejectTransport, 1),
            DispatchResult::NotDispatched
        );
    }

    #[test]
    fn only_valid_dispatch_and_observe_reach_observed() {
        for (status, disposition) in [
            (Status::Rejected, DeliveryDisposition::NotDelivered),
            (Status::Ok, DeliveryDisposition::DeliveredUnverified),
        ] {
            let mut supervisor = ready(HostSafetyState::default());
            assert_eq!(
                supervisor.dispatch(&state(), &request(100), &mut TestTransport(Vec::new()), 1),
                DispatchResult::InFlight
            );
            assert_eq!(
                supervisor.response(&dispatch_response(status, disposition), 2),
                DispatchResult::Unknown
            );
        }
        let mut supervisor = ready(HostSafetyState::default());
        assert_eq!(
            supervisor.dispatch(&state(), &request(100), &mut TestTransport(Vec::new()), 1),
            DispatchResult::InFlight
        );
        assert_eq!(
            supervisor.response(
                &dispatch_response(Status::Ok, DeliveryDisposition::DeliveredVerified),
                2
            ),
            DispatchResult::InFlight
        );
        assert_eq!(
            supervisor.observe(
                &observe_request(100),
                &observe_response(),
                3,
                &mut StableOracle(10)
            ),
            DispatchResult::Observed
        );
    }

    #[test]
    fn worker_verdict_cannot_replace_two_stable_host_samples() {
        struct Unstable;
        impl ReadOnlyOracle for Unstable {
            fn sample(&mut self, _target: &TargetIdentity, _predicate: &str) -> OracleSample {
                OracleSample {
                    identity_match: true,
                    predicate_satisfied: true,
                    monotonic_ms: 10,
                }
            }
        }
        let mut supervisor = ready(HostSafetyState::default());
        assert_eq!(
            supervisor.dispatch(&state(), &request(100), &mut TestTransport(Vec::new()), 1),
            DispatchResult::InFlight
        );
        assert_eq!(
            supervisor.response(
                &dispatch_response(Status::Ok, DeliveryDisposition::DeliveredVerified),
                2
            ),
            DispatchResult::InFlight
        );
        assert_eq!(
            supervisor.observe(&observe_request(100), &observe_response(), 3, &mut Unstable),
            DispatchResult::Unknown
        );
    }

    #[test]
    fn worker_observe_mismatch_or_unsatisfied_cannot_be_observed() {
        for (identity_verdict, predicate_verdict) in [
            (IdentityVerdict::Mismatch, PredicateVerdict::Satisfied),
            (IdentityVerdict::Match, PredicateVerdict::Unsatisfied),
        ] {
            let mut supervisor = ready(HostSafetyState::default());
            assert_eq!(
                supervisor.dispatch(&state(), &request(100), &mut TestTransport(Vec::new()), 1),
                DispatchResult::InFlight
            );
            assert_eq!(
                supervisor.response(
                    &dispatch_response(Status::Ok, DeliveryDisposition::DeliveredVerified),
                    2
                ),
                DispatchResult::InFlight
            );
            let mut response = observe_response();
            if let WorkerResponse::ObserveResult {
                target_identity_verdict,
                predicate_verdict: predicate,
                ..
            } = &mut response
            {
                *target_identity_verdict = identity_verdict;
                *predicate = predicate_verdict;
            }
            assert_eq!(
                supervisor.observe(&observe_request(100), &response, 3, &mut StableOracle(10)),
                DispatchResult::Unknown
            );
        }
    }

    #[test]
    fn identity_generation_observer_and_deadline_fail_closed() {
        for changed in [
            AdmissionSnapshot {
                input_generation: 4,
                ..state()
            },
            AdmissionSnapshot {
                input_generation: 2,
                ..state()
            },
            AdmissionSnapshot {
                observer_ready: false,
                ..state()
            },
        ] {
            let mut supervisor = ready(HostSafetyState::default());
            assert_eq!(
                supervisor.dispatch(&changed, &request(100), &mut TestTransport(Vec::new()), 1),
                DispatchResult::NotDispatched
            );
        }
        let mut supervisor = ready(HostSafetyState::default());
        assert_eq!(
            supervisor.dispatch(&state(), &request(5), &mut TestTransport(Vec::new()), 5),
            DispatchResult::NotDispatched
        );
        let mut supervisor = ready(HostSafetyState::default());
        assert_eq!(
            supervisor.dispatch(&state(), &request(5), &mut TestTransport(Vec::new()), 1),
            DispatchResult::InFlight
        );
        assert_eq!(
            supervisor.response(
                &dispatch_response(Status::Ok, DeliveryDisposition::DeliveredVerified),
                5
            ),
            DispatchResult::Unknown
        );
    }

    #[test]
    fn quarantine_is_shared_across_supervisors() {
        let safety = HostSafetyState::default();
        let mut first = ready(safety.clone());
        first.record_termination(CleanupEvidence::unproven());
        let mut next = Supervisor::new(state(), safety);
        assert!(!next.accept_hello(&hello(), &identity("hello"), &["b", "s", "a"]));
        assert!(next.is_quarantined());
    }

    #[test]
    fn worker_restart_cannot_roll_back_host_generation() {
        let safety = HostSafetyState::default();
        let _first = ready(safety.clone());
        let mut rolled_back = Supervisor::new(
            AdmissionSnapshot {
                input_generation: 2,
                ..state()
            },
            safety,
        );
        assert!(!rolled_back.accept_hello(&hello(), &identity("hello"), &["b", "s", "a"]));
    }

    #[test]
    fn admission_update_cannot_cross_compare_and_flush_boundary() {
        use std::sync::mpsc;
        use std::thread;
        struct BlockingWriter {
            entered: mpsc::Sender<()>,
            release: mpsc::Receiver<()>,
        }
        impl Write for BlockingWriter {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                self.entered.send(()).map_err(io::Error::other)?;
                self.release.recv().map_err(io::Error::other)?;
                Ok(())
            }
        }
        let mut supervisor = ready(HostSafetyState::default());
        let gate = supervisor.admission_gate();
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let dispatch = thread::spawn(move || {
            let result = supervisor.dispatch(
                &state(),
                &request(100),
                &mut TestTransport(BlockingWriter {
                    entered: entered_tx,
                    release: release_rx,
                }),
                1,
            );
            (supervisor, result)
        });
        let Ok(()) = entered_rx.recv() else {
            panic!("dispatch 未进入 flush")
        };
        let (updated_tx, updated_rx) = mpsc::channel();
        let updater = thread::spawn(move || {
            gate.update(|s| s.input_generation += 1);
            assert!(updated_tx.send(()).is_ok());
        });
        assert!(
            updated_rx.try_recv().is_err(),
            "准入更新不得穿越 compare+flush 临界区"
        );
        assert!(release_tx.send(()).is_ok());
        let Ok((_, result)) = dispatch.join() else {
            panic!("dispatch thread panic")
        };
        assert!(updater.join().is_ok());
        assert_eq!(result, DispatchResult::InFlight);
        assert!(updated_rx.recv().is_ok());
    }

    #[test]
    fn deadline_writer_kills_before_returning_delivery_unknown() {
        use std::sync::mpsc;
        struct NeverFlush(mpsc::Receiver<()>);
        impl Write for NeverFlush {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.0.recv().map_err(io::Error::other)?;
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let (release_tx, release_rx) = mpsc::channel();
        let mut killed = false;
        let outcome = deadline_write(
            &mut NeverFlush(release_rx),
            &request(100),
            Duration::from_millis(10),
            || {
                killed = true;
                assert!(release_tx.send(()).is_ok());
                CleanupEvidence::unproven()
            },
        );
        assert!(killed);
        assert!(matches!(outcome, TransportOutcome::DeliveryUnknown(_)));
    }

    #[test]
    fn delivery_unknown_cleanup_is_applied_before_dispatch_returns() {
        struct TimeoutTransport;
        impl DispatchTransport for TimeoutTransport {
            fn send_dispatch(
                &mut self,
                _request: &HostRequest,
                _deadline: u64,
            ) -> TransportOutcome {
                TransportOutcome::DeliveryUnknown(CleanupEvidence::unproven())
            }
            fn terminate_and_probe(&mut self) -> CleanupEvidence {
                CleanupEvidence::unproven()
            }
        }
        let safety = HostSafetyState::default();
        let mut supervisor = ready(safety.clone());
        assert_eq!(
            supervisor.dispatch(&state(), &request(100), &mut TimeoutTransport, 1),
            DispatchResult::Unknown
        );
        assert!(supervisor.is_quarantined());
        assert!(safety.is_quarantined());
    }
}
