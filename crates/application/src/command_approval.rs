//! Agent 命令的本机一次性批准；完整命令只驻留组合根内存。
use crate::{AuthContext, Status, Task, command::{CommandRequest, validate_request}};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::{Mutex, atomic::{AtomicU64, Ordering}}};

pub const MAX_COMMAND_APPROVALS: usize = 32;
pub const MAX_COMMAND_APPROVAL_LIFETIME_MS: u64 = 5 * 60 * 1000;
pub const MAX_COMMAND_APPROVAL_BYTES: usize = 48 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandApprovalState { AwaitingUser, Approved }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandApprovalSummary { pub command_id: String, pub state: CommandApprovalState, pub expires_at_ms: u64 }

/// 仅供受信任本机 UI 展示；不得经 Gateway、日志或任务事实返回。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandApprovalPreview { pub command_id: String, pub request: CommandRequest, pub expires_at_ms: u64 }

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandApprovalError { InvalidInput, PermissionDenied, NotFound, Expired, Rejected, Capacity, Unavailable }

#[derive(Clone, Debug)]
struct Approval { task_id: String, owner_agent_id: String, sequence: u64, command_digest: [u8;32], request: CommandRequest, expires_at_ms: u64, state: CommandApprovalState }

#[derive(Default)]
pub struct CommandApprovalRegistry { approvals: Mutex<BTreeMap<String, Approval>>, next: AtomicU64 }

impl CommandApprovalRegistry {
    pub fn propose(&self, auth: AuthContext<'_>, task: &Task, request: CommandRequest, now_ms: u64) -> Result<CommandApprovalSummary, CommandApprovalError> {
        if !matches!(auth, AuthContext::Agent(_)) || auth.agent_id() != task.owner_agent_id || task.status != Status::Created { return Err(CommandApprovalError::PermissionDenied); }
        validate_request(&request).map_err(|_| CommandApprovalError::InvalidInput)?;
        if command_bytes(&request) > MAX_COMMAND_APPROVAL_BYTES { return Err(CommandApprovalError::InvalidInput); }
        let mut approvals=self.approvals.lock().map_err(|_| CommandApprovalError::Unavailable)?;
        approvals.retain(|_, value| value.expires_at_ms > now_ms);
        if approvals.len() >= MAX_COMMAND_APPROVALS { return Err(CommandApprovalError::Capacity); }
        let command_id=format!("command_{}",self.next.fetch_add(1,Ordering::Relaxed)+1);
        let expires_at_ms=now_ms+MAX_COMMAND_APPROVAL_LIFETIME_MS;
        let command_digest=command_digest(&request);
        approvals.insert(command_id.clone(),Approval{task_id:task.id.clone(),owner_agent_id:task.owner_agent_id.clone(),sequence:task.sequence,command_digest,request,expires_at_ms,state:CommandApprovalState::AwaitingUser});
        Ok(CommandApprovalSummary{command_id,state:CommandApprovalState::AwaitingUser,expires_at_ms})
    }

    pub fn preview_for_local(&self, auth: AuthContext<'_>, task: &Task, command_id: &str, now_ms: u64) -> Result<CommandApprovalPreview, CommandApprovalError> {
        require_local(auth)?; let mut approvals=self.approvals.lock().map_err(|_| CommandApprovalError::Unavailable)?;
        let approval=fetch(&mut approvals,task,command_id,now_ms)?;
        Ok(CommandApprovalPreview{command_id:command_id.into(),request:approval.request,expires_at_ms:approval.expires_at_ms})
    }

    /// 仅向本机可信 UI 返回指定任务的安全摘要；完整命令仍需逐项预览。
    pub fn list_for_local(&self, auth: AuthContext<'_>, task: &Task, now_ms: u64) -> Result<Vec<CommandApprovalSummary>, CommandApprovalError> {
        require_local(auth)?;
        if !active(task) { return Err(CommandApprovalError::PermissionDenied); }
        let mut approvals=self.approvals.lock().map_err(|_| CommandApprovalError::Unavailable)?;
        approvals.retain(|_, value| value.expires_at_ms > now_ms);
        Ok(approvals.iter().filter_map(|(command_id, approval)| (approval.task_id==task.id && approval.owner_agent_id==task.owner_agent_id && approval.sequence==task.sequence).then(|| CommandApprovalSummary{command_id:command_id.clone(),state:approval.state,expires_at_ms:approval.expires_at_ms})).collect())
    }

