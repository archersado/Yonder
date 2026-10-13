//! 活动执行的单一内存状态所有者。SQLite projector 只消费这里产生的连续事件，
//! 不参与 Driver、Observe 或下一步决策。
use crate::valid_id;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

pub const DEFAULT_COMMAND_CAPACITY: usize = 128;
pub const DEFAULT_EVENT_HARD_LIMIT: usize = 256;
const SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimePhase {
    Running,
    HandBack,
    Paused,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
    PersistenceBackpressure,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeStepPhase {
    Executing,
    Completed,
    Unverified,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RuntimeStep {
    pub step_id: String,
    pub label: String,
    pub attempt_id: String,
    pub worker_instance_id: String,
    pub host_session_id: String,
    pub phase: RuntimeStepPhase,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RuntimeBrowserReference {
    pub external_task_ref: String,
    pub ownership: String,
    pub managed_pages: usize,
    pub finished: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RuntimePlanCursor { pub plan_id:String, pub plan_version:u64, pub current_slot:u16 }
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RuntimeControl { pub attempt_id:String, pub control_id:String, pub kind:String, pub accepted_sequence:u64 }

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RuntimeSnapshot {
    pub task_id: String,
    pub sequence: u64,
    pub checkpoint: u64,
    pub phase: RuntimePhase,
    pub current_step: Option<RuntimeStep>,
    pub last_step: Option<RuntimeStep>,
    pub last_step_succeeded: Option<bool>,
    pub last_observation_sequence: Option<u64>,
    pub goal_verification: Option<crate::GoalVerificationRecord>,
    pub browser_reference: Option<RuntimeBrowserReference>,
    pub active_plan: Option<RuntimePlanCursor>,
    pub active_control: Option<RuntimeControl>,
    pub pending_events: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RuntimeEventKind {
    Activated,
    StepStarted {
        step_id: String,
        label: String,
        attempt_id: String,
        worker_instance_id: String,
        host_session_id: String,
    },
    StepCompleted {
        step_id: String,
        action_succeeded: bool,
    },
    StepUnverified {
        step_id: String,
        reason: crate::unknown_reason::UnknownReason,
    },
    StepBoundaryAdvanced { step_id: String },
    PlanSlotAdvanced { plan_id:String, plan_version:u64, from_slot:u16, step_id:String },
    ControlStopped { attempt_id:String, control_id:String, control_kind:String },
    BrowserReferenceUpdated { reference: RuntimeBrowserReference },
    HandedBack { reason: String },
    GoalVerified {
        verification_id: String,
        observation_sequence: u64,
        outcome: crate::GoalVerificationOutcome,
    },
    Terminal { phase: RuntimePhase },
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RuntimeEvent {
    pub task_id: String,
    pub sequence: u64,
    pub kind: RuntimeEventKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeError {
    InvalidInput,
    NotFound,
    Conflict,
    Backpressure,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeCommand {
    Activate {
        task_id: String,
        checkpoint: u64,
    },
    StartStep {
        task_id: String,
        step_id: String,
        label: String,
        attempt_id: String,
        worker_instance_id: String,
        host_session_id: String,
    },
    CompleteStep {
        task_id: String,
        step_id: String,
        action_succeeded: bool,
    },
    UnverifyStep {
        task_id: String,
        step_id: String,
        reason: crate::unknown_reason::UnknownReason,
    },
    AdvanceStepBoundary { task_id: String, step_id: String },
    BindPlan { task_id:String, plan_id:String, plan_version:u64, current_slot:u16, source_sequence:u64 },
    AdvancePlanSlot { task_id:String, plan_id:String, plan_version:u64, expected_slot:u16, step_id:String },
    StopForControl { task_id:String, control_kind:String },
    UpdateBrowserReference {
        task_id: String,
        reference: RuntimeBrowserReference,
    },
    HandBack {
        task_id: String,
        reason: String,
    },
    VerifyGoal {
        task_id: String,
        verification_id: String,
        observation_sequence: u64,
        outcome: crate::GoalVerificationOutcome,
    },
    Terminate {
        task_id: String,
        phase: RuntimePhase,
    },
    Ack {
        task_id: String,
        through_sequence: u64,
    },
}

struct ActiveExecution {
    snapshot: RuntimeSnapshot,
    pending: VecDeque<RuntimeEvent>,
}
struct State {
    tasks: HashMap<String, ActiveExecution>,
    event_hard_limit: usize,
    projector_wakes: Vec<SyncSender<String>>,
}

enum Message {
    Apply(
        RuntimeCommand,
        mpsc::Sender<Result<RuntimeSnapshot, RuntimeError>>,
    ),
    Snapshot(String, mpsc::Sender<Result<RuntimeSnapshot, RuntimeError>>),
    Pending(
        String,
        usize,
        mpsc::Sender<Result<Vec<RuntimeEvent>, RuntimeError>>,
    ),
    Subscribe(SyncSender<String>, mpsc::Sender<Result<(), RuntimeError>>),
    Shutdown,
}

#[derive(Clone)]
pub struct ExecutionRuntimeHandle {
    sender: SyncSender<Message>,
}
pub struct ExecutionRuntime {
    handle: ExecutionRuntimeHandle,
    thread: Option<thread::JoinHandle<()>>,
}

/// Projector 必须把一个连续批次中的状态、事件与 Outbox 原子写入；只有返回成功，
/// worker 才会向运行时 ACK。失败只保留内存积压，不阻塞当前 Driver 返回。
pub trait RuntimeEventProjector: Send + 'static {
    fn project(&mut self, events: &[RuntimeEvent]) -> Result<(), RuntimeProjectionError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimeProjectionError {
    Unavailable,
    Conflict,
}

pub struct RuntimeProjectionWorker {
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

pub fn begin_attempt(
    runtime: &ExecutionRuntimeHandle,
    admission: &crate::admission::Admission,
    task: &crate::Task,
    step: &crate::StepDeclaration,
    expected_sequence: u64,
    worker_instance_id: &str,
    host_session_id: &str,
    resources: &[crate::admission::Resource],
) -> Result<crate::ExecutionAttempt, RuntimeError> {
    if task.id.is_empty()
        || step.step_id.is_empty()
        || !valid_id(worker_instance_id)
        || !valid_id(host_session_id)
    {
        return Err(RuntimeError::InvalidInput);
    }
    let (active, resumed) = match runtime.snapshot(&task.id) {
        Ok(snapshot) => (snapshot, true),
        Err(RuntimeError::NotFound) => {
            if task.sequence != expected_sequence || task.status != crate::Status::Created {
                return Err(RuntimeError::Conflict);
            }
            admission
                .admit_runtime(&task.id, resources)
                .map_err(|_| RuntimeError::Backpressure)?;
            (
                runtime.apply(RuntimeCommand::Activate {
                    task_id: task.id.clone(),
                    checkpoint: expected_sequence,
                })?,
                false,
            )
        }
        Err(error) => return Err(error),
    };
    if (resumed && active.sequence != expected_sequence) || active.phase != RuntimePhase::Running {
        return Err(RuntimeError::Conflict);
    }
    let active = if active
        .current_step
        .as_ref()
        .is_some_and(|step| step.phase == RuntimeStepPhase::Completed)
    {
        runtime.apply(RuntimeCommand::AdvanceStepBoundary {
            task_id: task.id.clone(),
            step_id: active
                .current_step
                .as_ref()
                .expect("completed step checked")
                .step_id
                .clone(),
        })?
    } else {
        active
    };
    admission
        .admit_runtime(&task.id, resources)
        .map_err(|_| RuntimeError::Backpressure)?;
    let accepted_sequence = active
        .sequence
        .checked_add(1)
        .ok_or(RuntimeError::Conflict)?;
    let attempt = crate::ExecutionAttempt {
        task_id: task.id.clone(),
        step_id: step.step_id.clone(),
        attempt_id: format!("attempt_{accepted_sequence}"),
        worker_instance_id: worker_instance_id.into(),
        host_session_id: host_session_id.into(),
        phase: crate::AttemptPhase::Prepared,
        accepted_sequence,
    };
    let snapshot = runtime.apply(RuntimeCommand::StartStep {
        task_id: task.id.clone(),
        step_id: step.step_id.clone(),
        label: step.label.clone(),
        attempt_id: attempt.attempt_id.clone(),
        worker_instance_id: attempt.worker_instance_id.clone(),
        host_session_id: attempt.host_session_id.clone(),
    })?;
    if snapshot.sequence != accepted_sequence {
        return Err(RuntimeError::Conflict);
    }
    Ok(attempt)
}

pub fn record_attempt_outcome(
    runtime: &ExecutionRuntimeHandle,
    attempt: &crate::ExecutionAttempt,
    conclusion: crate::AttemptConclusion,
) -> Result<(RuntimeSnapshot, crate::AttemptResultRecord), RuntimeError> {
    let snapshot = match conclusion {
        crate::AttemptConclusion::Observed { action_succeeded } => runtime.apply(
            RuntimeCommand::CompleteStep {
                task_id: attempt.task_id.clone(),
                step_id: attempt.step_id.clone(),
                action_succeeded,
            },
        )?,
        crate::AttemptConclusion::Unknown { reason } => runtime.apply(
            RuntimeCommand::UnverifyStep {
                task_id: attempt.task_id.clone(),
                step_id: attempt.step_id.clone(),
                reason,
            },
        )?,
    };
    Ok((
        snapshot.clone(),
        crate::AttemptResultRecord {
            task_id: attempt.task_id.clone(),
            step_id: attempt.step_id.clone(),
            attempt_id: attempt.attempt_id.clone(),
            worker_instance_id: attempt.worker_instance_id.clone(),
            host_session_id: attempt.host_session_id.clone(),
            conclusion,
            result_sequence: snapshot.sequence,
        },
    ))
}

pub fn advance_boundary(
    runtime: &ExecutionRuntimeHandle,
    task_id: &str,
    expected_sequence: u64,
) -> Result<RuntimeSnapshot, RuntimeError> {
    let current = runtime.snapshot(task_id)?;
    if current.sequence != expected_sequence {
        return Err(RuntimeError::Conflict);
    }
    let step = current.current_step.ok_or(RuntimeError::Conflict)?;
    if step.phase != RuntimeStepPhase::Completed {
        return Err(RuntimeError::Conflict);
    }
    runtime.apply(RuntimeCommand::AdvanceStepBoundary {
        task_id: task_id.into(),
        step_id: step.step_id,
    })
}

pub fn stop_for_control(
    runtime:&ExecutionRuntimeHandle,store:&mut impl crate::TaskStore,auth:crate::AuthContext<'_>,task_id:&str,expected:u64,kind:crate::ControlKind,
)->Result<(crate::Task,crate::ControlRequestRecord),crate::Error>{
    let task=store.get(task_id)?;if !auth.can_read(&task){return Err(crate::Error::NotFound);}
    let current=runtime.snapshot(task_id).map_err(runtime_application_error)?;if current.sequence!=expected{return Err(crate::Error::Conflict);}
    let kind_name=match kind{crate::ControlKind::Pause=>"pause",crate::ControlKind::Cancel=>"cancel",crate::ControlKind::Takeover=>"takeover"};
    let stopped=runtime.apply(RuntimeCommand::StopForControl{task_id:task_id.into(),control_kind:kind_name.into()}).map_err(runtime_application_error)?;
    let control=stopped.active_control.clone().ok_or(crate::Error::Conflict)?;
    Ok((crate::Task{status:if kind==crate::ControlKind::Cancel{crate::Status::Cancelled}else{crate::Status::Paused},sequence:stopped.sequence,..task},crate::ControlRequestRecord{task_id:task_id.into(),attempt_id:control.attempt_id,control_id:control.control_id,kind,phase:crate::ControlPhase::Stopped,accepted_sequence:control.accepted_sequence,stopped_sequence:Some(control.accepted_sequence),focus_phase:None,focus_failure:None}))
}

fn runtime_application_error(error:RuntimeError)->crate::Error{match error{RuntimeError::InvalidInput=>crate::Error::InvalidInput,RuntimeError::NotFound=>crate::Error::NotFound,RuntimeError::Conflict=>crate::Error::Conflict,RuntimeError::Backpressure=>crate::Error::StopRequired,RuntimeError::Unavailable=>crate::Error::StorageUnavailable}}

impl ExecutionRuntime {
    pub fn start(command_capacity: usize, event_hard_limit: usize) -> Result<Self, RuntimeError> {
        if command_capacity == 0 || event_hard_limit == 0 {
            return Err(RuntimeError::InvalidInput);
        }
        let (sender, receiver) = mpsc::sync_channel(command_capacity);
        let thread = thread::Builder::new()
            .name("yonder-execution-runtime".into())
            .spawn(move || {
                run(
                    receiver,
                    State {
                        tasks: HashMap::new(),
                        event_hard_limit,
                        projector_wakes: Vec::new(),
                    },
                )
            })
            .map_err(|_| RuntimeError::Unavailable)?;
        Ok(Self {
            handle: ExecutionRuntimeHandle { sender },
            thread: Some(thread),
        })
    }
    pub fn with_defaults() -> Result<Self, RuntimeError> {
        Self::start(DEFAULT_COMMAND_CAPACITY, DEFAULT_EVENT_HARD_LIMIT)
    }
    pub fn handle(&self) -> ExecutionRuntimeHandle {
        self.handle.clone()
    }
}

impl Drop for ExecutionRuntime {
    fn drop(&mut self) {
        // 不能用 try_send：命令队列恰好满时会丢失 Shutdown，随后 join 永久等待。
        let _ = self.handle.sender.send(Message::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl ExecutionRuntimeHandle {
    fn request<T>(
        &self,
        build: impl FnOnce(mpsc::Sender<T>) -> Message,
    ) -> Result<T, RuntimeError> {
        let (reply, receive) = mpsc::channel();
        self.sender
            .try_send(build(reply))
            .map_err(|error| match error {
                TrySendError::Full(_) => RuntimeError::Backpressure,
                TrySendError::Disconnected(_) => RuntimeError::Unavailable,
            })?;
        receive.recv().map_err(|_| RuntimeError::Unavailable)
    }
    pub fn apply(&self, command: RuntimeCommand) -> Result<RuntimeSnapshot, RuntimeError> {
        self.request(|reply| Message::Apply(command, reply))?
    }
    pub fn snapshot(&self, task_id: &str) -> Result<RuntimeSnapshot, RuntimeError> {
        if !valid_id(task_id) {
            return Err(RuntimeError::InvalidInput);
        }
        self.request(|reply| Message::Snapshot(task_id.into(), reply))?
    }
    pub fn pending(&self, task_id: &str, limit: usize) -> Result<Vec<RuntimeEvent>, RuntimeError> {
        if !valid_id(task_id) || limit == 0 {
            return Err(RuntimeError::InvalidInput);
        }
        self.request(|reply| Message::Pending(task_id.into(), limit, reply))?
    }

    fn subscribe_projection(&self, capacity: usize) -> Result<Receiver<String>, RuntimeError> {
        if capacity == 0 {
            return Err(RuntimeError::InvalidInput);
        }
        let (wake, receive) = mpsc::sync_channel(capacity);
        self.request(|reply| Message::Subscribe(wake, reply))??;
        Ok(receive)
    }
}

impl RuntimeProjectionWorker {
    pub fn start(
        runtime: ExecutionRuntimeHandle,
        mut projector: impl RuntimeEventProjector,
        batch_size: usize,
        retry_interval: Duration,
    ) -> Result<Self, RuntimeError> {
        if batch_size == 0 || retry_interval.is_zero() {
            return Err(RuntimeError::InvalidInput);
        }
        let wakes = runtime.subscribe_projection(DEFAULT_COMMAND_CAPACITY)?;
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let thread = thread::Builder::new()
            .name("yonder-runtime-projector".into())
            .spawn(move || {
                let mut dirty = HashSet::new();
                let mut failures = HashMap::new();
                loop {
                    if worker_stop.load(Ordering::Acquire) {
                        while let Ok(task_id) = wakes.try_recv() {
                            dirty.insert(task_id);
                        }
                        let deadline = Instant::now() + SHUTDOWN_DRAIN_TIMEOUT;
                        while !dirty.is_empty() && Instant::now() < deadline {
                            project_dirty(
                                &runtime,
                                &mut projector,
                                &mut dirty,
                                &mut failures,
                                batch_size,
                            );
                            while let Ok(task_id) = wakes.try_recv() {
                                dirty.insert(task_id);
                            }
                            if !dirty.is_empty() {
                                thread::sleep(retry_interval.min(Duration::from_millis(10)));
                            }
                        }
                        if !dirty.is_empty() {
                            eprintln!(
                                "运行时投影关闭排空超时：待处理任务数={}",
                                dirty.len()
                            );
                        }
                        break;
                    }
                    match wakes.recv_timeout(retry_interval) {
                        Ok(task_id) => {
                            dirty.insert(task_id);
                            while let Ok(task_id) = wakes.try_recv() {
                                dirty.insert(task_id);
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                    project_dirty(
                        &runtime,
                        &mut projector,
                        &mut dirty,
                        &mut failures,
                        batch_size,
                    );
                }
            })
            .map_err(|_| RuntimeError::Unavailable)?;
        Ok(Self {
            stop,
            thread: Some(thread),
        })
    }
}

impl Drop for RuntimeProjectionWorker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn project_dirty(
    runtime: &ExecutionRuntimeHandle,
    projector: &mut impl RuntimeEventProjector,
    dirty: &mut HashSet<String>,
    failures: &mut HashMap<String, u32>,
    batch_size: usize,
) {
    let task_ids: Vec<_> = dirty.iter().cloned().collect();
    for task_id in task_ids {
        let Ok(events) = runtime.pending(&task_id, batch_size) else {
            continue;
        };
        if events.is_empty() {
            dirty.remove(&task_id);
            continue;
        }
        let through_sequence = events.last().expect("non-empty batch").sequence;
        let projected = match projector.project(&events) {
            Ok(()) => true,
            Err(error) => {
                projection_retry_diagnostic(failures, &task_id, format!("{error:?}"));
                false
            }
        };
        if !projected {
            continue;
        }
        if runtime
            .apply(RuntimeCommand::Ack {
                task_id: task_id.clone(),
                through_sequence,
            })
            .is_err()
        {
            projection_retry_diagnostic(failures, &task_id, "ack".into());
            continue;
        }
        failures.remove(&task_id);
        if events.len() < batch_size {
            dirty.remove(&task_id);
        }
    }
}

fn projection_retry_diagnostic(
    failures: &mut HashMap<String, u32>,
    task_id: &str,
    error: String,
) {
    let attempts = failures.entry(task_id.to_owned()).or_default();
    *attempts = attempts.saturating_add(1);
    if attempts.is_power_of_two() && *attempts <= 1024 {
        eprintln!("运行时投影待重试：分类={error}，累计次数={attempts}");
    }
}

fn run(receiver: Receiver<Message>, mut state: State) {
    while let Ok(message) = receiver.recv() {
        match message {
            Message::Apply(command, reply) => {
                let _ = reply.send(apply(&mut state, command));
            }
            Message::Snapshot(task_id, reply) => {
                let _ = reply.send(
                    state
                        .tasks
                        .get(&task_id)
                        .map(|task| task.snapshot.clone())
                        .ok_or(RuntimeError::NotFound),
                );
            }
            Message::Pending(task_id, limit, reply) => {
                let _ = reply.send(
                    state
                        .tasks
                        .get(&task_id)
                        .map(|task| task.pending.iter().take(limit).cloned().collect())
                        .ok_or(RuntimeError::NotFound),
                );
            }
            Message::Subscribe(wake, reply) => {
                for task in state.tasks.values().filter(|task| !task.pending.is_empty()) {
                    let _ = wake.try_send(task.snapshot.task_id.clone());
                }
                state.projector_wakes.push(wake);
                let _ = reply.send(Ok(()));
            }
            Message::Shutdown => break,
        }
    }
}

fn apply(state: &mut State, command: RuntimeCommand) -> Result<RuntimeSnapshot, RuntimeError> {
    let notify_task_id = match &command {
        RuntimeCommand::Ack { .. } => None,
        command => Some(command_task_id(command).to_owned()),
    };
    let result = match command {
        RuntimeCommand::Activate {
            task_id,
            checkpoint,
        } => {
            if !valid_id(&task_id) || checkpoint == u64::MAX {
                return Err(RuntimeError::InvalidInput);
            }
            if let Some(task) = state.tasks.get(&task_id) {
                return Ok(task.snapshot.clone());
            }
            let mut task = ActiveExecution {
                snapshot: RuntimeSnapshot {
                    task_id: task_id.clone(),
                    sequence: checkpoint,
                    checkpoint,
                    phase: RuntimePhase::Running,
                    current_step: None,
                    last_step: None,
                    last_step_succeeded: None,
                    last_observation_sequence: None,
                    goal_verification: None,
                    browser_reference: None,
                    active_plan: None,
                    active_control: None,
                    pending_events: 0,
                },
                pending: VecDeque::new(),
            };
            append(&mut task, RuntimeEventKind::Activated)?;
            let snapshot = task.snapshot.clone();
            state.tasks.insert(task_id, task);
            Ok(snapshot)
        }
        RuntimeCommand::Ack {
            task_id,
            through_sequence,
        } => {
            let task = state
                .tasks
                .get_mut(&task_id)
                .ok_or(RuntimeError::NotFound)?;
            if through_sequence < task.snapshot.checkpoint
                || through_sequence > task.snapshot.sequence
            {
                return Err(RuntimeError::Conflict);
            }
            while task
                .pending
                .front()
                .is_some_and(|event| event.sequence <= through_sequence)
            {
                task.pending.pop_front();
            }
            task.snapshot.checkpoint = through_sequence;
            task.snapshot.pending_events = task.pending.len();
            if task.snapshot.phase == RuntimePhase::PersistenceBackpressure
                && task.pending.len() < state.event_hard_limit
            {
                task.snapshot.phase = RuntimePhase::Running;
            }
            Ok(task.snapshot.clone())
        }
        command => {
            let task_id = command_task_id(&command);
            if !valid_id(task_id) {
                return Err(RuntimeError::InvalidInput);
            }
            let task = state.tasks.get_mut(task_id).ok_or(RuntimeError::NotFound)?;
            if matches!(command, RuntimeCommand::StartStep { .. })
                && task.pending.len() >= state.event_hard_limit
            {
                task.snapshot.phase = RuntimePhase::PersistenceBackpressure;
                return Err(RuntimeError::Backpressure);
            }
            match command {
                RuntimeCommand::StartStep {
                    step_id,
                    label,
                    attempt_id,
                    worker_instance_id,
                    host_session_id,
                    ..
                } => {
                    if !valid_id(&step_id)
                        || !valid_id(&attempt_id)
                        || !valid_id(&worker_instance_id)
                        || !valid_id(&host_session_id)
                        || label.is_empty()
                        || label.len() > 120
                        || task
                            .snapshot
                            .current_step
                            .as_ref()
                            .is_some_and(|step| step.phase == RuntimeStepPhase::Executing)
                    {
                        return Err(RuntimeError::Conflict);
                    }
                    task.snapshot.phase = RuntimePhase::Running;
                    task.snapshot.last_step_succeeded = None;
                    task.snapshot.last_observation_sequence = None;
                    task.snapshot.goal_verification = None;
                    task.snapshot.current_step = Some(RuntimeStep {
                        step_id: step_id.clone(),
                        label: label.clone(),
                        attempt_id: attempt_id.clone(),
                        worker_instance_id: worker_instance_id.clone(),
                        host_session_id: host_session_id.clone(),
                        phase: RuntimeStepPhase::Executing,
                    });
                    append(
                        task,
                        RuntimeEventKind::StepStarted {
                            step_id,
                            label,
                            attempt_id,
                            worker_instance_id,
                            host_session_id,
                        },
                    )?;
                }
                RuntimeCommand::CompleteStep {
                    step_id,
                    action_succeeded,
                    ..
                } => {
                    require_executing(task, &step_id)?;
                    task.snapshot.current_step.as_mut().unwrap().phase =
                        RuntimeStepPhase::Completed;
                    task.snapshot.last_step_succeeded = Some(action_succeeded);
                    task.snapshot.last_observation_sequence = Some(task.snapshot.sequence.saturating_add(1));
                    append(
                        task,
                        RuntimeEventKind::StepCompleted {
                            step_id,
                            action_succeeded,
                        },
                    )?;
                }
                RuntimeCommand::UnverifyStep {
                    step_id, reason, ..
                } => {
                    require_executing(task, &step_id)?;
                    task.snapshot.current_step.as_mut().unwrap().phase =
                        RuntimeStepPhase::Unverified;
                    task.snapshot.last_step_succeeded = None;
                    append(task, RuntimeEventKind::StepUnverified { step_id, reason })?;
                }
                RuntimeCommand::AdvanceStepBoundary { step_id, .. } => {
                    if !task.snapshot.current_step.as_ref().is_some_and(|step| {
                        step.step_id == step_id && step.phase == RuntimeStepPhase::Completed
                    }) {
                        return Err(RuntimeError::Conflict);
                    }
                    task.snapshot.last_step=task.snapshot.current_step.take();
                    append(task, RuntimeEventKind::StepBoundaryAdvanced { step_id })?;
                }
                RuntimeCommand::BindPlan { plan_id,plan_version,current_slot,source_sequence,.. } => {
                    if !valid_id(&plan_id)||plan_version==0||source_sequence==0{return Err(RuntimeError::InvalidInput);}
                    match task.snapshot.active_plan.as_ref(){
                        Some(cursor) if cursor.plan_id==plan_id&&cursor.plan_version==plan_version&&cursor.current_slot==current_slot=>{}
                        Some(_)=>{
                            if task.snapshot.phase!=RuntimePhase::HandBack||task.snapshot.current_step.as_ref().is_some_and(|step|step.phase==RuntimeStepPhase::Executing)||!task.pending.is_empty()||source_sequence<task.snapshot.sequence{return Err(RuntimeError::Conflict);}
                            task.snapshot.sequence=source_sequence;task.snapshot.checkpoint=source_sequence;task.snapshot.phase=RuntimePhase::Running;task.snapshot.current_step=None;task.snapshot.last_step_succeeded=None;task.snapshot.last_observation_sequence=None;task.snapshot.goal_verification=None;
                            task.snapshot.active_plan=Some(RuntimePlanCursor{plan_id,plan_version,current_slot});
                        }
                        None=>{
                            if source_sequence>task.snapshot.sequence{return Err(RuntimeError::Conflict);}
                            task.snapshot.active_plan=Some(RuntimePlanCursor{plan_id,plan_version,current_slot});
                        }
                    }
                }
                RuntimeCommand::AdvancePlanSlot { plan_id,plan_version,expected_slot,step_id,.. } => {
                    if !valid_id(&step_id){return Err(RuntimeError::InvalidInput);}
                    let cursor=task.snapshot.active_plan.as_mut().ok_or(RuntimeError::Conflict)?;
                    if cursor.plan_id!=plan_id||cursor.plan_version!=plan_version||cursor.current_slot!=expected_slot||task.snapshot.current_step.is_some(){return Err(RuntimeError::Conflict);}
                    cursor.current_slot=cursor.current_slot.checked_add(1).ok_or(RuntimeError::Conflict)?;
                    append(task,RuntimeEventKind::PlanSlotAdvanced{plan_id,plan_version,from_slot:expected_slot,step_id})?;
                }
                RuntimeCommand::StopForControl{control_kind,..}=>{
                    if !matches!(control_kind.as_str(),"pause"|"cancel"|"takeover"){return Err(RuntimeError::InvalidInput);}
                    if !matches!(task.snapshot.phase,RuntimePhase::Running|RuntimePhase::HandBack){return Err(RuntimeError::Conflict);}
                    let attempt_id=task.snapshot.current_step.as_ref().or(task.snapshot.last_step.as_ref()).map(|step|step.attempt_id.clone()).ok_or(RuntimeError::Conflict)?;
                    let accepted_sequence=task.snapshot.sequence.checked_add(1).ok_or(RuntimeError::Conflict)?;let control_id=format!("control_{accepted_sequence}");
                    task.snapshot.current_step=None;task.snapshot.phase=if control_kind=="cancel"{RuntimePhase::Cancelled}else{RuntimePhase::Paused};
                    task.snapshot.active_control=Some(RuntimeControl{attempt_id:attempt_id.clone(),control_id:control_id.clone(),kind:control_kind.clone(),accepted_sequence});
                    append(task,RuntimeEventKind::ControlStopped{attempt_id,control_id,control_kind})?;
                }
                RuntimeCommand::UpdateBrowserReference { reference, .. } => {
                    if !reference
                        .external_task_ref
                        .strip_prefix("ego:")
                        .is_some_and(|value| value.parse::<u64>().is_ok_and(|value| value > 0))
                        || !matches!(
                            reference.ownership.as_str(),
                            "agent" | "agentDelegatedToUser" | "user"
                        )
                        || reference.managed_pages > 100
                    {
                        return Err(RuntimeError::InvalidInput);
                    }
                    task.snapshot.browser_reference = Some(reference.clone());
                    append(task, RuntimeEventKind::BrowserReferenceUpdated { reference })?;
                }
                RuntimeCommand::HandBack { reason, .. } => {
                    if reason.is_empty()||reason.as_bytes().len()>1024||reason.chars().any(|value|value.is_control()&&value!='\n') {
                        return Err(RuntimeError::InvalidInput);
                    }
                    task.snapshot.phase = RuntimePhase::HandBack;
                    append(task, RuntimeEventKind::HandedBack { reason })?;
                }
                RuntimeCommand::VerifyGoal {
                    verification_id,
                    observation_sequence,
                    outcome,
                    ..
                } => {
                    if !valid_id(&verification_id)
                        || observation_sequence == 0
                        || Some(observation_sequence) != task.snapshot.last_observation_sequence
                        || task.snapshot.current_step.is_some()
                        || task.snapshot.last_step_succeeded != Some(true)
                        || !matches!(
                            task.snapshot.phase,
                            RuntimePhase::Running | RuntimePhase::HandBack
                        )
                    {
                        return Err(RuntimeError::Conflict);
                    }
                    let verified_sequence = task.snapshot.sequence.saturating_add(1);
                    task.snapshot.goal_verification = Some(crate::GoalVerificationRecord {
                        task_id: task.snapshot.task_id.clone(),
                        verification_id: verification_id.clone(),
                        observation_sequence,
                        verified_sequence,
                        outcome,
                    });
                    // not-achieved 是慢脑给出的终局否定事实：任务保持非终态，
                    // 但当前片段必须进入可重规划的 HandBack，而不能继续保留
                    // Running + 旧 active_plan 形成一个永远无法绑定新版本的状态。
                    if outcome == crate::GoalVerificationOutcome::NotAchieved {
                        task.snapshot.phase = RuntimePhase::HandBack;
                    } else if task.snapshot.phase == RuntimePhase::HandBack {
                        // Jev 可能因无法解释开放业务语义而先交回；
                        // 归属慢脑对同一新鲜、成功 Observation 的终局核验
                        // 只恢复终结门禁，不重启 Driver 或重放任何动作。
                        task.snapshot.phase = RuntimePhase::Running;
                    }
                    append(
                        task,
                        RuntimeEventKind::GoalVerified {
                            verification_id,
                            observation_sequence,
                            outcome,
                        },
                    )?;
                }
                RuntimeCommand::Terminate { phase, .. } => {
                    if !matches!(
                        phase,
                        RuntimePhase::Completed
                            | RuntimePhase::Failed
                            | RuntimePhase::Cancelled
                            | RuntimePhase::Interrupted
                    ) {
                        return Err(RuntimeError::InvalidInput);
                    }
                    task.snapshot.phase = phase.clone();
                    append(task, RuntimeEventKind::Terminal { phase })?;
                }
            RuntimeCommand::Activate { .. } | RuntimeCommand::Ack { .. } => unreachable!(),
            }
            Ok(task.snapshot.clone())
        }
    };
    if result.is_ok() {
        if let Some(task_id) = notify_task_id {
            state
                .projector_wakes
                .retain(|wake| match wake.try_send(task_id.clone()) {
                    Ok(()) | Err(TrySendError::Full(_)) => true,
                    Err(TrySendError::Disconnected(_)) => false,
                });
        }
    }
    result
}

fn append(task: &mut ActiveExecution, kind: RuntimeEventKind) -> Result<(), RuntimeError> {
    let sequence = task
        .snapshot
        .sequence
        .checked_add(1)
        .ok_or(RuntimeError::Conflict)?;
    task.snapshot.sequence = sequence;
    task.pending.push_back(RuntimeEvent {
        task_id: task.snapshot.task_id.clone(),
        sequence,
        kind,
    });
    task.snapshot.pending_events = task.pending.len();
    Ok(())
}
fn require_executing(task: &ActiveExecution, step_id: &str) -> Result<(), RuntimeError> {
    if task
        .snapshot
        .current_step
        .as_ref()
        .is_some_and(|step| step.step_id == step_id && step.phase == RuntimeStepPhase::Executing)
    {
        Ok(())
    } else {
        Err(RuntimeError::Conflict)
    }
}
fn command_task_id(command: &RuntimeCommand) -> &str {
    match command {
        RuntimeCommand::Activate { task_id, .. }
        | RuntimeCommand::StartStep { task_id, .. }
        | RuntimeCommand::CompleteStep { task_id, .. }
        | RuntimeCommand::UnverifyStep { task_id, .. }
        | RuntimeCommand::AdvanceStepBoundary { task_id, .. }
        | RuntimeCommand::BindPlan { task_id, .. }
        | RuntimeCommand::AdvancePlanSlot { task_id, .. }
        | RuntimeCommand::StopForControl { task_id, .. }
        | RuntimeCommand::UpdateBrowserReference { task_id, .. }
        | RuntimeCommand::HandBack { task_id, .. }
        | RuntimeCommand::VerifyGoal { task_id, .. }
        | RuntimeCommand::Terminate { task_id, .. }
        | RuntimeCommand::Ack { task_id, .. } => task_id,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    fn start_step(task_id: &str, step_id: &str, label: &str) -> RuntimeCommand {
        RuntimeCommand::StartStep {
            task_id: task_id.into(),
            step_id: step_id.into(),
            label: label.into(),
            attempt_id: format!("attempt-{step_id}"),
            worker_instance_id: "worker".into(),
            host_session_id: "host".into(),
        }
    }
    #[test]
    fn reducer_advances_without_a_persistence_ack() {
        let runtime = ExecutionRuntime::start(8, 8).unwrap();
        let handle = runtime.handle();
        let active = handle
            .apply(RuntimeCommand::Activate {
                task_id: "task-1".into(),
                checkpoint: 7,
            })
            .unwrap();
        assert_eq!(active.sequence, 8);
        assert_eq!(active.checkpoint, 7);
        handle
            .apply(start_step("task-1", "open", "打开应用"))
            .unwrap();
        let completed = handle
            .apply(RuntimeCommand::CompleteStep {
                task_id: "task-1".into(),
                step_id: "open".into(),
                action_succeeded: true,
            })
            .unwrap();
        assert_eq!(completed.sequence, 10);
        assert_eq!(completed.checkpoint, 7);
        assert_eq!(
            completed.current_step.unwrap().phase,
            RuntimeStepPhase::Completed
        );
        assert_eq!(handle.pending("task-1", 8).unwrap().len(), 3);
    }
    #[test]
    fn plan_cursor_advances_and_hands_back_without_a_persistence_ack(){
        let runtime=ExecutionRuntime::start(16,16).unwrap();let handle=runtime.handle();
        handle.apply(RuntimeCommand::Activate{task_id:"plan-task".into(),checkpoint:2}).unwrap();
        handle.apply(RuntimeCommand::BindPlan{task_id:"plan-task".into(),plan_id:"plan-v1".into(),plan_version:1,current_slot:0,source_sequence:2}).unwrap();
        handle.apply(start_step("plan-task","open","打开应用")).unwrap();
        handle.apply(RuntimeCommand::CompleteStep{task_id:"plan-task".into(),step_id:"open".into(),action_succeeded:true}).unwrap();
        handle.apply(RuntimeCommand::AdvanceStepBoundary{task_id:"plan-task".into(),step_id:"open".into()}).unwrap();
        let advanced=handle.apply(RuntimeCommand::AdvancePlanSlot{task_id:"plan-task".into(),plan_id:"plan-v1".into(),plan_version:1,expected_slot:0,step_id:"open".into()}).unwrap();
        assert_eq!(advanced.active_plan.unwrap().current_slot,1);assert_eq!(advanced.checkpoint,2);
        let handed_back=handle.apply(RuntimeCommand::HandBack{task_id:"plan-task".into(),reason:"需要慢脑重新规划".into()}).unwrap();
        assert_eq!(handed_back.phase,RuntimePhase::HandBack);assert_eq!(handed_back.checkpoint,2);assert_eq!(handle.pending("plan-task",16).unwrap().len(),6);
    }
    #[test]
    fn hard_backpressure_stops_the_next_side_effect_but_accepts_the_current_result() {
        let runtime = ExecutionRuntime::start(8, 2).unwrap();
        let handle = runtime.handle();
        handle
            .apply(RuntimeCommand::Activate {
                task_id: "task-1".into(),
                checkpoint: 0,
            })
            .unwrap();
        handle
            .apply(start_step("task-1", "one", "第一步"))
            .unwrap();
        let result = handle
            .apply(RuntimeCommand::CompleteStep {
                task_id: "task-1".into(),
                step_id: "one".into(),
                action_succeeded: true,
            })
            .unwrap();
        assert_eq!(result.pending_events, 3);
        assert_eq!(
            handle.apply(start_step("task-1", "two", "第二步")),
            Err(RuntimeError::Backpressure)
        );
        assert_eq!(
            handle.snapshot("task-1").unwrap().phase,
            RuntimePhase::PersistenceBackpressure
        );
    }
    #[test]
    fn checkpoint_ack_is_continuous_and_releases_backpressure() {
        let runtime = ExecutionRuntime::start(8, 2).unwrap();
        let handle = runtime.handle();
        handle
            .apply(RuntimeCommand::Activate {
                task_id: "task-1".into(),
                checkpoint: 4,
            })
            .unwrap();
        handle
            .apply(start_step("task-1", "one", "第一步"))
            .unwrap();
        assert!(handle
            .apply(start_step("task-1", "two", "第二步"))
            .is_err());
        assert_eq!(
            handle
                .apply(RuntimeCommand::Ack {
                    task_id: "task-1".into(),
                    through_sequence: 6
                })
                .unwrap()
                .pending_events,
            0
        );
        assert_eq!(
            handle.snapshot("task-1").unwrap().phase,
            RuntimePhase::Running
        );
        assert_eq!(
            handle.apply(RuntimeCommand::Ack {
                task_id: "task-1".into(),
                through_sequence: 7
            }),
            Err(RuntimeError::Conflict)
        );
    }
    #[test]
    fn unknown_result_and_handback_are_immediate_runtime_facts() {
        let runtime = ExecutionRuntime::start(8, 8).unwrap();
        let handle = runtime.handle();
        handle
            .apply(RuntimeCommand::Activate {
                task_id: "task-1".into(),
                checkpoint: 0,
            })
            .unwrap();
        handle
            .apply(start_step("task-1", "observe", "观察"))
            .unwrap();
        handle
            .apply(RuntimeCommand::UnverifyStep {
                task_id: "task-1".into(),
                step_id: "observe".into(),
                reason: crate::unknown_reason::UnknownReason::ObserveFailed,
            })
            .unwrap();
        let handback = handle
            .apply(RuntimeCommand::HandBack {
                task_id: "task-1".into(),
                reason: "需要慢脑重新观察".into(),
            })
            .unwrap();
        assert_eq!(handback.phase, RuntimePhase::HandBack);
        assert_eq!(
            handback.current_step.unwrap().phase,
            RuntimeStepPhase::Unverified
        );
    }

    struct FlakyProjector {
        attempts: Arc<AtomicUsize>,
        projected: mpsc::Sender<Vec<RuntimeEvent>>,
    }

    impl RuntimeEventProjector for FlakyProjector {
        fn project(&mut self, events: &[RuntimeEvent]) -> Result<(), RuntimeProjectionError> {
            if self.attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                return Err(RuntimeProjectionError::Unavailable);
            }
            self.projected
                .send(events.to_vec())
                .map_err(|_| RuntimeProjectionError::Unavailable)
        }
    }

    #[test]
    fn projector_retries_without_blocking_runtime_progress_and_acks_after_success() {
        let runtime = ExecutionRuntime::start(8, 8).unwrap();
        let handle = runtime.handle();
        let attempts = Arc::new(AtomicUsize::new(0));
        let (projected, receive) = mpsc::channel();
        let _worker = RuntimeProjectionWorker::start(
            handle.clone(),
            FlakyProjector {
                attempts: Arc::clone(&attempts),
                projected,
            },
            8,
            Duration::from_millis(10),
        )
        .unwrap();

        handle
            .apply(RuntimeCommand::Activate {
                task_id: "task-1".into(),
                checkpoint: 0,
            })
            .unwrap();
        let running = handle
            .apply(start_step("task-1", "open", "打开应用"))
            .unwrap();
        assert_eq!(running.sequence, 2);
        assert_eq!(running.checkpoint, 0);

        let events = receive.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(events.len(), 2);
        for _ in 0..100 {
            if handle.snapshot("task-1").unwrap().checkpoint == 2 {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(handle.snapshot("task-1").unwrap().checkpoint, 2);
        assert!(attempts.load(Ordering::SeqCst) >= 2);
    }

    #[test]
    fn common_attempt_lifecycle_reuses_one_runtime_for_consecutive_steps() {
        let runtime = ExecutionRuntime::start(8, 16).unwrap();
        let handle = runtime.handle();
        let admission = crate::admission::Admission::new(4).unwrap();
        let task = crate::Task {
            id: "task-1".into(),
            owner_agent_id: "agent-a".into(),
            name: Some("跨能力任务".into()),
            source: crate::TaskSource::LocalAgent,
            status: crate::Status::Created,
            sequence: 2,
        };
        let first_step = crate::StepDeclaration {
            step_id: "first".into(),
            label: "第一步".into(),
            accepted_sequence: 2,
        };
        let first = begin_attempt(
            &handle,
            &admission,
            &task,
            &first_step,
            2,
            "cua_worker",
            "host",
            &[crate::admission::Resource::Desktop],
        )
        .unwrap();
        assert_eq!(first.accepted_sequence, 4);
        let (observed, result) = record_attempt_outcome(
            &handle,
            &first,
            crate::AttemptConclusion::Observed {
                action_succeeded: true,
            },
        )
        .unwrap();
        assert_eq!(observed.sequence, 5);
        assert_eq!(result.result_sequence, 5);
        let boundary = handle
            .apply(RuntimeCommand::AdvanceStepBoundary {
                task_id: task.id.clone(),
                step_id: first.step_id,
            })
            .unwrap();
        assert_eq!(boundary.sequence, 6);

        let second_step = crate::StepDeclaration {
            step_id: "second".into(),
            label: "第二步".into(),
            accepted_sequence: 6,
        };
        let second = begin_attempt(
            &handle,
            &admission,
            &task,
            &second_step,
            6,
            "command_worker",
            "host",
            &[],
        )
        .unwrap();
        assert_eq!(second.accepted_sequence, 7);
        assert!(admission.holds("task-1").unwrap());
    }

    #[test]
    fn next_attempt_publishes_the_missing_observed_boundary() {
        let runtime = ExecutionRuntime::start(8, 16).unwrap();
        let handle = runtime.handle();
        let admission = crate::admission::Admission::new(2).unwrap();
        let task = crate::Task {
            id: "task-boundary".into(),
            owner_agent_id: "agent-a".into(),
            name: Some("连续 BUA 动作".into()),
            source: crate::TaskSource::LocalAgent,
            status: crate::Status::Created,
            sequence: 2,
        };
        let first_step = crate::StepDeclaration {
            step_id: "browse".into(),
            label: "浏览器操作".into(),
            accepted_sequence: 2,
        };
        let first = begin_attempt(
            &handle,
            &admission,
            &task,
            &first_step,
            2,
            "ego_bridge",
            "host",
            &[crate::admission::Resource::Browser],
        )
        .unwrap();
        let observed = record_attempt_outcome(
            &handle,
            &first,
            crate::AttemptConclusion::Observed {
                action_succeeded: true,
            },
        )
        .unwrap()
        .0;
        let second = begin_attempt(
            &handle,
            &admission,
            &task,
            &first_step,
            observed.sequence,
            "ego_bridge",
            "host",
            &[crate::admission::Resource::Browser],
        )
        .unwrap();
        assert_eq!(second.accepted_sequence, observed.sequence + 2);
        let pending = handle.pending(&task.id, 16).unwrap();
        assert!(matches!(
            pending[pending.len() - 2].kind,
            RuntimeEventKind::StepBoundaryAdvanced { .. }
        ));
        assert!(matches!(
            pending[pending.len() - 1].kind,
            RuntimeEventKind::StepStarted { .. }
        ));
    }

    #[test]
    fn goal_verification_binds_latest_observation_and_new_step_invalidates_it() {
        let runtime=ExecutionRuntime::start(8,16).unwrap();
        let handle=runtime.handle();
        handle.apply(RuntimeCommand::Activate{task_id:"task-goal".into(),checkpoint:0}).unwrap();
        handle.apply(start_step("task-goal","open","打开目标")).unwrap();
        let observed=handle.apply(RuntimeCommand::CompleteStep{task_id:"task-goal".into(),step_id:"open".into(),action_succeeded:true}).unwrap();
        let observation_sequence=observed.last_observation_sequence.unwrap();
        let boundary=handle.apply(RuntimeCommand::AdvanceStepBoundary{task_id:"task-goal".into(),step_id:"open".into()}).unwrap();
        assert!(handle.apply(RuntimeCommand::VerifyGoal{task_id:"task-goal".into(),verification_id:"verify-old".into(),observation_sequence:observation_sequence.saturating_sub(1),outcome:crate::GoalVerificationOutcome::Achieved}).is_err());
        let verified=handle.apply(RuntimeCommand::VerifyGoal{task_id:"task-goal".into(),verification_id:"verify-goal".into(),observation_sequence,outcome:crate::GoalVerificationOutcome::Achieved}).unwrap();
        assert_eq!(verified.goal_verification.as_ref().unwrap().verified_sequence,verified.sequence);
        handle.apply(start_step("task-goal","continue","继续执行")).unwrap();
        assert!(handle.snapshot("task-goal").unwrap().goal_verification.is_none());
        assert!(boundary.sequence < verified.sequence);

        handle.apply(RuntimeCommand::Activate{task_id:"task-replan".into(),checkpoint:0}).unwrap();
        handle.apply(start_step("task-replan","inspect","检查目标")).unwrap();
        let observed=handle.apply(RuntimeCommand::CompleteStep{task_id:"task-replan".into(),step_id:"inspect".into(),action_succeeded:true}).unwrap();
        let observation_sequence=observed.last_observation_sequence.unwrap();
        handle.apply(RuntimeCommand::AdvanceStepBoundary{task_id:"task-replan".into(),step_id:"inspect".into()}).unwrap();
        let not_achieved=handle.apply(RuntimeCommand::VerifyGoal{task_id:"task-replan".into(),verification_id:"verify-replan".into(),observation_sequence,outcome:crate::GoalVerificationOutcome::NotAchieved}).unwrap();
        assert_eq!(not_achieved.phase,RuntimePhase::HandBack);
        assert_eq!(not_achieved.goal_verification.unwrap().outcome,crate::GoalVerificationOutcome::NotAchieved);

        handle.apply(RuntimeCommand::Activate{task_id:"task-handback-achieved".into(),checkpoint:0}).unwrap();
        handle.apply(start_step("task-handback-achieved","play","激活播放")).unwrap();
        let observed=handle.apply(RuntimeCommand::CompleteStep{task_id:"task-handback-achieved".into(),step_id:"play".into(),action_succeeded:true}).unwrap();
        let observation_sequence=observed.last_observation_sequence.unwrap();
        handle.apply(RuntimeCommand::AdvanceStepBoundary{task_id:"task-handback-achieved".into(),step_id:"play".into()}).unwrap();
        handle.apply(RuntimeCommand::HandBack{task_id:"task-handback-achieved".into(),reason:"快脑无法解释开放业务语义".into()}).unwrap();
        let achieved=handle.apply(RuntimeCommand::VerifyGoal{task_id:"task-handback-achieved".into(),verification_id:"verify-playing".into(),observation_sequence,outcome:crate::GoalVerificationOutcome::Achieved}).unwrap();
        assert_eq!(achieved.phase,RuntimePhase::Running);
        assert_eq!(achieved.goal_verification.unwrap().outcome,crate::GoalVerificationOutcome::Achieved);

        let paused=handle.apply(RuntimeCommand::StopForControl{task_id:"task-handback-achieved".into(),control_kind:"pause".into()}).unwrap();
        assert_eq!(paused.phase,RuntimePhase::Paused);
        assert!(handle.apply(RuntimeCommand::VerifyGoal{task_id:"task-handback-achieved".into(),verification_id:"verify-paused".into(),observation_sequence,outcome:crate::GoalVerificationOutcome::Achieved}).is_err());
    }
}
