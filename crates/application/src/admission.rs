//! 可信执行器的进程内资源准入；不保存任务状态，不承担路径身份解析。
use std::sync::Mutex;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Resource {
    Desktop,
    Browser,
    /// 必须来自可信 Adapter 的规范化身份，不接受请求中的原始路径。
    File(String),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Denied { InvalidInput, Capacity, DuplicateTask, ResourceBusy(Resource), DesktopTakenOver, PresentationBusy, Unavailable }

struct Occupancy { task_id: String, resources: Vec<Resource> }
struct State { occupied: Vec<Occupancy>, desktop_taken_over: bool, rest_reserved: bool }

pub struct Admission { capacity: usize, state: Mutex<State> }

/// 丢弃不会释放：超时或 unknown 不能被误判为执行已停止。
#[must_use = "只有确认执行停止后才能显式释放资源"]
pub struct Permit<'a> { admission: &'a Admission, task_id: String }

/// 必须持有至确认展开；丢弃不会放行新执行。
#[must_use = "确认展开或从未开始收起后才能释放预约"]
pub struct RestPermit<'a> { admission: &'a Admission }

#[derive(Debug, Eq, PartialEq)]
pub enum RestDenied { Busy, Unavailable }

/// 仅供完成单实例恢复的可信宿主；成功后才能开始收起。
pub fn reserve_rest<'a>(store: &mut impl crate::TaskStore, admission: Option<&'a Admission>) -> Result<RestPermit<'a>, RestDenied> {
    let admission = admission.ok_or(RestDenied::Unavailable)?;
    let permit = admission.reserve_rest_slot()?;
    match crate::running_state(store) {
        crate::RunningState::NoRunningTask => Ok(permit),
        state => {
            // 尚未交付展示凭证，不可能由本次调用开始收起，可以安全撤销。
            permit.release_after_wake().map_err(|_| RestDenied::Unavailable)?;
            Err(if state == crate::RunningState::Running { RestDenied::Busy } else { RestDenied::Unavailable })
        }
    }
}

impl RestPermit<'_> {
    /// 窗口操作失败且无法确认展开时不得调用；不能在新任务请求到达时直接释放。
    pub fn release_after_wake(self) -> Result<(), Denied> {
        self.admission.state.lock().map_err(|_| Denied::Unavailable)?.rest_reserved = false;
        Ok(())
    }
}

pub enum Outcome { Completed, Failed }

pub enum FinishError<'a> {
    /// 状态未成功提交，占用仍在；交还凭证供调用方处理，不自动重试。
    Task { error: crate::Error, permit: Permit<'a> },
    /// 终态已经提交，不可再次提交同一迁移；资源仍保守占用。
    Release { task: crate::Task, error: Denied },
}

pub enum BoundaryStopError<'a> {
    Task { error: crate::Error, permit: Permit<'a> },
    Release { task: crate::Task, stop: crate::StopRecord, error: Denied },
}

#[derive(Debug, Eq, PartialEq)]
pub enum StartError {
    Admission(Denied),
    Task(crate::Error),
    Cleanup { task_error: crate::Error, release_error: Denied },
}

/// 仅供可信宿主；返回成功前不得派发，返回后仍需检查接管/取消/截止时间。
/// 只启动 created 任务，恢复必须走显式 Resume 流程。
pub fn start<'a>(store: &mut impl crate::TaskStore, admission: &'a Admission, task_id: &str, expected_sequence: u64, resources: &[Resource]) -> Result<(crate::Task, Permit<'a>), StartError> {
    let permit = admission.try_acquire(task_id, resources).map_err(StartError::Admission)?;
    match crate::transition(store, task_id, expected_sequence, crate::Action::Start) {
        Ok(task) => Ok((task, permit)),
        Err(task_error) => match permit.release_after_stop() {
            Ok(()) => Err(StartError::Task(task_error)),
            Err(release_error) => Err(StartError::Cleanup { task_error, release_error }),
        },
    }
}

