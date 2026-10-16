use super::*;

pub(super) fn source_name(value: TaskSource) -> &'static str {
    match value {
        TaskSource::LocalAgent => "local-agent",
        TaskSource::CloudAgent => "cloud-agent",
        TaskSource::Legacy => "legacy",
    }
}

pub(super) fn task_source(value: &str) -> Result<TaskSource, Error> {
    match value {
        "local-agent" => Ok(TaskSource::LocalAgent),
        "cloud-agent" => Ok(TaskSource::CloudAgent),
        "legacy" => Ok(TaskSource::Legacy),
        _ => Err(Error::StorageUnavailable),
    }
}

pub(super) fn observation_result_name(value: TaskObservationResult) -> &'static str {
    match value {
        TaskObservationResult::Matched => "matched",
        TaskObservationResult::NotMatched => "not-matched",
        TaskObservationResult::Unknown => "unknown",
    }
}

pub(super) fn observation_result(value: &str) -> Result<TaskObservationResult, Error> {
    match value {
        "matched" => Ok(TaskObservationResult::Matched),
        "not-matched" => Ok(TaskObservationResult::NotMatched),
        "unknown" => Ok(TaskObservationResult::Unknown),
        _ => Err(Error::StorageUnavailable),
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredObservation {
    step_id: String,
    result: String,
    summary: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredSource {
    source: String,
}

pub(super) fn stored_source(payload: &str) -> Result<TaskSource, Error> {
    let value: StoredSource =
        serde_json::from_str(payload).map_err(|_| Error::StorageUnavailable)?;
    task_source(&value.source)
}

pub(super) fn stored_observation(payload: &str) -> Result<TaskObservation, Error> {
    let value: StoredObservation =
        serde_json::from_str(payload).map_err(|_| Error::StorageUnavailable)?;
    if !yonder_application::valid_id(&value.step_id)
        || value.summary.is_empty()
        || value.summary.trim() != value.summary
        || value.summary.len() > 2048
        || value
            .summary
            .chars()
            .any(|character| character.is_control() && character != '\n')
    {
        return Err(Error::StorageUnavailable);
    }
    Ok(TaskObservation {
        step_id: value.step_id,
        result: observation_result(&value.result)?,
        summary: value.summary,
    })
}

pub(super) fn name(status: Status) -> &'static str {
    match status {
        Status::Created => "created",
        Status::Running => "running",
        Status::WaitingForUser => "waiting-for-user",
        Status::Paused => "paused",
        Status::Interrupted => "interrupted",
        Status::Completed => "completed",
        Status::Failed => "failed",
        Status::Cancelled => "cancelled",
    }
}

pub(super) fn status(value: &str) -> Result<Status, Error> {
    match value {
        "created" => Ok(Status::Created),
        "running" => Ok(Status::Running),
        "waiting-for-user" => Ok(Status::WaitingForUser),
        "paused" => Ok(Status::Paused),
        "interrupted" => Ok(Status::Interrupted),
        "completed" => Ok(Status::Completed),
        "failed" => Ok(Status::Failed),
        "cancelled" => Ok(Status::Cancelled),
        _ => Err(Error::StorageUnavailable),
    }
}

/// SQLite 持久化字符串与 wire 形状同源于 protocol 的 kebab-case 派生；
/// reason/parse_reason 只做存储字符串转换，不得偏离 wire 名。
pub(super) fn reason(value: UnknownReason) -> &'static str {
    match value {
        UnknownReason::InvalidInput => "invalid-input",
        UnknownReason::DependencyUnavailable => "dependency-unavailable",
        UnknownReason::WorkerFailed => "worker-failed",
        UnknownReason::TimedOut => "timed-out",
        UnknownReason::InvalidResponse => "invalid-response",
        UnknownReason::IdentityMismatch => "identity-mismatch",
        UnknownReason::ObserveFailed => "observe-failed",
        UnknownReason::UserInput => "user-input",
        UnknownReason::DriverActionTimedOut => "driver-action-timed-out",
        UnknownReason::DriverActionRejected => "driver-action-rejected",
        UnknownReason::DriverActionUnconfirmed => "driver-action-unconfirmed",
        UnknownReason::DriverTargetStale => "driver-target-stale",
        UnknownReason::DriverObservationFailed => "driver-observation-failed",
        UnknownReason::DriverTransportFailed => "driver-transport-failed",
    }
}

pub(super) fn parse_reason(value: &str) -> Result<UnknownReason, Error> {
    match value {
        "invalid-input" => Ok(UnknownReason::InvalidInput),
        "dependency-unavailable" => Ok(UnknownReason::DependencyUnavailable),
        "worker-failed" => Ok(UnknownReason::WorkerFailed),
        "timed-out" => Ok(UnknownReason::TimedOut),
        "invalid-response" => Ok(UnknownReason::InvalidResponse),
        "identity-mismatch" => Ok(UnknownReason::IdentityMismatch),
        "observe-failed" => Ok(UnknownReason::ObserveFailed),
        "user-input" => Ok(UnknownReason::UserInput),
        "driver-action-timed-out" => Ok(UnknownReason::DriverActionTimedOut),
        "driver-action-rejected" => Ok(UnknownReason::DriverActionRejected),
        "driver-action-unconfirmed" => Ok(UnknownReason::DriverActionUnconfirmed),
        "driver-target-stale" => Ok(UnknownReason::DriverTargetStale),
        "driver-observation-failed" => Ok(UnknownReason::DriverObservationFailed),
        "driver-transport-failed" => Ok(UnknownReason::DriverTransportFailed),
        _ => Err(Error::StorageUnavailable),
    }
}

pub(super) fn focus_failure(value: FocusFailure) -> &'static str {
    match value {
        FocusFailure::PermissionUnavailable => "permission-unavailable",
        FocusFailure::ProcessChanged => "process-changed",
        FocusFailure::WindowMissing => "window-missing",
        FocusFailure::MappingNotUnique => "mapping-not-unique",
        FocusFailure::ActivationFailed => "activation-failed",
        FocusFailure::VerificationFailed => "verification-failed",
        FocusFailure::GeometryChanged => "geometry-changed",
        FocusFailure::ReferenceUnavailable => "reference-unavailable",
    }
}
pub(super) fn parse_focus_failure(value: &str) -> Result<FocusFailure, Error> {
    match value {
        "permission-unavailable" => Ok(FocusFailure::PermissionUnavailable),
        "process-changed" => Ok(FocusFailure::ProcessChanged),
        "window-missing" => Ok(FocusFailure::WindowMissing),
        "mapping-not-unique" => Ok(FocusFailure::MappingNotUnique),
        "activation-failed" => Ok(FocusFailure::ActivationFailed),
        "verification-failed" => Ok(FocusFailure::VerificationFailed),
        "geometry-changed" => Ok(FocusFailure::GeometryChanged),
        "reference-unavailable" => Ok(FocusFailure::ReferenceUnavailable),
        _ => Err(Error::StorageUnavailable),
    }
}

