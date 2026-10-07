use crate::{AttemptResultRecord, AuthContext, Error, ExecutionAttempt, Status, Task, TaskStore, admission::{Admission, Resource, start_execution}};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkTarget { pub pid: u32, pub window_id: u32 }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputerAction { pub tool_name:String, pub arguments_json:String }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputerObservation {
    pub element_count:u16,
    pub screenshot_path:Option<String>,
    pub screenshot_mime:Option<String>,
    pub target_visible:Option<bool>,
    pub observation_ref:Option<String>,
    pub transcript:Option<String>,
    pub elements:Vec<yonder_protocol::ObservedElement>,
}

/// 执行未知结果的内部有界分类；wire 表达是 protocol 的单一来源枚举，
/// 经本模块的 `From` 实现映射，不得在 Gateway/Query 内手写第二份映射。
pub type UnknownReason = crate::unknown_reason::UnknownReason;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchOutcome {
    Known { action_succeeded: bool, observation:Option<ComputerObservation> },
    Unknown(UnknownReason),
    UnknownObserved { reason: UnknownReason, observation: ComputerObservation },
}

fn outcome_observation(outcome: &DispatchOutcome) -> Option<ComputerObservation> {
    match outcome {
        DispatchOutcome::Known { observation, .. } => observation.clone(),
        DispatchOutcome::UnknownObserved { observation, .. } => Some(observation.clone()),
        DispatchOutcome::Unknown(_) => None,
    }
}

/// 只供可信 Application 编排；Agent/UI 不得直接构造目标或上报执行结果。
pub trait ComputerUsePort {
    fn dispatch(&self, attempt: &ExecutionAttempt, target: &WorkTarget, action: &ComputerAction) -> DispatchOutcome;
    fn end_session(&self) {}
    fn explicit_takeover_requested(&self, _task_id: &str) -> bool { false }
    fn project_decision(&self, _task_id: &str, _step_id: &str, _step_label: &str, _summary: &str) {}
    fn project_step_completed(&self, _task_id: &str, _step_id: &str) {}
    fn project_step_unverified(&self, _task_id: &str, _step_id: &str) {}
}

pub trait WorkTargetPort { fn frontmost(&self) -> Result<WorkTarget, UnknownReason>; }

/// 从任务事实源取得完整尝试身份；调用方不能用请求字段替代已准备的 attempt。
pub fn dispatch_prepared(store: &mut impl crate::TaskStore, port: &(impl ComputerUsePort + ?Sized), task_id: &str, attempt_id: &str, target: &WorkTarget, action: &ComputerAction) -> Result<DispatchOutcome, crate::Error> {
    if !crate::valid_id(task_id) || !crate::valid_id(attempt_id) { return Err(crate::Error::InvalidInput); }
    if store.get_control(task_id)?.is_some_and(|control| control.phase == crate::ControlPhase::Pending) { return Err(crate::Error::StopRequired); }
    let attempt = store.get_attempt(task_id)?.filter(|attempt| attempt.attempt_id == attempt_id && attempt.phase == crate::AttemptPhase::Prepared).ok_or(crate::Error::Conflict)?;
    Ok(port.dispatch(&attempt, target, action))
}

