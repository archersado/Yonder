use crate::{AttemptPhase, ControlKind, ControlPhase, Error, ExecutionAttempt, TaskStore, computer_use::WorkTarget};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkRef {
    pub work_ref_id: String,
    pub task_id: String, pub step_id: String, pub attempt_id: String,
    pub worker_instance_id: String, pub host_session_id: String,
    pub pid: u32, pub window_id: u32,
    pub process_start_seconds: u64, pub process_start_microseconds: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusFailure { PermissionUnavailable, ProcessChanged, WindowMissing, MappingNotUnique, ActivationFailed, VerificationFailed, GeometryChanged, ReferenceUnavailable }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusOutcome { Focused, Refused(FocusFailure) }

pub trait WorkFocusPort {
    fn capture(&mut self, attempt: &ExecutionAttempt, target: &WorkTarget) -> Result<WorkRef, FocusFailure>;
    fn focus(&mut self, reference: &WorkRef) -> FocusOutcome;
    fn release(&mut self, reference: &WorkRef);
}

/// 只能在可信后置Observe已提交后捕获；Agent/UI不能提供目标或引用字段。
pub fn capture_after_observe(port: &mut impl WorkFocusPort, attempt: &ExecutionAttempt, target: &WorkTarget) -> Result<WorkRef, Error> {
    if !matches!(attempt.phase,AttemptPhase::Observed|AttemptPhase::Stopped) || attempt.accepted_sequence == 0 || target.pid == 0 || target.window_id == 0 { return Err(Error::StopRequired); }
    port.capture(attempt,target).map_err(|_| Error::StopRequired)
}

/// 停止事实已提交后发布定位阶段；原生失败保持paused，不自动重试。
pub fn focus_takeover(store: &mut impl TaskStore, port: &mut impl WorkFocusPort, reference: &WorkRef) -> Result<(crate::Task, crate::ControlRequestRecord, FocusOutcome), Error> {
    let control=store.get_control(&reference.task_id)?.filter(|value|value.attempt_id==reference.attempt_id&&value.kind==ControlKind::Takeover&&value.phase==ControlPhase::Stopped).ok_or(Error::StopRequired)?;
    let (_, control) = store.begin_focus(&reference.task_id, &control.control_id)?;
    let outcome = focus_after_takeover(store,port,reference)?;
    let failure = match outcome { FocusOutcome::Focused=>None, FocusOutcome::Refused(value)=>Some(value) };
    let (task, control) = store.finish_focus(&reference.task_id,&control.control_id,failure)?;
    Ok((task,control,outcome))
}

/// 只有当前takeover控制已确认停止，才能派发原生前置；结果不改变任务状态或启动Recording。
pub fn focus_after_takeover(store: &mut impl TaskStore, port: &mut impl WorkFocusPort, reference: &WorkRef) -> Result<FocusOutcome, Error> {
    let attempt = store.get_attempt(&reference.task_id)?.filter(|attempt| attempt.attempt_id == reference.attempt_id).ok_or(Error::Conflict)?;
    let control = store.get_control(&reference.task_id)?.filter(|control| control.attempt_id == reference.attempt_id).ok_or(Error::Conflict)?;
    if attempt.phase != AttemptPhase::Stopped || control.phase != ControlPhase::Stopped || control.kind != ControlKind::Takeover
        || attempt.step_id != reference.step_id || attempt.worker_instance_id != reference.worker_instance_id
        || attempt.host_session_id != reference.host_session_id || reference.work_ref_id != format!("work_{}",attempt.accepted_sequence) {
        return Err(Error::StopRequired);
    }
    Ok(port.focus(reference))
}