/// TM-S2：原子持久化完整尝试身份后才允许调用方派发。
pub fn start_attempt<'a>(store: &mut impl crate::TaskStore, admission: &'a Admission, attempt: &crate::ExecutionAttempt, expected_sequence: u64, resources: &[Resource]) -> Result<(crate::Task, crate::ExecutionAttempt, Permit<'a>), StartError> {
    let permit = admission.try_acquire(&attempt.task_id, resources).map_err(StartError::Admission)?;
    match crate::prepare_attempt(store, attempt, expected_sequence) {
        Ok((task, attempt)) => Ok((task, attempt, permit)),
        Err(task_error) => match permit.release_after_stop() {
            Ok(()) => Err(StartError::Task(task_error)),
            Err(release_error) => Err(StartError::Cleanup { task_error, release_error }),
        },
    }
}

/// CUA/BUA/Document/Command 的唯一启动边界；资源由各能力声明。
pub fn start_execution(store: &mut impl crate::TaskStore, admission: &Admission, task: &crate::Task, attempt: &crate::ExecutionAttempt, expected_sequence: u64, resources: &[Resource]) -> Result<(crate::Task, crate::ExecutionAttempt), StartError> {
    match task.status {
        crate::Status::Created => start_attempt(store, admission, attempt, expected_sequence, resources).map(|(task, attempt, _)| (task, attempt)),
        crate::Status::Running => match admission.holds(&attempt.task_id) {
            Ok(true) => crate::prepare_next_attempt(store, attempt, expected_sequence).map_err(StartError::Task),
            Ok(false) => Err(StartError::Task(crate::Error::StopRequired)),
            Err(_) => Err(StartError::Task(crate::Error::StorageUnavailable)),
        },
        _ => Err(StartError::Task(crate::Error::StopRequired)),
    }
}

impl Admission {
    /// 组合根取得单实例所有权并完成恢复后，才允许创建并使用唯一实例。
    pub fn new(capacity: usize) -> Result<Self, Denied> {
        if !(1..=64).contains(&capacity) { return Err(Denied::InvalidInput); }
        Ok(Self { capacity, state: Mutex::new(State { occupied: Vec::new(), desktop_taken_over: false, rest_reserved: false }) })
    }

    pub fn try_acquire(&self, task_id: &str, resources: &[Resource]) -> Result<Permit<'_>, Denied> {
        if !yonder_protocol::valid_id(task_id) || resources.len() > 32 || resources.iter().any(|r| matches!(r, Resource::File(id) if id.is_empty() || id.len() > 4096 || id.contains('\0'))) {
            return Err(Denied::InvalidInput);
        }
        let mut state = self.state.lock().map_err(|_| Denied::Unavailable)?;
        if state.rest_reserved { return Err(Denied::PresentationBusy); }
        if state.occupied.iter().any(|entry| entry.task_id == task_id) { return Err(Denied::DuplicateTask); }
        if state.desktop_taken_over && resources.contains(&Resource::Desktop) { return Err(Denied::DesktopTakenOver); }
        // ponytail: 至多 64 个占用、每个 32 个资源；上限提高后再考虑索引。
        for resource in resources {
            if state.occupied.iter().any(|entry| entry.resources.contains(resource)) {
                return Err(Denied::ResourceBusy(resource.clone()));
            }
        }
        if state.occupied.len() >= self.capacity { return Err(Denied::Capacity); }
        state.occupied.push(Occupancy { task_id: task_id.into(), resources: resources.to_vec() });
        Ok(Permit { admission: self, task_id: task_id.into() })
    }

    fn reserve_rest_slot(&self) -> Result<RestPermit<'_>, RestDenied> {
        let mut state = self.state.lock().map_err(|_| RestDenied::Unavailable)?;
        if state.rest_reserved || !state.occupied.is_empty() { return Err(RestDenied::Busy); }
        state.rest_reserved = true;
        Ok(RestPermit { admission: self })
    }

    /// 任意未确认停止的执行占用，包含空资源后台任务；不缓存任务状态。
    pub fn has_occupancy(&self) -> Result<bool, Denied> {
        Ok(!self.state.lock().map_err(|_| Denied::Unavailable)?.occupied.is_empty())
    }

    pub fn has_resource(&self, resource: Resource) -> Result<bool, Denied> {
        Ok(self.state.lock().map_err(|_| Denied::Unavailable)?.occupied.iter().any(|entry| entry.resources.contains(&resource)))
    }

    pub fn holds(&self,task_id:&str)->Result<bool,Denied>{Ok(self.state.lock().map_err(|_|Denied::Unavailable)?.occupied.iter().any(|entry|entry.task_id==task_id))}
    pub fn holds_resource(&self,task_id:&str,resource:Resource)->Result<bool,Denied>{Ok(self.state.lock().map_err(|_|Denied::Unavailable)?.occupied.iter().any(|entry|entry.task_id==task_id&&entry.resources.contains(&resource)))}

    pub fn release_task_after_stop(&self,task_id:&str)->Result<(),Denied>{
        let mut state=self.state.lock().map_err(|_|Denied::Unavailable)?;
        if !state.occupied.iter().any(|entry|entry.task_id==task_id){return Err(Denied::InvalidInput);}
        state.occupied.retain(|entry|entry.task_id!=task_id); Ok(())
    }

    /// 仅阻断新桌面准入；宿主仍须暂停既有桌面执行，归还必须来自显式用户操作。
    pub fn set_desktop_taken_over(&self, taken_over: bool) -> Result<(), Denied> {
        self.state.lock().map_err(|_| Denied::Unavailable)?.desktop_taken_over = taken_over;
        Ok(())
    }
}

