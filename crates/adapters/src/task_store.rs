use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use std::{path::Path, time::Duration};
use yonder_application::{Action, Error, Status, Task, TaskStore, Transition};

pub struct SqlCipherTaskStore(Connection);

fn storage(_: rusqlite::Error) -> Error { Error::StorageUnavailable }

fn name(status: Status) -> &'static str {
    match status {
        Status::Created => "created", Status::Running => "running",
        Status::WaitingForUser => "waiting-for-user", Status::Paused => "paused",
        Status::Interrupted => "interrupted", Status::Completed => "completed",
        Status::Failed => "failed", Status::Cancelled => "cancelled",
    }
}

fn status(value: &str) -> Result<Status, Error> {
    match value {
        "created" => Ok(Status::Created), "running" => Ok(Status::Running),
        "waiting-for-user" => Ok(Status::WaitingForUser), "paused" => Ok(Status::Paused),
        "interrupted" => Ok(Status::Interrupted), "completed" => Ok(Status::Completed),
        "failed" => Ok(Status::Failed), "cancelled" => Ok(Status::Cancelled),
        _ => Err(Error::StorageUnavailable),
    }
}

impl SqlCipherTaskStore {
    pub fn open(path: &Path, database_key: &[u8; 32]) -> Result<Self, Error> {
        let mut db = Connection::open(path).map_err(storage)?;
        // SAFETY: 连接句柄有效；同步调用期间 key 的 32 字节保持存活。
        let result = unsafe {
            rusqlite::ffi::sqlite3_key(db.handle(), database_key.as_ptr().cast(), 32)
        };
        if result != rusqlite::ffi::SQLITE_OK { return Err(Error::StorageUnavailable); }
        let version: String = db.query_row("PRAGMA cipher_version", [], |r| r.get(0)).map_err(storage)?;
        if version.is_empty() { return Err(Error::StorageUnavailable); }
        db.busy_timeout(Duration::from_secs(2)).map_err(storage)?;
        db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;").map_err(storage)?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let schema: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(storage)?;
        if schema == 0 {
            let objects: i64 = tx.query_row("SELECT count(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'", [], |r| r.get(0)).map_err(storage)?;
            if objects != 0 { return Err(Error::StorageUnavailable); }
            tx.execute_batch(include_str!("task_schema.sql")).map_err(storage)?;
        } else if schema != 1 { return Err(Error::StorageUnavailable); }
        tx.commit().map_err(storage)?;
        Ok(Self(db))
    }
}