pub fn execute_agent_action(
    store:&mut impl TaskStore, admission:&Admission, port:&(impl ComputerUsePort + ?Sized), targets:&(impl WorkTargetPort + ?Sized),
    auth:AuthContext<'_>, task_id:&str, expected:u64, tool_name:&str, arguments_json:&str, host_session_id:&str,
) -> Result<(Task,AttemptResultRecord,Option<ComputerObservation>),Error> {
    if tool_name.is_empty()||tool_name.len()>64||arguments_json.is_empty()||arguments_json.len()>16*1024||arguments_json.contains('\0')||!crate::valid_id(host_session_id){return Err(Error::InvalidInput);}
    if port.explicit_takeover_requested(task_id){return Err(Error::StopRequired);}
    let (task,step)=crate::get_with_step(store,auth,task_id)?; let step=step.ok_or(Error::StopRequired)?;
    if task.sequence!=expected{return Err(Error::Conflict);}
    let target=targets.frontmost().map_err(|_|Error::StopRequired)?;
    let attempt=ExecutionAttempt{task_id:task_id.into(),step_id:step.step_id,attempt_id:format!("attempt_{}",expected+1),worker_instance_id:"cua_worker".into(),host_session_id:host_session_id.into(),phase:crate::AttemptPhase::Prepared,accepted_sequence:0};
    let accepted=start_execution(store,admission,&task,&attempt,expected,&[Resource::Desktop]).map_err(|_|Error::StopRequired)?.1;
    let action=ComputerAction{tool_name:tool_name.into(),arguments_json:arguments_json.into()};
    let outcome=dispatch_prepared(store,port,task_id,&accepted.attempt_id,&target,&action)?;
    let observation=outcome_observation(&outcome);
    let interrupted=outcome==DispatchOutcome::Unknown(UnknownReason::UserInput);
    let (result_task,result)=record_dispatch_outcome(store,task_id,&accepted.attempt_id,outcome)?;
    if interrupted{
        let task=crate::transition(store,task_id,result_task.sequence,crate::Action::Interrupt)?;
        admission.release_task_after_stop(task_id).map_err(|_|Error::StorageUnavailable)?;
        return Ok((task,result,observation));
    }
    Ok((result_task,result,observation))
}

/// TM-S9 正式热路径：SQLite 只用于副作用前的授权/步骤快照；Driver 返回后先
/// 进入内存 Runtime，持久化由独立 projector 完成。
pub fn execute_agent_action_runtime(
    store: &mut impl TaskStore,
    runtime: &crate::execution_runtime::ExecutionRuntimeHandle,
    admission: &Admission,
    port: &(impl ComputerUsePort + ?Sized),
    targets: &(impl WorkTargetPort + ?Sized),
    auth: AuthContext<'_>,
    task_id: &str,
    expected: u64,
    tool_name: &str,
    arguments_json: &str,
    host_session_id: &str,
) -> Result<(Task, AttemptResultRecord, Option<ComputerObservation>), Error> {
    if tool_name.is_empty()
        || tool_name.len() > 64
        || arguments_json.is_empty()
        || arguments_json.len() > 16 * 1024
        || arguments_json.contains('\0')
        || !crate::valid_id(host_session_id)
    {
        return Err(Error::InvalidInput);
    }
    if port.explicit_takeover_requested(task_id) {
        return Err(Error::StopRequired);
    }
    let (task, step) = crate::get_with_step(store, auth, task_id)?;
    let step = step.ok_or(Error::StopRequired)?;
    if store
        .get_control(task_id)?
        .is_some_and(|control| control.phase == crate::ControlPhase::Pending)
    {
        return Err(Error::StopRequired);
    }
    let target = targets.frontmost().map_err(|_| Error::StopRequired)?;
    let attempt = crate::execution_runtime::begin_attempt(
        runtime,
        admission,
        &task,
        &step,
        expected,
        "cua_worker",
        host_session_id,
        &[Resource::Desktop],
    )
    .map_err(runtime_error)?;
    let action = ComputerAction {
        tool_name: tool_name.into(),
        arguments_json: arguments_json.into(),
    };
    let outcome = port.dispatch(&attempt, &target, &action);
    let observation = outcome_observation(&outcome);
    let interrupted = outcome == DispatchOutcome::Unknown(UnknownReason::UserInput);
    let conclusion = match outcome {
        DispatchOutcome::Known {
            action_succeeded, ..
        } => crate::AttemptConclusion::Observed { action_succeeded },
        DispatchOutcome::Unknown(reason) | DispatchOutcome::UnknownObserved { reason, .. } => crate::AttemptConclusion::Unknown { reason },
    };
    let (snapshot, result) =
        crate::execution_runtime::record_attempt_outcome(runtime, &attempt, conclusion)
            .map_err(runtime_error)?;
    let mut result_task = Task {
        status: Status::Running,
        sequence: snapshot.sequence,
        ..task
    };
    if interrupted {
        let stopped = runtime
            .apply(crate::execution_runtime::RuntimeCommand::Terminate {
                task_id: task_id.into(),
                phase: crate::execution_runtime::RuntimePhase::Interrupted,
            })
            .map_err(runtime_error)?;
        result_task.status = Status::Interrupted;
        result_task.sequence = stopped.sequence;
        admission
            .release_task_after_stop(task_id)
            .map_err(|_| Error::StorageUnavailable)?;
    }
    Ok((result_task, result, observation))
}