    pub fn approve(&self, auth: AuthContext<'_>, task: &Task, command_id: &str, now_ms: u64) -> Result<CommandApprovalSummary, CommandApprovalError> {
        require_local(auth)?; let mut approvals=self.approvals.lock().map_err(|_| CommandApprovalError::Unavailable)?;
        let mut approval=fetch(&mut approvals,task,command_id,now_ms)?;
        if approval.state != CommandApprovalState::AwaitingUser { return Err(CommandApprovalError::Rejected); }
        approval.state=CommandApprovalState::Approved; let expires_at_ms=approval.expires_at_ms; approvals.insert(command_id.into(),approval);
        Ok(CommandApprovalSummary{command_id:command_id.into(),state:CommandApprovalState::Approved,expires_at_ms})
    }

    pub fn reject(&self, auth: AuthContext<'_>, task: &Task, command_id: &str, now_ms: u64) -> Result<(), CommandApprovalError> {
        require_local(auth)?; let mut approvals=self.approvals.lock().map_err(|_| CommandApprovalError::Unavailable)?;
        let _=fetch(&mut approvals,task,command_id,now_ms)?; approvals.remove(command_id); Ok(())
    }

    /// 在启动任务事务前确认引用已经由本机用户批准；不返回命令正文，也不消费引用。
    pub fn verify_for_execution(&self, auth: AuthContext<'_>, task: &Task, expected_sequence: u64, command_id: &str, now_ms: u64) -> Result<(), CommandApprovalError> {
        let mut approvals=self.approvals.lock().map_err(|_| CommandApprovalError::Unavailable)?;
        let approval=approved_for_execution(&mut approvals,auth,task,expected_sequence,command_id,now_ms)?;
        if command_digest(&approval.request) != approval.command_digest { return Err(CommandApprovalError::Unavailable); }
        Ok(())
    }

    /// 执行者必须是归属 Agent；成功返回后引用立即不可重放。
    pub fn consume(&self, auth: AuthContext<'_>, task: &Task, expected_sequence: u64, command_id: &str, now_ms: u64) -> Result<CommandRequest, CommandApprovalError> {
        let mut approvals=self.approvals.lock().map_err(|_| CommandApprovalError::Unavailable)?;
        let approval=approved_for_execution(&mut approvals,auth,task,expected_sequence,command_id,now_ms)?;
        if command_digest(&approval.request) != approval.command_digest { return Err(CommandApprovalError::Unavailable); }
        approvals.remove(command_id); Ok(approval.request)
    }

    pub fn revoke_owner(&self, auth: AuthContext<'_>, owner: &str) -> Result<usize, CommandApprovalError> {
        require_local(auth)?; let mut approvals=self.approvals.lock().map_err(|_| CommandApprovalError::Unavailable)?; let count=approvals.len(); approvals.retain(|_,item|item.owner_agent_id!=owner); Ok(count-approvals.len())
    }
}

