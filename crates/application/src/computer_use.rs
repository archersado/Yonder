use crate::{AttemptResultRecord, AuthContext, Error, ExecutionAttempt, Status, Task, TaskStore, admission::{Admission, Resource, start_execution}};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkTarget { pub pid: u32, pub window_id: u32 }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputerAction { pub tool_name:String, pub arguments_json:String }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputerObservation { pub element_count:u16, pub screenshot_path:Option<String>, pub screenshot_mime:Option<String>, pub target_visible:Option<bool> }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnknownReason { InvalidInput, DependencyUnavailable, WorkerFailed, TimedOut, InvalidResponse, IdentityMismatch, ObserveFailed, UserInput }

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DispatchOutcome {
    Known { action_succeeded: bool, observation:Option<ComputerObservation> },
    Unknown(UnknownReason),
}

/// 只供可信 Application 编排；Agent/UI 不得直接构造目标或上报执行结果。
pub trait ComputerUsePort {
    fn dispatch(&self, attempt: &ExecutionAttempt, target: &WorkTarget, action: &ComputerAction) -> DispatchOutcome;
    fn end_session(&self) {}
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
    let (task,step)=crate::get_with_step(store,auth,task_id)?; let step=step.ok_or(Error::StopRequired)?;
    if task.sequence!=expected{return Err(Error::Conflict);}
    let target=targets.frontmost().map_err(|_|Error::StopRequired)?;
    let attempt=ExecutionAttempt{task_id:task_id.into(),step_id:step.step_id,attempt_id:format!("attempt_{}",expected+1),worker_instance_id:"cua_worker".into(),host_session_id:host_session_id.into(),phase:crate::AttemptPhase::Prepared,accepted_sequence:0};
    let accepted=start_execution(store,admission,&task,&attempt,expected,&[Resource::Desktop]).map_err(|_|Error::StopRequired)?.1;
    let action=ComputerAction{tool_name:tool_name.into(),arguments_json:arguments_json.into()};
    let outcome=dispatch_prepared(store,port,task_id,&accepted.attempt_id,&target,&action)?;
    let observation=match &outcome{DispatchOutcome::Known{observation,..}=>observation.clone(),DispatchOutcome::Unknown(_)=>None};
    let interrupted=outcome==DispatchOutcome::Unknown(UnknownReason::UserInput);
    let (result_task,result)=record_dispatch_outcome(store,task_id,&accepted.attempt_id,outcome)?;
    if interrupted{
        let task=crate::transition(store,task_id,result_task.sequence,crate::Action::Interrupt)?;
        admission.release_task_after_stop(task_id).map_err(|_|Error::StorageUnavailable)?;
        return Ok((task,result,observation));
    }
    Ok((result_task,result,observation))
}

pub fn execute_agent_step(
    store:&mut impl TaskStore,admission:&Admission,port:&(impl ComputerUsePort + ?Sized),targets:&(impl WorkTargetPort + ?Sized),auth:AuthContext<'_>,
    task_id:&str,expected:u64,step_id:&str,label:&str,tool_name:&str,arguments_json:&str,host_session_id:&str,
)->Result<(Task,AttemptResultRecord,Option<ComputerObservation>),Error>{
    let (declared,_)=crate::declare_step(store,auth,task_id,expected,step_id,label)?;
    let (task,result,observation)=execute_agent_action(store,admission,port,targets,auth,task_id,declared.sequence,tool_name,arguments_json,host_session_id)?;
    if !matches!(&result.conclusion,crate::AttemptConclusion::Observed{..}){return Ok((task,result,observation));}
    let (advanced,_)=crate::advance_after_observe(store,task_id,&result.attempt_id)?;
    Ok((advanced,result,observation))
}

pub fn complete_agent_task(store:&mut impl TaskStore,admission:&Admission,auth:AuthContext<'_>,task_id:&str,expected:u64)->Result<Task,Error>{
    finish_agent_task(store,admission,auth,task_id,expected,false)
}

pub fn fail_agent_task(store:&mut impl TaskStore,admission:&Admission,auth:AuthContext<'_>,task_id:&str,expected:u64)->Result<Task,Error>{
    finish_agent_task(store,admission,auth,task_id,expected,true)
}

fn finish_agent_task(store:&mut impl TaskStore,admission:&Admission,auth:AuthContext<'_>,task_id:&str,expected:u64,failed:bool)->Result<Task,Error>{
    let (task,_)=crate::get_with_step(store,auth,task_id)?;
    if task.sequence!=expected{return Err(Error::Conflict);}
    if task.status!=Status::Running||store.get_control(task_id)?.is_some_and(|value|value.phase==crate::ControlPhase::Pending)||!admission.holds_resource(task_id,Resource::Desktop).map_err(|_|Error::StorageUnavailable)?{return Err(Error::StopRequired);}
    if !store.get_attempt(task_id)?.is_some_and(|value|value.phase==crate::AttemptPhase::Stopped){return Err(Error::StopRequired);}
    if !store.get_attempt_result(task_id)?.is_some_and(|value|value.conclusion==crate::AttemptConclusion::Observed{action_succeeded:!failed}){return Err(Error::StopRequired);}
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
        DispatchOutcome::Unknown(reason) => crate::AttemptConclusion::Unknown { reason },
    };
    store.record_attempt_result(&attempt, task.sequence, conclusion)
}