pub(super) fn artifact_availability_name(value: ArtifactAvailability) -> &'static str {
    match value {
        ArtifactAvailability::Available => "available",
        ArtifactAvailability::Missing => "missing",
        ArtifactAvailability::Changed => "changed",
        ArtifactAvailability::Unverified => "unverified",
    }
}

pub(super) fn artifact_availability(value: &str) -> Result<ArtifactAvailability, Error> {
    match value {
        "available" => Ok(ArtifactAvailability::Available),
        "missing" => Ok(ArtifactAvailability::Missing),
        "changed" => Ok(ArtifactAvailability::Changed),
        "unverified" => Ok(ArtifactAvailability::Unverified),
        _ => Err(Error::StorageUnavailable),
    }
}

pub(super) fn projected_state(
    current: &str,
    kind: &RuntimeEventKind,
) -> Result<&'static str, RuntimeProjectionError> {
    match kind {
        RuntimeEventKind::Activated if matches!(current, "created" | "running") => Ok("running"),
        RuntimeEventKind::StepStarted { .. }
        | RuntimeEventKind::StepCompleted { .. }
        | RuntimeEventKind::StepUnverified { .. }
        | RuntimeEventKind::StepBoundaryAdvanced { .. }
        | RuntimeEventKind::PlanSlotAdvanced { .. }
        | RuntimeEventKind::BrowserReferenceUpdated { .. }
        | RuntimeEventKind::ArtifactProduced { .. }
        | RuntimeEventKind::HandedBack { .. }
        | RuntimeEventKind::GoalVerified { .. }
            if current == "running" =>
        {
            Ok("running")
        }
        RuntimeEventKind::ControlStopped{control_kind,..} if current=="running"=>if control_kind=="cancel"{Ok("cancelled")}else{Ok("paused")},
        RuntimeEventKind::Terminal { phase, .. } => match phase {
            RuntimePhase::Completed => Ok("completed"),
            RuntimePhase::Failed => Ok("failed"),
            RuntimePhase::Cancelled => Ok("cancelled"),
            RuntimePhase::Interrupted => Ok("interrupted"),
            RuntimePhase::Paused => Ok("paused"),
            _ => Err(RuntimeProjectionError::Conflict),
        },
        _ => Err(RuntimeProjectionError::Conflict),
    }
}
