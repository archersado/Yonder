//! 活动执行的单一内存状态所有者。SQLite projector 只消费这里产生的连续事件，
//! 不参与 Driver、Observe 或下一步决策。
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError},
        Arc,
    },
    thread,
    time::Duration,
};

pub const DEFAULT_COMMAND_CAPACITY: usize = 128;
pub const DEFAULT_EVENT_HARD_LIMIT: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimePhase {
    Running,
    HandBack,
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
    pub phase: RuntimeStepPhase,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RuntimeSnapshot {
    pub task_id: String,
    pub sequence: u64,
    pub checkpoint: u64,
    pub phase: RuntimePhase,
    pub current_step: Option<RuntimeStep>,
    pub pending_events: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum RuntimeEventKind {
    Activated,
    StepStarted { step_id: String, label: String },
    StepCompleted { step_id: String },
    StepUnverified { step_id: String, reason: String },
    HandedBack { reason: String },
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
    },
    CompleteStep {
        task_id: String,
        step_id: String,
    },
    UnverifyStep {
        task_id: String,
        step_id: String,
        reason: String,
    },
    HandBack {
        task_id: String,
        reason: String,
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
                while !worker_stop.load(Ordering::Acquire) {
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
                    project_dirty(&runtime, &mut projector, &mut dirty, batch_size);
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
        if projector.project(&events).is_ok()
            && runtime
                .apply(RuntimeCommand::Ack {
                    task_id: task_id.clone(),
                    through_sequence,
                })
                .is_ok()
        {
            if events.len() < batch_size {
                dirty.remove(&task_id);
            }
        }
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
                RuntimeCommand::StartStep { step_id, label, .. } => {
                    if !valid_id(&step_id)
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
                    task.snapshot.current_step = Some(RuntimeStep {
                        step_id: step_id.clone(),
                        label: label.clone(),
                        phase: RuntimeStepPhase::Executing,
                    });
                    append(task, RuntimeEventKind::StepStarted { step_id, label })?;
                }
                RuntimeCommand::CompleteStep { step_id, .. } => {
                    require_executing(task, &step_id)?;
                    task.snapshot.current_step.as_mut().unwrap().phase =
                        RuntimeStepPhase::Completed;
                    append(task, RuntimeEventKind::StepCompleted { step_id })?;
                }
                RuntimeCommand::UnverifyStep {
                    step_id, reason, ..
                } => {
                    if reason.is_empty() || reason.len() > 120 {
                        return Err(RuntimeError::InvalidInput);
                    }
                    require_executing(task, &step_id)?;
                    task.snapshot.current_step.as_mut().unwrap().phase =
                        RuntimeStepPhase::Unverified;
                    append(task, RuntimeEventKind::StepUnverified { step_id, reason })?;
                }
                RuntimeCommand::HandBack { reason, .. } => {
                    if reason.is_empty() || reason.len() > 160 {
                        return Err(RuntimeError::InvalidInput);
                    }
                    task.snapshot.phase = RuntimePhase::HandBack;
                    append(task, RuntimeEventKind::HandedBack { reason })?;
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
        | RuntimeCommand::HandBack { task_id, .. }
        | RuntimeCommand::Terminate { task_id, .. }
        | RuntimeCommand::Ack { task_id, .. } => task_id,
    }
}
fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
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
            .apply(RuntimeCommand::StartStep {
                task_id: "task-1".into(),
                step_id: "open".into(),
                label: "打开应用".into(),
            })
            .unwrap();
        let completed = handle
            .apply(RuntimeCommand::CompleteStep {
                task_id: "task-1".into(),
                step_id: "open".into(),
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
            .apply(RuntimeCommand::StartStep {
                task_id: "task-1".into(),
                step_id: "one".into(),
                label: "第一步".into(),
            })
            .unwrap();
        let result = handle
            .apply(RuntimeCommand::CompleteStep {
                task_id: "task-1".into(),
                step_id: "one".into(),
            })
            .unwrap();
        assert_eq!(result.pending_events, 3);
        assert_eq!(
            handle.apply(RuntimeCommand::StartStep {
                task_id: "task-1".into(),
                step_id: "two".into(),
                label: "第二步".into()
            }),
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
            .apply(RuntimeCommand::StartStep {
                task_id: "task-1".into(),
                step_id: "one".into(),
                label: "第一步".into(),
            })
            .unwrap();
        assert!(handle
            .apply(RuntimeCommand::StartStep {
                task_id: "task-1".into(),
                step_id: "two".into(),
                label: "第二步".into()
            })
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
            .apply(RuntimeCommand::StartStep {
                task_id: "task-1".into(),
                step_id: "observe".into(),
                label: "观察".into(),
            })
            .unwrap();
        handle
            .apply(RuntimeCommand::UnverifyStep {
                task_id: "task-1".into(),
                step_id: "observe".into(),
                reason: "screenshot-unavailable".into(),
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
            .apply(RuntimeCommand::StartStep {
                task_id: "task-1".into(),
                step_id: "open".into(),
                label: "打开应用".into(),
            })
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
}
