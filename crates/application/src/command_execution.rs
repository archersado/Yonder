//! 受本机一次性批准约束的 Agent Command 执行用例。
use crate::{
    AttemptConclusion, AttemptPhase, AttemptResultRecord, AuthContext, Error, ExecutionAttempt,
    Task, TaskStore,
    admission::{Admission, start_execution},
    command::{self, CommandCancellation, CommandError, CommandExecution, CommandOutcome, CommandPort},
    command_approval::{CommandApprovalError, CommandApprovalRegistry},
    computer_use::UnknownReason,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentCommandExecution {
    pub task: Task,
    pub attempt_result: AttemptResultRecord,
    pub execution: CommandExecution,
}

/// 先提交 TM-S7 启动事实，再消费批准并派发一次；任何结果都不会自动重试。
pub fn execute_agent_command(
    store: &mut impl TaskStore,
    admission: &Admission,
    approvals: &CommandApprovalRegistry,
    port: &dyn CommandPort,
    cancellation: &dyn CommandCancellation,
    auth: AuthContext<'_>,
    task_id: &str,
    expected_sequence: u64,
    command_id: &str,
    host_session_id: &str,
    now_ms: u64,
) -> Result<AgentCommandExecution, Error> {
    if !crate::valid_id(task_id) || !crate::valid_id(command_id) || !crate::valid_id(host_session_id) {
        return Err(Error::InvalidInput);
    }
    let (task, step) = crate::get_with_step(store, auth, task_id)?;
    let step = step.ok_or(Error::StopRequired)?;
    if task.sequence != expected_sequence { return Err(Error::Conflict); }
    let attempt = ExecutionAttempt {
        task_id: task_id.into(),
        step_id: step.step_id,
        attempt_id: format!("attempt_{}", expected_sequence + 1),
        worker_instance_id: "command_worker".into(),
        host_session_id: host_session_id.into(),
        phase: AttemptPhase::Prepared,
        accepted_sequence: 0,
    };
    let accepted = start_execution(store, admission, &task, &attempt, expected_sequence, &[])
        .map_err(|_| Error::StopRequired)?.1;
    let request = match approvals.consume(auth, &task, expected_sequence, command_id, now_ms) {
        Ok(request) => request,
        Err(error) => {
            record_rejected_after_start(store, &accepted, task_id)?;
            return Err(approval_error(error));
        }
    };
    let execution = match command::execute(port, &request, cancellation) {
        Ok(execution) => execution,
        Err(error) => {
            record_rejected_after_start(store, &accepted, task_id)?;
            return Err(command_error(error));
        }
    };
    let conclusion = match execution.outcome {
        CommandOutcome::Unknown => AttemptConclusion::Unknown { reason: UnknownReason::WorkerFailed },
        CommandOutcome::Exited { exit_code: 0 } => AttemptConclusion::Observed { action_succeeded: true },
        CommandOutcome::Exited { .. } | CommandOutcome::TimedOut | CommandOutcome::Cancelled | CommandOutcome::OutputLimitExceeded =>
            AttemptConclusion::Observed { action_succeeded: false },
    };
    let current = store.get(task_id)?;
    let (task, attempt_result) = store.record_attempt_result(&accepted, current.sequence, conclusion)
        .map_err(|_| Error::StorageUnavailable)?;
    Ok(AgentCommandExecution { task, attempt_result, execution })
}

fn record_rejected_after_start(store: &mut impl TaskStore, accepted: &ExecutionAttempt, task_id: &str) -> Result<(), Error> {
    let current = store.get(task_id)?;
    store.record_attempt_result(accepted, current.sequence, AttemptConclusion::Observed { action_succeeded: false })
        .map(|_| ()).map_err(|_| Error::StorageUnavailable)
}

fn approval_error(error: CommandApprovalError) -> Error {
    match error {
        CommandApprovalError::InvalidInput => Error::InvalidInput,
        CommandApprovalError::PermissionDenied => Error::PermissionDenied,
        CommandApprovalError::Unavailable => Error::StorageUnavailable,
        CommandApprovalError::NotFound | CommandApprovalError::Expired | CommandApprovalError::Rejected | CommandApprovalError::Capacity => Error::StopRequired,
    }
}

fn command_error(error: CommandError) -> Error {
    match error {
        CommandError::InvalidInput => Error::InvalidInput,
        CommandError::UnsupportedPlatform | CommandError::StartFailed => Error::StopRequired,
    }
}