impl TaskStore for SqlCipherTaskStore {
    fn create(&mut self, id: &str) -> Result<Task, Error> {
        if id.is_empty() || id.len() > 128 || !id.bytes().all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c)) {
            return Err(Error::InvalidInput);
        }
        let tx = self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM tasks WHERE id=?1)", [id], |r| r.get(0)).map_err(storage)?;
        if exists { return Err(Error::Conflict); }
        tx.execute("INSERT INTO tasks VALUES (?1,'created',1)", [id]).map_err(storage)?;
        tx.execute("INSERT INTO events VALUES (?1,1,'created','created')", [id]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,1)", [id]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Task { id: id.into(), status: Status::Created, sequence: 1 })
    }

    fn get(&mut self, id: &str) -> Result<Task, Error> {
        let row: Option<(String,i64)> = self.0.query_row("SELECT state,sequence FROM tasks WHERE id=?1", [id], |r| Ok((r.get(0)?,r.get(1)?))).optional().map_err(storage)?;
        let (state, sequence) = row.ok_or(Error::NotFound)?;
        Ok(Task { id: id.into(), status: status(&state)?, sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? })
    }

    fn events(&mut self, id: &str, after: u64, limit: usize) -> Result<Vec<Transition>, Error> {
        if !(1..=100).contains(&limit) { return Err(Error::InvalidInput); }
        if after >= i64::MAX as u64 { return Ok(vec![]); }
        let mut stmt = self.0.prepare("SELECT previous,state,sequence FROM events WHERE task_id=?1 AND sequence>?2 ORDER BY sequence LIMIT ?3").map_err(storage)?;
        let rows = stmt.query_map(params![id, after as i64, limit as i64], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?))).map_err(storage)?;
        rows.map(|row| {
            let (previous,next,sequence) = row.map_err(storage)?;
            Ok(Transition { previous: status(&previous)?, next: status(&next)?, sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? })
        }).collect()
    }

    fn running(&mut self, limit: usize) -> Result<Vec<Task>, Error> {
        if !(1..=100).contains(&limit) { return Err(Error::InvalidInput); }
        let mut stmt = self.0.prepare("SELECT id,sequence FROM tasks WHERE state='running' ORDER BY id LIMIT ?1").map_err(storage)?;
        let rows = stmt.query_map([limit as i64], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?))).map_err(storage)?;
        rows.map(|row| {
            let (id, sequence) = row.map_err(storage)?;
            Ok(Task { id, status: Status::Running, sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? })
        }).collect()
    }

    fn commit(&mut self, id: &str, expected: u64, change: Transition) -> Result<(), Error> {
        if expected == 0 || expected >= i64::MAX as u64 || change.sequence != expected + 1 {
            return Err(Error::InvalidInput);
        }
        let legal = [Action::Start, Action::Pause, Action::WaitForUser, Action::Resume, Action::Interrupt, Action::Complete, Action::Fail, Action::Cancel]
            .iter().any(|action| change.previous.transition(expected, *action) == Ok(change));
        if !legal { return Err(Error::InvalidInput); }
        let tx = self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let changed = tx.execute("UPDATE tasks SET state=?1,sequence=?2 WHERE id=?3 AND sequence=?4 AND state=?5", params![name(change.next), change.sequence as i64, id, expected as i64, name(change.previous)]).map_err(storage)?;
        if changed != 1 { return Err(Error::Conflict); }
        tx.execute("INSERT INTO events VALUES (?1,?2,?3,?4)", params![id,change.sequence as i64,name(change.previous),name(change.next)]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)", params![id,change.sequence as i64]).map_err(storage)?;
        tx.commit().map_err(storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yonder_application::{Action, create, transition};

    #[test]
    fn restart_recovery_is_bounded_repeatable_and_preserves_atomic_events() {
        use yonder_application::recover_running;
        let directory = std::env::temp_dir().join(format!("yonder-recovery-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let key = [19;32]; // 仅合成测试库使用。
        let mut store = SqlCipherTaskStore::open(&path, &key).unwrap();
        for id in ["a", "b", "c"] {
            create(&mut store, id).unwrap();
            transition(&mut store, id, 1, Action::Start).unwrap();
        }
        let mut unchanged = vec![create(&mut store, "created").unwrap()];
        for (id, action) in [("paused", Action::Pause), ("waiting", Action::WaitForUser), ("interrupted", Action::Interrupt), ("completed", Action::Complete), ("failed", Action::Fail), ("cancelled", Action::Cancel)] {
            create(&mut store, id).unwrap();
            transition(&mut store, id, 1, Action::Start).unwrap();
            unchanged.push(transition(&mut store, id, 2, action).unwrap());
        }
        drop(store);
        let mut store = SqlCipherTaskStore::open(&path, &key).unwrap();
        // 普通连接不会误中断运行中任务；恢复由启动用例显式发起。
        assert_eq!(store.running(100).unwrap().len(), 3);
        assert_eq!(recover_running(&mut store, 0), Err(Error::InvalidInput));
        assert_eq!(recover_running(&mut store, 101), Err(Error::InvalidInput));
        store.0.execute_batch("CREATE TRIGGER fail_recovery BEFORE INSERT ON outbox WHEN NEW.task_id='b' AND NEW.sequence=3 BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert_eq!(recover_running(&mut store, 100), Err(Error::StorageUnavailable));
        assert_eq!(store.get("a").unwrap().status, Status::Interrupted);
        for id in ["b", "c"] {
            assert_eq!(store.get(id).unwrap().status, Status::Running);
            assert_eq!(store.events(id, 0, 100).unwrap().len(), 2);
            let count: i64 = store.0.query_row("SELECT count(*) FROM outbox WHERE task_id=?1", [id], |r| r.get(0)).unwrap();
            assert_eq!(count, 2);
        }
        store.0.execute_batch("DROP TRIGGER fail_recovery;").unwrap();
        assert_eq!(recover_running(&mut store, 1), Ok(1));
        assert_eq!(store.get("c").unwrap().status, Status::Running);
        assert_eq!(recover_running(&mut store, 1), Ok(1));
        assert_eq!(recover_running(&mut store, 100), Ok(0));
        drop(store);
        let mut store = SqlCipherTaskStore::open(&path, &key).unwrap();
        assert_eq!(recover_running(&mut store, 100), Ok(0));
        for id in ["a", "b", "c"] {
            assert_eq!(store.get(id).unwrap(), Task { id: id.into(), status: Status::Interrupted, sequence: 3 });
            assert_eq!(store.events(id, 2, 100).unwrap(), vec![Status::Running.transition(2, Action::Interrupt).unwrap()]);
            let count: i64 = store.0.query_row("SELECT count(*) FROM outbox WHERE task_id=?1", [id], |r| r.get(0)).unwrap();
            assert_eq!(count, 3);
        }
        for snapshot in unchanged {
            assert_eq!(store.get(&snapshot.id).unwrap(), snapshot);
            assert_eq!(store.events(&snapshot.id, snapshot.sequence, 100).unwrap(), vec![]);
        }
        drop(store);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn encrypted_transactions_reject_stale_writes_and_rollback_outbox_failure() {
        let directory = std::env::temp_dir().join(format!("yonder-store-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let key = [17;32]; // 仅合成测试库使用。
        {
            let mut first = SqlCipherTaskStore::open(&path, &key).unwrap();
            assert_eq!(first.get("missing"), Err(Error::NotFound));
            create(&mut first, "task-1").unwrap();
            assert_eq!(create(&mut first, "task-1"), Err(Error::Conflict));
            let mut second = SqlCipherTaskStore::open(&path, &key).unwrap();
            let stale = second.get("task-1").unwrap();
            transition(&mut first, "task-1", 1, Action::Start).unwrap();
            // JSON 查询经过同一 Application 和真实 SQLCipher，不引入内存状态源。
            use yonder_protocol::{QueryResult, Response, TaskStatus};
            let get = br#"{"jsonrpc":"2.0","id":"r1","method":"task.get","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"task-1"}}"#;
            match yonder_application::query::handle(&mut second, get, 1000) {
                Response::Success { id, result: QueryResult::Snapshot { task }, .. } => {
                    assert_eq!(id, "r1"); assert_eq!(task.status, TaskStatus::Running); assert_eq!(task.sequence, "2");
                }
                response => panic!("查询响应不匹配：{response:?}"),
            }
            let events = br#"{"jsonrpc":"2.0","id":"r2","method":"task.events","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"task-1","after_sequence":"1","limit":1}}"#;
            match yonder_application::query::handle(&mut second, events, 1000) {
                Response::Success { id, result: QueryResult::Events { events, .. }, .. } => {
                    assert_eq!(id, "r2"); assert_eq!(events.len(), 1); assert_eq!(events[0].sequence, "2");
                }
                response => panic!("事件响应不匹配：{response:?}"),
            }
            match yonder_application::query::handle(&mut second, get, 2000) {
                Response::Failure { id, error, .. } => { assert_eq!(id.as_deref(), Some("r1")); assert_eq!(error.code, -32001); }
                response => panic!("过期请求未被拒绝：{response:?}"),
            }
            let stale_change = stale.status.transition(stale.sequence, Action::Start).unwrap();
            assert_eq!(second.commit("task-1", 1, stale_change), Err(Error::Conflict));
            first.0.execute_batch("CREATE TRIGGER fail_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
            assert_eq!(transition(&mut first, "task-1", 2, Action::Complete), Err(Error::StorageUnavailable));
            assert_eq!(second.get("task-1").unwrap().status, Status::Running);
            assert_eq!(second.events("task-1", 0, 100).unwrap().len(), 2);
            let count: i64 = second.0.query_row("SELECT count(*) FROM outbox", [], |r| r.get(0)).unwrap();
            assert_eq!(count, 2);
            first.0.execute_batch("DROP TRIGGER fail_outbox;").unwrap();
            transition(&mut first, "task-1", 2, Action::Interrupt).unwrap();
        }
        let mut reopened = SqlCipherTaskStore::open(&path, &key).unwrap();
        assert_eq!(reopened.get("task-1").unwrap().status, Status::Interrupted);
        assert_eq!(reopened.events("task-1", 2, 1).unwrap()[0].sequence, 3);
        assert!(reopened.events("task-1", u64::MAX, 1).unwrap().is_empty());
        drop(reopened);
        assert!(!std::fs::read(&path).unwrap().starts_with(b"SQLite format 3"));
        assert!(SqlCipherTaskStore::open(&path, &[18;32]).is_err());
        let opened = SqlCipherTaskStore::open(&path, &key).unwrap();
        opened.0.execute_batch("PRAGMA user_version=2;").unwrap();
        drop(opened);
        assert!(SqlCipherTaskStore::open(&path, &key).is_err());
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