impl<'a> Permit<'a> {
    /// observed边界先提交停止事实，再释放占用；失败交还Permit。
    pub fn stop_at_boundary(self, store: &mut impl crate::TaskStore, attempt_id: &str, kind: crate::ControlKind) -> Result<(crate::Task, crate::StopRecord), BoundaryStopError<'a>> {
        let (task, stop) = match crate::stop_at_boundary(store,&self.task_id,attempt_id,kind) {
            Ok(value) => value,
            Err(error) => return Err(BoundaryStopError::Task { error, permit:self }),
        };
        match self.release_after_stop() {
            Ok(()) => Ok((task,stop)),
            Err(error) => Err(BoundaryStopError::Release { task,stop,error }),
        }
    }

    /// 仅在执行已停止且结果确定后调用；超时/unknown 不是 Failed。
    pub fn finish_after_stop(self, store: &mut impl crate::TaskStore, expected_sequence: u64, outcome: Outcome) -> Result<crate::Task, FinishError<'a>> {
        let action = match outcome { Outcome::Completed => crate::Action::Complete, Outcome::Failed => crate::Action::Fail };
        let task = match crate::transition(store, &self.task_id, expected_sequence, action) {
            Ok(task) => task,
            Err(error) => return Err(FinishError::Task { error, permit: self }),
        };
        match self.release_after_stop() {
            Ok(()) => Ok(task),
            Err(error) => Err(FinishError::Release { task, error }),
        }
    }

    /// 调用方必须已确认执行停止，或确认从未派发；不能在取消请求时调用。
    pub fn release_after_stop(self) -> Result<(), Denied> {
        let mut state = self.admission.state.lock().map_err(|_| Denied::Unavailable)?;
        state.occupied.retain(|entry| entry.task_id != self.task_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resources_are_atomic_bounded_and_only_released_after_stop() {
        assert!(Admission::new(0).is_err());
        assert!(Admission::new(65).is_err());
        let gate = Admission::new(3).unwrap();
        let file = Resource::File("volume-1-file-1".into());
        let first = gate.try_acquire("first", &[file.clone()]).unwrap();
        assert_eq!(gate.try_acquire("first", &[]).err(), Some(Denied::DuplicateTask));
        assert_eq!(gate.try_acquire("second", &[Resource::Browser, file.clone()]).err(), Some(Denied::ResourceBusy(file.clone())));
        let browser = gate.try_acquire("browser", &[Resource::Browser]).unwrap(); // 失败未占用部分资源。
        assert_eq!(gate.try_acquire("browser-2", &[Resource::Browser]).err(), Some(Denied::ResourceBusy(Resource::Browser)));
        let other = gate.try_acquire("other", &[Resource::File("volume-1-file-2".into())]).unwrap();
        assert_eq!(gate.try_acquire("full", &[]).err(), Some(Denied::Capacity));
        other.release_after_stop().unwrap();
        browser.release_after_stop().unwrap();
        gate.set_desktop_taken_over(true).unwrap();
        assert_eq!(gate.try_acquire("desktop", &[Resource::Desktop]).err(), Some(Denied::DesktopTakenOver));
        gate.try_acquire("background", &[]).unwrap().release_after_stop().unwrap();
        gate.set_desktop_taken_over(false).unwrap();
        let desktop = gate.try_acquire("desktop", &[Resource::Desktop]).unwrap();
        assert_eq!(gate.try_acquire("desktop-2", &[Resource::Desktop]).err(), Some(Denied::ResourceBusy(Resource::Desktop)));
        desktop.release_after_stop().unwrap();
        first.release_after_stop().unwrap();
        drop(gate.try_acquire("unknown", &[file.clone()]).unwrap());
        assert_eq!(gate.try_acquire("retry", &[file.clone()]).err(), Some(Denied::ResourceBusy(file)));
        for (id, resources) in [("../bad", vec![]), ("bad", vec![Resource::File(String::new())]), ("bad", vec![Resource::Desktop; 33])] {
            assert_eq!(gate.try_acquire(id, &resources).err(), Some(Denied::InvalidInput));
        }
    }

    #[test]
    fn poisoned_occupancy_is_unavailable_not_empty() {
        let gate = Admission::new(1).unwrap();
        assert_eq!(gate.has_occupancy(), Ok(false));
        let _ = std::panic::catch_unwind(|| {
            let _guard = gate.state.lock().unwrap();
            panic!("合成准入故障");
        });
        assert_eq!(gate.has_occupancy(), Err(Denied::Unavailable));
        assert_eq!(gate.reserve_rest_slot().err(), Some(RestDenied::Unavailable));
    }

    #[test]
    fn rest_and_execution_compete_atomically_and_dropped_rest_stays_reserved() {
        for _ in 0..32 {
            let gate = Admission::new(1).unwrap();
            let barrier = std::sync::Barrier::new(2);
            std::thread::scope(|scope| {
                let rest = scope.spawn(|| {
                    barrier.wait();
                    let permit = gate.reserve_rest_slot();
                    barrier.wait(); // 双方都申请完才允许释放。
                    permit
                });
                barrier.wait();
                let execution = gate.try_acquire("task", &[]);
                barrier.wait();
                let rest = rest.join().unwrap();
                assert_ne!(rest.is_ok(), execution.is_ok());
                if let Ok(permit) = rest {
                    assert_eq!(execution.err(), Some(Denied::PresentationBusy));
                    assert_eq!(gate.reserve_rest_slot().err(), Some(RestDenied::Busy));
                    permit.release_after_wake().unwrap();
                } else { execution.unwrap().release_after_stop().unwrap(); }
            });
            gate.try_acquire("after", &[]).unwrap().release_after_stop().unwrap();
        }
        let gate = Admission::new(1).unwrap();
        drop(gate.reserve_rest_slot().unwrap());
        assert_eq!(gate.try_acquire("blocked", &[]).err(), Some(Denied::PresentationBusy));
    }

    #[test]
    fn competing_threads_cannot_exceed_capacity_or_share_desktop() {
        for desktop in [false, true] {
            let gate = Admission::new(3).unwrap();
            let barrier = std::sync::Barrier::new(8);
            let admitted = std::sync::atomic::AtomicUsize::new(0);
            std::thread::scope(|scope| {
                for index in 0..8 {
                    let (gate, barrier, admitted) = (&gate, &barrier, &admitted);
                    scope.spawn(move || {
                        barrier.wait();
                        let resources = if desktop { vec![Resource::Desktop] } else { vec![Resource::File(format!("file-{index}"))] };
                        let permit = gate.try_acquire(&format!("task-{index}"), &resources);
                        if permit.is_ok() { admitted.fetch_add(1, std::sync::atomic::Ordering::SeqCst); }
                        barrier.wait(); // 所有人申请完成前不释放。
                        if let Ok(permit) = permit { permit.release_after_stop().unwrap(); }
                    });
                }
            });
            assert_eq!(admitted.load(std::sync::atomic::Ordering::SeqCst), if desktop { 1 } else { 3 });
            gate.try_acquire("after", &[Resource::Desktop]).unwrap().release_after_stop().unwrap();
        }
    }
}