pub fn execute_agent_step_runtime(
    store: &mut impl TaskStore,
    runtime: &crate::execution_runtime::ExecutionRuntimeHandle,
    admission: &Admission,
    port: &(impl ComputerUsePort + ?Sized),
    targets: &(impl WorkTargetPort + ?Sized),
    auth: AuthContext<'_>,
    task_id: &str,
    expected: u64,
    step_id: &str,
    label: &str,
    tool_name: &str,
    arguments_json: &str,
    host_session_id: &str,
) -> Result<(Task, AttemptResultRecord, Option<ComputerObservation>), Error> {
    if !crate::valid_id(step_id)
        || label.is_empty()
        || label.len() > 120
        || tool_name.is_empty()
        || tool_name.len() > 64
        || arguments_json.is_empty()
        || arguments_json.len() > 16 * 1024
        || arguments_json.contains('\0')
        || !crate::valid_id(host_session_id)
    {
        return Err(Error::InvalidInput);
    }
    if port.explicit_takeover_requested(task_id) {
        return Err(Error::StopRequired);
    }
    let task = crate::get(store, task_id)?;
    if !auth.can_read(&task) {
        return Err(Error::NotFound);
    }
    if store
        .get_control(task_id)?
        .is_some_and(|control| control.phase == crate::ControlPhase::Pending)
    {
        return Err(Error::StopRequired);
    }
    let step = crate::StepDeclaration {
        step_id: step_id.into(),
        label: label.into(),
        accepted_sequence: expected,
    };
    let target = targets.frontmost().map_err(|_| Error::StopRequired)?;
    let attempt = crate::execution_runtime::begin_attempt(
        runtime,
        admission,
        &task,
        &step,
        expected,
        "cua_worker",
        host_session_id,
        &[Resource::Desktop],
    )
    .map_err(runtime_error)?;
    let outcome = port.dispatch(
        &attempt,
        &target,
        &ComputerAction {
            tool_name: tool_name.into(),
            arguments_json: arguments_json.into(),
        },
    );
    let observation = outcome_observation(&outcome);
    let interrupted = outcome == DispatchOutcome::Unknown(UnknownReason::UserInput);
    let conclusion = match outcome {
        DispatchOutcome::Known {
            action_succeeded, ..
        } => crate::AttemptConclusion::Observed { action_succeeded },
        DispatchOutcome::Unknown(reason) | DispatchOutcome::UnknownObserved { reason, .. } => crate::AttemptConclusion::Unknown { reason },
    };
    let (snapshot, result) =
        crate::execution_runtime::record_attempt_outcome(runtime, &attempt, conclusion)
            .map_err(runtime_error)?;
    let (status, sequence) = if interrupted {
        let stopped = runtime
            .apply(crate::execution_runtime::RuntimeCommand::Terminate {
                task_id: task_id.into(),
                phase: crate::execution_runtime::RuntimePhase::Interrupted,
            })
            .map_err(runtime_error)?;
        admission
            .release_task_after_stop(task_id)
            .map_err(|_| Error::StorageUnavailable)?;
        (Status::Interrupted, stopped.sequence)
    } else if let crate::AttemptConclusion::Observed { action_succeeded } = result.conclusion {
        if action_succeeded {
            port.project_step_completed(task_id, step_id);
        } else {
            port.project_step_unverified(task_id, step_id);
        }
        let boundary = runtime
            .apply(crate::execution_runtime::RuntimeCommand::AdvanceStepBoundary {
                task_id: task_id.into(),
                step_id: step_id.into(),
            })
            .map_err(runtime_error)?;
        (Status::Running, boundary.sequence)
    } else {
        port.project_step_unverified(task_id, step_id);
        (Status::Running, snapshot.sequence)
    };
    Ok((
        Task {
            status,
            sequence,
            ..task
        },
        result,
        observation,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation() -> ComputerObservation {
        ComputerObservation {
            element_count: 0,
            screenshot_path: Some("/private/visual-fallback.png".into()),
            screenshot_mime: Some("image/png".into()),
            target_visible: Some(true),
            observation_ref: None,
            transcript: None,
            elements: Vec::new(),
        }
    }

    #[test]
    fn unknown_observed_keeps_visual_evidence_for_replan() {
        let expected = observation();
        assert_eq!(
            outcome_observation(&DispatchOutcome::UnknownObserved {
                reason: UnknownReason::ObserveFailed,
                observation: expected.clone(),
            }),
            Some(expected)
        );
    }

    #[test]
    fn unknown_without_observe_has_no_visual_evidence() {
        assert_eq!(
            outcome_observation(&DispatchOutcome::Unknown(UnknownReason::TimedOut)),
            None
        );
    }
}

fn runtime_error(error: crate::execution_runtime::RuntimeError) -> Error {
    match error {
        crate::execution_runtime::RuntimeError::InvalidInput => Error::InvalidInput,
        crate::execution_runtime::RuntimeError::Conflict => Error::Conflict,
        crate::execution_runtime::RuntimeError::Backpressure => Error::StopRequired,
        crate::execution_runtime::RuntimeError::NotFound => Error::NotFound,
        crate::execution_runtime::RuntimeError::Unavailable => Error::StorageUnavailable,
    }
}

pub fn execute_agent_step(
    store:&mut impl TaskStore,admission:&Admission,port:&(impl ComputerUsePort + ?Sized),targets:&(impl WorkTargetPort + ?Sized),auth:AuthContext<'_>,
    task_id:&str,expected:u64,step_id:&str,label:&str,tool_name:&str,arguments_json:&str,host_session_id:&str,
)->Result<(Task,AttemptResultRecord,Option<ComputerObservation>),Error>{
    if port.explicit_takeover_requested(task_id){return Err(Error::StopRequired);}
    let (declared,_)=crate::declare_step(store,auth,task_id,expected,step_id,label)?;
    let (task,result,observation)=execute_agent_action(store,admission,port,targets,auth,task_id,declared.sequence,tool_name,arguments_json,host_session_id)?;
    if !matches!(&result.conclusion,crate::AttemptConclusion::Observed{..}){return Ok((task,result,observation));}
    let (advanced,_)=crate::advance_after_observe(store,task_id,&result.attempt_id)?;
    Ok((advanced,result,observation))
}

pub fn complete_agent_task(store:&mut impl TaskStore,admission:&Admission,auth:AuthContext<'_>,task_id:&str,expected:u64)->Result<Task,Error>{
    finish_agent_task(store,admission,auth,task_id,expected,false,None)
}

pub fn verify_agent_goal(
    store: &mut impl TaskStore,
    auth: AuthContext<'_>,
    task_id: &str,
    expected: u64,
    observation_sequence: u64,
    verification_id: &str,
    outcome: crate::GoalVerificationOutcome,
) -> Result<(Task, crate::GoalVerificationRecord), Error> {
    let task = crate::get(store, task_id)?;
    if !auth.can_read(&task) { return Err(Error::NotFound); }
    if task.sequence != expected || observation_sequence > expected || task.status != Status::Running {
        return Err(Error::Conflict);
    }
    store.record_goal_verification(task_id, expected, observation_sequence, verification_id, outcome)
}

pub fn verify_agent_goal_for_owner(
    store: &mut impl TaskStore,
    runtime: &crate::execution_runtime::ExecutionRuntimeHandle,
    auth: AuthContext<'_>,
    task_id: &str,
    expected: u64,
    observation_sequence: u64,
    verification_id: &str,
    outcome: crate::GoalVerificationOutcome,
) -> Result<(Task, crate::GoalVerificationRecord), Error> {
    let task = crate::get(store, task_id)?;
    if !auth.can_read(&task) { return Err(Error::NotFound); }
    match runtime.snapshot(task_id) {
        Ok(snapshot) => {
            if snapshot.sequence != expected || snapshot.last_observation_sequence != Some(observation_sequence) {
                return Err(Error::Conflict);
            }
            let next = runtime.apply(crate::execution_runtime::RuntimeCommand::VerifyGoal {
                task_id: task_id.into(), verification_id: verification_id.into(), observation_sequence, outcome,
            }).map_err(runtime_error)?;
            let verification = next.goal_verification.clone().ok_or(Error::StorageUnavailable)?;
            Ok((Task { sequence: next.sequence, status: Status::Running, ..task }, verification))
        }
        Err(crate::execution_runtime::RuntimeError::NotFound) => verify_agent_goal(store, auth, task_id, expected, observation_sequence, verification_id, outcome),
        Err(error) => Err(runtime_error(error)),
    }
}

pub fn complete_agent_task_verified(store:&mut impl TaskStore,admission:&Admission,auth:AuthContext<'_>,task_id:&str,expected:u64,verification_id:&str)->Result<Task,Error>{
    finish_agent_task(store,admission,auth,task_id,expected,false,Some(verification_id))
}

pub fn fail_agent_task(store:&mut impl TaskStore,admission:&Admission,auth:AuthContext<'_>,task_id:&str,expected:u64)->Result<Task,Error>{
    finish_agent_task(store,admission,auth,task_id,expected,true,None)
}

pub fn finish_agent_task_runtime(
    store: &mut impl TaskStore,
    runtime: &crate::execution_runtime::ExecutionRuntimeHandle,
    admission: &Admission,
    auth: AuthContext<'_>,
    task_id: &str,
    expected: u64,
    failed: bool,
    verification_id: Option<&str>,
) -> Result<Task, Error> {
    let task = crate::get(store, task_id)?;
    if !auth.can_read(&task) {
        return Err(Error::NotFound);
    }
    let snapshot = runtime.snapshot(task_id).map_err(runtime_error)?;
    if snapshot.sequence != expected
        || snapshot.phase != crate::execution_runtime::RuntimePhase::Running
        || snapshot.current_step.is_some()
        || snapshot.last_step_succeeded != Some(!failed)
    {
        return Err(Error::StopRequired);
    }
    if !failed && !snapshot.goal_verification.as_ref().is_some_and(|verification| {
        verification.verification_id == verification_id.unwrap_or_default()
            && verification.verified_sequence == snapshot.sequence
            && verification.outcome == crate::GoalVerificationOutcome::Achieved
    }) { return Err(Error::StopRequired); }
    let phase = if failed {
        crate::execution_runtime::RuntimePhase::Failed
    } else {
        crate::execution_runtime::RuntimePhase::Completed
    };
    let terminal = runtime
        .apply(crate::execution_runtime::RuntimeCommand::Terminate {
            task_id: task_id.into(),
            phase,
        })
        .map_err(runtime_error)?;
    admission
        .release_task_after_stop(task_id)
        .map_err(|_| Error::StorageUnavailable)?;
    Ok(Task {
        status: if failed { Status::Failed } else { Status::Completed },
        sequence: terminal.sequence,
        ..task
    })
}

/// 只在任务实际由内存Runtime持有时走事件驱动终结。计划片段兼容链尚未登记
/// 到Runtime，不能因为组合根存在Runtime句柄就把真实任务误报成不存在；只有
/// `NotFound`允许回到原有Observed/步骤边界/桌面租约三重门禁，其他错误失败关闭。
pub fn finish_agent_task_for_owner_verified(
    store: &mut impl TaskStore,
    runtime: &crate::execution_runtime::ExecutionRuntimeHandle,
    admission: &Admission,
    auth: AuthContext<'_>,
    task_id: &str,
    expected: u64,
    failed: bool,
    verification_id: Option<&str>,
) -> Result<Task, Error> {
    match runtime.snapshot(task_id) {
        Ok(_) => finish_agent_task_runtime(
            store, runtime, admission, auth, task_id, expected, failed, verification_id,
        ),
        Err(crate::execution_runtime::RuntimeError::NotFound) => {
            finish_agent_task(store, admission, auth, task_id, expected, failed, verification_id)
        }
        Err(error) => Err(runtime_error(error)),
    }
}

pub fn finish_agent_task_for_owner(
    store: &mut impl TaskStore,
    runtime: &crate::execution_runtime::ExecutionRuntimeHandle,
    admission: &Admission,
    auth: AuthContext<'_>,
    task_id: &str,
    expected: u64,
    failed: bool,
) -> Result<Task, Error> {
    finish_agent_task_for_owner_verified(store, runtime, admission, auth, task_id, expected, failed, None)
}

fn finish_agent_task(store:&mut impl TaskStore,admission:&Admission,auth:AuthContext<'_>,task_id:&str,expected:u64,failed:bool,verification_id:Option<&str>)->Result<Task,Error>{
    let (task,_)=crate::get_with_step(store,auth,task_id)?;
    if task.sequence!=expected{return Err(Error::Conflict);}
    if task.status!=Status::Running||store.get_control(task_id)?.is_some_and(|value|value.phase==crate::ControlPhase::Pending)||!admission.holds(task_id).map_err(|_|Error::StorageUnavailable)?{return Err(Error::StopRequired);}
    if !store.get_attempt(task_id)?.is_some_and(|value|value.phase==crate::AttemptPhase::Stopped){return Err(Error::StopRequired);}
    if !store.get_attempt_result(task_id)?.is_some_and(|value|value.conclusion==crate::AttemptConclusion::Observed{action_succeeded:!failed}){return Err(Error::StopRequired);}
    if !failed && !store.get_goal_verification(task_id)?.is_some_and(|verification| {
        verification.verification_id == verification_id.unwrap_or_default()
            && verification.verified_sequence == expected
            && verification.outcome == crate::GoalVerificationOutcome::Achieved
    }) { return Err(Error::StopRequired); }
    let completed=crate::transition(store,task_id,expected,if failed{crate::Action::Fail}else{crate::Action::Complete})?;
    admission.release_task_after_stop(task_id).map_err(|_|Error::StorageUnavailable)?;
    Ok(completed)
}

/// 只提交本次已持久化attempt的有界分类；Store负责CAS、事件与Outbox原子性。
pub fn record_dispatch_outcome(store: &mut impl crate::TaskStore, task_id: &str, attempt_id: &str, outcome: DispatchOutcome) -> Result<(crate::Task, crate::AttemptResultRecord), crate::Error> {
    if !crate::valid_id(task_id) || !crate::valid_id(attempt_id) { return Err(crate::Error::InvalidInput); }
    let task = store.get(task_id)?;
    let attempt = store.get_attempt(task_id)?.filter(|attempt| attempt.attempt_id == attempt_id).ok_or(crate::Error::Conflict)?;
    let conclusion = match outcome {
        DispatchOutcome::Known { action_succeeded, .. } => crate::AttemptConclusion::Observed { action_succeeded },
        DispatchOutcome::Unknown(reason) | DispatchOutcome::UnknownObserved { reason, .. } => crate::AttemptConclusion::Unknown { reason },
    };
    store.record_attempt_result(&attempt, task.sequence, conclusion)
}