fn fetch(approvals:&mut BTreeMap<String,Approval>,task:&Task,id:&str,now_ms:u64)->Result<Approval,CommandApprovalError>{
    let approval=approvals.get(id).cloned().ok_or(CommandApprovalError::NotFound)?;
    if approval.expires_at_ms<=now_ms { approvals.remove(id); return Err(CommandApprovalError::Expired); }
    if approval.task_id!=task.id || approval.owner_agent_id!=task.owner_agent_id || approval.sequence!=task.sequence || !active(task) { approvals.remove(id); return Err(CommandApprovalError::PermissionDenied); }
    Ok(approval)
}
fn approved_for_execution(approvals:&mut BTreeMap<String,Approval>,auth:AuthContext<'_>,task:&Task,expected_sequence:u64,id:&str,now_ms:u64)->Result<Approval,CommandApprovalError>{
    if !matches!(auth,AuthContext::Agent(_))||auth.agent_id()!=task.owner_agent_id||task.sequence!=expected_sequence||!active(task){return Err(CommandApprovalError::PermissionDenied);}
    let approval=fetch(approvals,task,id,now_ms)?;
    if approval.sequence!=expected_sequence{return Err(CommandApprovalError::PermissionDenied);}
    if approval.state!=CommandApprovalState::Approved{return Err(CommandApprovalError::Rejected);}
    Ok(approval)
}
fn require_local(auth:AuthContext<'_>)->Result<(),CommandApprovalError>{if matches!(auth,AuthContext::LocalUser(_)){Ok(())}else{Err(CommandApprovalError::PermissionDenied)}}
fn active(task:&Task)->bool{matches!(task.status,Status::Created|Status::Running|Status::WaitingForUser|Status::Paused)}
fn command_bytes(value:&CommandRequest)->usize{value.program.len()+value.cwd.len()+value.args.iter().map(String::len).sum::<usize>()+value.env.iter().map(|(k,v)|k.len()+v.len()).sum::<usize>()}
fn command_digest(value:&CommandRequest)->[u8;32]{ let mut digest=Sha256::new(); for value in std::iter::once(&value.program).chain(value.args.iter()).chain(std::iter::once(&value.cwd)).chain(value.env.iter().flat_map(|(key,value)|[key,value])) { digest.update((value.len() as u64).to_be_bytes()); digest.update(value.as_bytes()); } digest.update(value.timeout_ms.to_be_bytes()); digest.finalize().into() }

#[cfg(test)] mod tests { use super::*; use crate::TaskSource; use std::collections::BTreeMap;
fn task()->Task{Task{id:"task-a".into(),owner_agent_id:"agent-a".into(),name:None,source:TaskSource::LocalAgent,status:Status::Created,sequence:4}}
fn request()->CommandRequest{CommandRequest{program:"/usr/bin/printf".into(),args:vec!["%s".into(),"ok".into()],cwd:"/tmp".into(),env:BTreeMap::new(),timeout_ms:1000}}
#[test] fn approval_is_local_bound_once_and_expires(){let r=CommandApprovalRegistry::default();let t=task();let a=r.propose(AuthContext::Agent("agent-a"),&t,request(),100).unwrap();assert_eq!(a.state,CommandApprovalState::AwaitingUser);assert_eq!(r.verify_for_execution(AuthContext::Agent("agent-a"),&t,4,&a.command_id,101),Err(CommandApprovalError::Rejected));assert_eq!(r.consume(AuthContext::Agent("agent-a"),&t,4,&a.command_id,101),Err(CommandApprovalError::Rejected));assert_eq!(r.preview_for_local(AuthContext::Agent("agent-a"),&t,&a.command_id,101),Err(CommandApprovalError::PermissionDenied));assert_eq!(r.list_for_local(AuthContext::LocalUser("desktop"),&t,101).unwrap(),vec![a.clone()]);assert!(r.preview_for_local(AuthContext::LocalUser("desktop"),&t,&a.command_id,101).is_ok());r.approve(AuthContext::LocalUser("desktop"),&t,&a.command_id,101).unwrap();assert_eq!(r.verify_for_execution(AuthContext::Agent("agent-a"),&t,4,&a.command_id,102),Ok(()));assert_eq!(r.consume(AuthContext::Agent("other"),&t,4,&a.command_id,102),Err(CommandApprovalError::PermissionDenied));assert_eq!(r.consume(AuthContext::Agent("agent-a"),&t,5,&a.command_id,102),Err(CommandApprovalError::PermissionDenied));assert_eq!(r.consume(AuthContext::Agent("agent-a"),&t,4,&a.command_id,102).unwrap(),request());assert_eq!(r.consume(AuthContext::Agent("agent-a"),&t,4,&a.command_id,103),Err(CommandApprovalError::NotFound));let b=r.propose(AuthContext::Agent("agent-a"),&t,request(),200).unwrap();assert_eq!(r.preview_for_local(AuthContext::LocalUser("desktop"),&t,&b.command_id,200+MAX_COMMAND_APPROVAL_LIFETIME_MS),Err(CommandApprovalError::Expired));}
#[test] fn proposal_requires_created_task(){let r=CommandApprovalRegistry::default();let mut t=task();t.status=Status::Running;assert_eq!(r.propose(AuthContext::Agent("agent-a"),&t,request(),100),Err(CommandApprovalError::PermissionDenied));}
#[test] fn sequence_change_invalidates_preview_and_listing(){let r=CommandApprovalRegistry::default();let mut t=task();let a=r.propose(AuthContext::Agent("agent-a"),&t,request(),100).unwrap();t.sequence+=1;assert!(r.list_for_local(AuthContext::LocalUser("desktop"),&t,101).unwrap().is_empty());assert_eq!(r.preview_for_local(AuthContext::LocalUser("desktop"),&t,&a.command_id,101),Err(CommandApprovalError::PermissionDenied));assert_eq!(r.preview_for_local(AuthContext::LocalUser("desktop"),&task(),&a.command_id,101),Err(CommandApprovalError::NotFound));}
}
