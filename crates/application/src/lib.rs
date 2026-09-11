//! 任务用例与事务存储 Port；不依赖具体 Adapter。
pub use yonder_domain::{Action, Status, Transition, TransitionError};
pub mod query;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Task {
    pub id: String,
    pub status: Status,
    pub sequence: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    NotFound,
    Conflict,
    InvalidInput,
    InvalidTransition(TransitionError),
    StorageUnavailable,
}

pub trait TaskStore {
    fn create(&mut self, id: &str) -> Result<Task, Error>;
    fn get(&mut self, id: &str) -> Result<Task, Error>;

    /// 启动恢复专用：按 id 升序读取至多 limit 个 running 快照。
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

pub fn get(store: &mut impl TaskStore, id: &str) -> Result<Task, Error> {
    validate_id(id)?;
    store.get(id)
}

pub fn create(store: &mut impl TaskStore, id: &str) -> Result<Task, Error> {
    validate_id(id)?;
    store.create(id)
}

pub fn events(store: &mut impl TaskStore, id: &str, after: u64, limit: usize) -> Result<Vec<Transition>, Error> {
    validate_id(id)?;
    if !(1..=100).contains(&limit) {
        return Err(Error::InvalidInput);
    }
    store.get(id)?;
    store.events(id, after, limit)
}

pub fn transition(store: &mut impl TaskStore, id: &str, expected_sequence: u64, action: Action) -> Result<Task, Error> {
    let task = get(store, id)?;
    if task.sequence != expected_sequence {
        return Err(Error::Conflict);
    }
    let change = task.status.transition(task.sequence, action).map_err(Error::InvalidTransition)?;
    store.commit(id, expected_sequence, change)?;
    Ok(Task { id: task.id, status: change.next, sequence: change.sequence })
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
        fn create(&mut self, _: &str) -> Result<Task, Error> { Err(Error::Conflict) }
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
            task: Task { id: "task-1".into(), status: Status::Created, sequence: 1 },
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
