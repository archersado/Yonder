//! 任务用例与事务存储 Port；不依赖具体 Adapter。
pub use yonder_domain::{Action, Status, Transition, TransitionError};
pub mod query;
pub mod gateway;
pub mod admission;
pub mod computer_use;
pub mod browser_use;
pub mod work_focus;
pub mod document;
pub mod agent_input;
pub mod region_preview;
pub mod jev_config;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Task {
    pub id: String,
    pub owner_agent_id: String,
    pub name: Option<String>,
    pub status: Status,
    pub sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepDeclaration { pub step_id: String, pub label: String, pub accepted_sequence: u64 }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttemptPhase { Prepared, Observed, Unknown, Stopped }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionAttempt {
    pub task_id: String,
    pub step_id: String,
    pub attempt_id: String,
    pub worker_instance_id: String,
    pub host_session_id: String,
    pub phase: AttemptPhase,
    pub accepted_sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttemptConclusion { Observed { action_succeeded: bool }, Unknown { reason: computer_use::UnknownReason } }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttemptResultRecord {
    pub task_id: String, pub step_id: String, pub attempt_id: String, pub worker_instance_id: String, pub host_session_id: String,
    pub conclusion: AttemptConclusion, pub result_sequence: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlKind { Pause, Cancel, Takeover }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlPhase { Pending, Stopped }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FocusPhase { Locating, Focused, Failed }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlRequestRecord {
    pub task_id: String, pub attempt_id: String, pub control_id: String, pub kind: ControlKind,
    pub phase: ControlPhase, pub accepted_sequence: u64, pub stopped_sequence: Option<u64>,
    pub focus_phase: Option<FocusPhase>, pub focus_failure: Option<work_focus::FocusFailure>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StopRecord {
    pub task_id: String, pub step_id: String, pub attempt_id: String, pub worker_instance_id: String, pub host_session_id: String,
    pub control_id: String, pub kind: ControlKind, pub stop_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepBoundaryRecord {
    pub task_id: String, pub step_id: String, pub attempt_id: String,
    pub worker_instance_id: String, pub host_session_id: String, pub stop_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskEventRecord { pub transition: Transition, pub step_declaration: Option<StepDeclaration>, pub attempt_result: Option<AttemptResultRecord>, pub wait_reason: Option<String> }

/// 由完成认证的组合根提供，不能从请求 JSON 反序列化或按请求 agent_id 构造。
#[derive(Clone, Copy)]
pub enum AuthContext<'a> {
    Agent(&'a str),
    LocalUser(&'a str),
}

impl AuthContext<'_> {
    pub fn agent_id(&self) -> &str { match self { Self::Agent(id) | Self::LocalUser(id) => id } }
    pub fn owner_filter(&self) -> Option<&str> { match self { Self::Agent(id) => Some(id), Self::LocalUser(_) => None } }
    pub fn can_read(&self, task: &Task) -> bool { self.owner_filter().is_none_or(|id| id == task.owner_agent_id) }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    NotFound,
    Conflict,
    InvalidInput,
    PermissionDenied,
    IdempotencyConflict,
    StepConflict,
    QuotaExceeded,
    StopRequired,
    InvalidTransition(TransitionError),
    StorageUnavailable,
}

pub trait TaskStore {
    fn supports_pending_cancel(&self) -> bool { false }
    fn supports_running_filter(&self) -> bool { false }
    /// 未升级的实现拒绝登记；能力声明不访问数据库。
    fn supports_registration(&self) -> bool { false }
    fn register(&mut self, _: &str, _: &str, _: &str, _: Option<&str>) -> Result<Task, Error> { Err(Error::StorageUnavailable) }
    fn supports_step_declarations(&self) -> bool { false }
    fn declare_step(&mut self, _: &str, _: &str, _: u64, _: &str, _: &str) -> Result<(Task, StepDeclaration), Error> { Err(Error::StorageUnavailable) }
    fn get_with_step(&mut self, id: &str) -> Result<(Task, Option<StepDeclaration>), Error> { self.get(id).map(|task| (task, None)) }
    fn events_with_steps(&mut self, id: &str, after: u64, limit: usize) -> Result<Vec<TaskEventRecord>, Error> {
        self.events(id, after, limit).map(|events| events.into_iter().map(|transition| TaskEventRecord { transition, step_declaration: None, attempt_result: None, wait_reason: None }).collect())
    }
    fn supports_execution_attempts(&self) -> bool { false }
    fn supports_browser_references(&self) -> bool { false }
    fn supports_controls(&self) -> bool { false }
    fn supports_wait_for_user(&self) -> bool { false }
    fn wait_for_user(&mut self, _: &str, _: u64, _: &str) -> Result<Task, Error> { Err(Error::StorageUnavailable) }
    fn prepare_attempt(&mut self, _: &ExecutionAttempt, _: u64) -> Result<(Task, ExecutionAttempt), Error> { Err(Error::StorageUnavailable) }
    fn get_attempt(&mut self, _: &str) -> Result<Option<ExecutionAttempt>, Error> { Err(Error::StorageUnavailable) }
    fn get_attempt_result(&mut self, _: &str) -> Result<Option<AttemptResultRecord>, Error> { Err(Error::StorageUnavailable) }
    fn record_attempt_result(&mut self, _: &ExecutionAttempt, _: u64, _: AttemptConclusion) -> Result<(Task, AttemptResultRecord), Error> { Err(Error::StorageUnavailable) }
    fn record_browser_result(&mut self, _: &ExecutionAttempt, _: u64, _: &browser_use::BrowserReferenceRecord) -> Result<(Task, AttemptResultRecord), Error> { Err(Error::StorageUnavailable) }
    fn get_browser_reference(&mut self, _: &str) -> Result<Option<browser_use::BrowserReferenceRecord>, Error> { Err(Error::StorageUnavailable) }
    fn request_control(&mut self, _: &ExecutionAttempt, _: u64, _: &str, _: ControlKind) -> Result<(Task, ControlRequestRecord), Error> { Err(Error::StorageUnavailable) }
    fn get_control(&mut self, _: &str) -> Result<Option<ControlRequestRecord>, Error> { Err(Error::StorageUnavailable) }
    fn begin_focus(&mut self, _: &str, _: &str) -> Result<(Task, ControlRequestRecord), Error> { Err(Error::StorageUnavailable) }
    fn finish_focus(&mut self, _: &str, _: &str, _: Option<work_focus::FocusFailure>) -> Result<(Task, ControlRequestRecord), Error> { Err(Error::StorageUnavailable) }
    fn stop_attempt(&mut self, _: &ExecutionAttempt, _: u64, _: &str, _: ControlKind) -> Result<(Task, StopRecord), Error> { Err(Error::StorageUnavailable) }
    fn advance_attempt(&mut self, _: &ExecutionAttempt, _: u64) -> Result<(Task, StepBoundaryRecord), Error> { Err(Error::StorageUnavailable) }

    fn create(&mut self, id: &str, owner_agent_id: &str) -> Result<Task, Error>;
    fn get(&mut self, id: &str) -> Result<Task, Error>;

    /// 可信调用方限定存储范围；按 id 排他分页，最多 101 行（含下一页探针）。
    fn list(&mut self, owner: Option<&str>, after: Option<&str>, include_finished: bool, limit: usize) -> Result<Vec<Task>, Error>;
    fn list_running(&mut self, _: Option<&str>, _: Option<&str>, _: usize) -> Result<Vec<Task>, Error> { Err(Error::StorageUnavailable) }

    /// 可信宿主全局查询/启动恢复：按 id 升序读取至多 limit 个 running 快照。
    fn running(&mut self, limit: usize) -> Result<Vec<Task>, Error>;

    /// 按 sequence 升序返回 after 之后的事件，最多 limit 条。
    fn events(&mut self, id: &str, after: u64, limit: usize) -> Result<Vec<Transition>, Error>;

    /// 同事务比较序号、更新状态、追加事件及 Outbox；失败必须全部回滚。
    /// 返回成功时持久化已完成，不得仅排入内存队列。
    fn commit(&mut self, id: &str, expected_sequence: u64, change: Transition) -> Result<(), Error>;
}

fn validate_id(id: &str) -> Result<(), Error> {
    if id.is_empty() || id.len() > 128 || !id.bytes().all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c)) {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

pub fn valid_id(id: &str) -> bool { validate_id(id).is_ok() }

pub fn get(store: &mut impl TaskStore, id: &str) -> Result<Task, Error> {
    validate_id(id)?;
    store.get(id)
}

pub fn create(store: &mut impl TaskStore, id: &str, auth: AuthContext<'_>) -> Result<Task, Error> {
    if !matches!(auth, AuthContext::Agent(_)) { return Err(Error::PermissionDenied); }
    validate_id(id)?;
    validate_id(auth.agent_id())?;
    store.create(id, auth.agent_id())
}

/// 已握手Gateway调用；仅登记元数据，不派发或执行说明。
pub fn register(store: &mut impl TaskStore, auth: AuthContext<'_>, key: &str, description: &str, name: Option<&str>) -> Result<Task, Error> {
    if !matches!(auth, AuthContext::Agent(_)) { return Err(Error::PermissionDenied); }
    validate_id(auth.agent_id())?; validate_id(key)?;
    if description.trim().is_empty() || description.len() > 4096 || name.is_some_and(|name| !valid_task_name(name)) { return Err(Error::InvalidInput); }
    store.register(auth.agent_id(), key, description, name)
}

pub fn declare_step(store: &mut impl TaskStore, auth: AuthContext<'_>, task_id: &str, expected: u64, step_id: &str, label: &str) -> Result<(Task, StepDeclaration), Error> {
    if !matches!(auth, AuthContext::Agent(_)) { return Err(Error::PermissionDenied); }
    validate_id(auth.agent_id())?; validate_id(task_id)?; validate_id(step_id)?;
    if expected == 0 || expected > i64::MAX as u64 || !valid_step_label(label) { return Err(Error::InvalidInput); }
    store.declare_step(auth.agent_id(), task_id, expected, step_id, label)
}

pub fn get_with_step(store: &mut impl TaskStore, auth: AuthContext<'_>, task_id: &str) -> Result<(Task, Option<StepDeclaration>), Error> {
    validate_id(auth.agent_id())?; validate_id(task_id)?;
    let result = store.get_with_step(task_id)?;
    if !auth.can_read(&result.0) { return Err(Error::NotFound); }
    Ok(result)
}

/// 仅准备可信内部执行身份；成功不代表动作已派发。
pub fn prepare_attempt(store: &mut impl TaskStore, attempt: &ExecutionAttempt, expected: u64) -> Result<(Task, ExecutionAttempt), Error> {
    if expected == 0 || expected >= i64::MAX as u64 || [attempt.task_id.as_str(), attempt.step_id.as_str(), attempt.attempt_id.as_str(), attempt.worker_instance_id.as_str(), attempt.host_session_id.as_str()].iter().any(|id| !valid_id(id)) || attempt.phase != AttemptPhase::Prepared || attempt.accepted_sequence != 0 {
        return Err(Error::InvalidInput);
    }
    store.prepare_attempt(attempt, expected)
}

/// 任务级Permit由Supervisor继续持有；这里只准备下一次动作身份。
pub fn prepare_next_attempt(store: &mut impl TaskStore, attempt: &ExecutionAttempt, expected: u64) -> Result<(Task, ExecutionAttempt), Error> {
    prepare_attempt(store, attempt, expected)
}

pub struct TaskPage {
    pub tasks: Vec<Task>,
    pub next_after_task_id: Option<String>,
}

pub fn list(store: &mut impl TaskStore, auth: AuthContext<'_>, after: Option<&str>, include_finished: bool, limit: usize) -> Result<TaskPage, Error> {
    validate_id(auth.agent_id())?;
    if let Some(id) = after { validate_id(id)?; }
    if !(1..=100).contains(&limit) { return Err(Error::InvalidInput); }
    let mut tasks = store.list(auth.owner_filter(), after, include_finished, limit + 1)?;
    let more = tasks.len() > limit;
    tasks.truncate(limit);
    let next_after_task_id = if more { tasks.last().map(|task| task.id.clone()) } else { None };
    Ok(TaskPage { tasks, next_after_task_id })
}

pub fn list_running(store: &mut impl TaskStore, auth: AuthContext<'_>, after: Option<&str>, limit: usize) -> Result<TaskPage, Error> {
    validate_id(auth.agent_id())?;
    if let Some(id) = after { validate_id(id)?; }
    if !(1..=100).contains(&limit) { return Err(Error::InvalidInput); }
    let mut tasks = store.list_running(auth.owner_filter(), after, limit + 1)?;
    let more = tasks.len() > limit;
    tasks.truncate(limit);
    let next_after_task_id = if more { tasks.last().map(|task| task.id.clone()) } else { None };
    Ok(TaskPage { tasks, next_after_task_id })
}

pub fn events(store: &mut impl TaskStore, id: &str, after: u64, limit: usize) -> Result<Vec<Transition>, Error> {
    validate_id(id)?;
    if !(1..=100).contains(&limit) {
        return Err(Error::InvalidInput);
    }
    store.get(id)?;
    store.events(id, after, limit)
}

/// 仅未开始任务；停止执行过的任务必须使用Driver停止确认契约。
pub fn cancel_pending(store: &mut impl TaskStore, auth: AuthContext<'_>, id: &str, expected: u64) -> Result<Task, Error> {
    validate_id(auth.agent_id())?;
    if expected == 0 || expected > i64::MAX as u64 { return Err(Error::InvalidInput); }
    let task = get(store, id)?;
    if !auth.can_read(&task) { return Err(Error::NotFound); }
    if task.status == Status::Cancelled && (expected == task.sequence || expected.checked_add(1) == Some(task.sequence)) { return Ok(task); }
    if expected != task.sequence { return Err(Error::Conflict); }
    if task.status != Status::Created { return Err(Error::StopRequired); }
    transition(store, id, expected, Action::Cancel)
}

pub fn transition(store: &mut impl TaskStore, id: &str, expected_sequence: u64, action: Action) -> Result<Task, Error> {
    let task = get(store, id)?;
    if task.sequence != expected_sequence {
        return Err(Error::Conflict);
    }
    let change = task.status.transition(task.sequence, action).map_err(Error::InvalidTransition)?;
    store.commit(id, expected_sequence, change)?;
    Ok(Task { id: task.id, owner_agent_id: task.owner_agent_id, name: task.name, status: change.next, sequence: change.sequence })
}

/// Agent 只可在已确认停止的步骤边界进入等待；恢复由独立用例处理。
pub fn wait_for_user(store: &mut impl TaskStore, auth: AuthContext<'_>, id: &str, expected: u64, reason: &str) -> Result<Task, Error> {
    if !matches!(auth, AuthContext::Agent(_)) { return Err(Error::PermissionDenied); }
    validate_id(auth.agent_id())?; validate_id(id)?;
    let reason = reason.trim();
    if expected == 0 || expected >= i64::MAX as u64 || reason.is_empty() || reason.as_bytes().len() > 512 || reason.chars().any(char::is_control) { return Err(Error::InvalidInput); }
    let task = store.get(id)?;
    if !auth.can_read(&task) { return Err(Error::NotFound); }
    if task.status != Status::Running || task.sequence != expected { return Err(Error::Conflict); }
    if store.get_control(id)?.is_some_and(|control| control.phase == ControlPhase::Pending)
        || !store.get_attempt(id)?.is_some_and(|attempt| attempt.phase == AttemptPhase::Stopped) { return Err(Error::StopRequired); }
    store.wait_for_user(id, expected, reason)
}

/// 仅在当前attempt已完成有效Observe时确认边界；定位与Recording不在本用例内。
pub fn stop_at_boundary(store: &mut impl TaskStore, task_id: &str, attempt_id: &str, kind: ControlKind) -> Result<(Task, StopRecord), Error> {
    if !valid_id(task_id) || !valid_id(attempt_id) { return Err(Error::InvalidInput); }
    let task = store.get(task_id)?;
    let attempt = store.get_attempt(task_id)?.filter(|attempt| attempt.attempt_id == attempt_id).ok_or(Error::Conflict)?;
    let control = store.get_control(task_id)?.filter(|control| control.attempt_id == attempt_id && control.kind == kind).ok_or(Error::StopRequired)?;
    store.stop_attempt(&attempt,task.sequence,&control.control_id,kind)
}

/// 无控制请求时结束已Observe步骤；不改变running状态、不释放任务级资源。
pub fn advance_after_observe(store: &mut impl TaskStore, task_id: &str, attempt_id: &str) -> Result<(Task, StepBoundaryRecord), Error> {
    if !valid_id(task_id) || !valid_id(attempt_id) { return Err(Error::InvalidInput); }
    if store.get_control(task_id)?.is_some_and(|control| control.phase == ControlPhase::Pending) { return Err(Error::StopRequired); }
    let task = store.get(task_id)?;
    let attempt = store.get_attempt(task_id)?.filter(|attempt| attempt.attempt_id == attempt_id && matches!(attempt.phase,AttemptPhase::Observed|AttemptPhase::Stopped)).ok_or(Error::StopRequired)?;
    store.advance_attempt(&attempt,task.sequence)
}

/// 外部控制只登记停止意图；执行器在Observe后的步骤边界另行确认停止。
pub fn request_control(store: &mut impl TaskStore, auth: AuthContext<'_>, task_id: &str, expected: u64, kind: ControlKind) -> Result<(Task, ControlRequestRecord), Error> {
    validate_id(auth.agent_id())?; validate_id(task_id)?;
    if expected == 0 || expected >= i64::MAX as u64 { return Err(Error::InvalidInput); }
    let task = store.get(task_id)?;
    if !auth.can_read(&task) { return Err(Error::NotFound); }
    if task.status != Status::Running { return Err(Error::StopRequired); }
    let attempt = store.get_attempt(task_id)?.ok_or(Error::StopRequired)?;
    // prepared 尚未派发副作用，控制可安全冻结后续派发；unknown 则不能伪造可确认的停止。
    if attempt.phase == AttemptPhase::Unknown { return Err(Error::StopRequired); }
    let control_id = format!("control_{}",attempt.accepted_sequence);
    store.request_control(&attempt,expected,&control_id,kind)
}

/// 仅供单实例宿主在开放 Gateway/执行器前调用；重复至返回 0。
/// 失败阻断启动，已提交的任务保持 interrupted；绝不重放动作。
pub fn recover_running(store: &mut impl TaskStore, limit: usize) -> Result<usize, Error> {
    if !(1..=100).contains(&limit) { return Err(Error::InvalidInput); }
    let tasks = store.running(limit)?;
    for task in &tasks {
        transition(store, &task.id, task.sequence, Action::Interrupt)?;
    }
    Ok(tasks.len())
}

/// 当前表的读取快照；NoRunningTask 不代表执行器已停止或允许桌宠隐藏。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunningState {
    Running,
    NoRunningTask,
    Unknown,
}

/// 仅供可信宿主使用，不按 Agent/页面筛选，不缓存或吞错为空闲。
pub fn running_state(store: &mut impl TaskStore) -> RunningState {
    match store.running(1) {
        Ok(tasks) if tasks.is_empty() => RunningState::NoRunningTask,
        Ok(_) => RunningState::Running,
        Err(_) => RunningState::Unknown,
    }
}

/// 两个来源的非原子观察；NoKnownWork 不是允许隐藏或派发的凭证。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityState { Busy, NoKnownWork, Unknown }

/// 仅供可信宿主；尚未完成恢复时传 None，否则传唯一真实准入实例。
/// 查询之后仍可能有新任务准入，实际收起必须由宿主协调。
pub fn activity_state(store: &mut impl TaskStore, admission: Option<&admission::Admission>) -> ActivityState {
    let Some(admission) = admission else { return ActivityState::Unknown; };
    match (running_state(store), admission.has_occupancy()) {
        (RunningState::Running, _) | (_, Ok(true)) => ActivityState::Busy,
        (RunningState::NoRunningTask, Ok(false)) => ActivityState::NoKnownWork,
        _ => ActivityState::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 仅用来注入读取与提交之间的竞争和存储故障；不作为生产状态源。
    struct Store {
        task: Task,
        log: Vec<Transition>,
        fail: Option<Error>,
    }

    impl TaskStore for Store {
        fn list(&mut self, _: Option<&str>, _: Option<&str>, _: bool, _: usize) -> Result<Vec<Task>, Error> { Err(Error::StorageUnavailable) }
        fn create(&mut self, _: &str, _: &str) -> Result<Task, Error> { Err(Error::Conflict) }
        fn get(&mut self, _: &str) -> Result<Task, Error> { Ok(self.task.clone()) }
        fn running(&mut self, limit: usize) -> Result<Vec<Task>, Error> {
            Ok(std::iter::once(self.task.clone()).filter(|t| t.status == Status::Running).take(limit).collect())
        }
        fn events(&mut self, _: &str, after: u64, limit: usize) -> Result<Vec<Transition>, Error> {
            Ok(self.log.iter().copied().filter(|e| e.sequence > after).take(limit).collect())
        }
        fn commit(&mut self, _: &str, expected: u64, change: Transition) -> Result<(), Error> {
            if let Some(error) = self.fail.take() { return Err(error); }
            if expected != self.task.sequence { return Err(Error::Conflict); }
            self.task.status = change.next;
            self.task.sequence = change.sequence;
            self.log.push(change);
            Ok(())
        }
    }

    #[test]
    fn commit_failure_never_reports_success_or_appends_events() {
        let mut store = Store {
            task: Task { id: "task-1".into(), owner_agent_id: "a1".into(), name: None, status: Status::Created, sequence: 1 },
            log: vec![], fail: None,
        };
        for error in [Error::Conflict, Error::StorageUnavailable] {
            store.fail = Some(error);
            assert!(transition(&mut store, "task-1", 1, Action::Start).is_err());
            assert_eq!(store.task.status, Status::Created);
            assert!(store.log.is_empty());
        }
        let task = transition(&mut store, "task-1", 1, Action::Start).unwrap();
        assert_eq!(task.sequence, 2);
        assert_eq!(transition(&mut store, "task-1", 1, Action::Complete), Err(Error::Conflict));
        assert_eq!(events(&mut store, "task-1", 1, 1).unwrap().len(), 1);
        assert!(events(&mut store, "task-1", 2, 1).unwrap().is_empty());
        assert_eq!(events(&mut store, "task-1", 0, 101), Err(Error::InvalidInput));
        assert_eq!(get(&mut store, "../task"), Err(Error::InvalidInput));
    }
}

pub use yonder_protocol::{valid_step_label, valid_task_name};
