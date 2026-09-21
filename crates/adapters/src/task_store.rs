use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use std::{path::Path, time::Duration};
use yonder_application::{Action, AttemptConclusion, AttemptPhase, AttemptResultRecord, ControlKind, ControlPhase, ControlRequestRecord, Error, ExecutionAttempt, FocusPhase, Status, StepBoundaryRecord, StepDeclaration, StopRecord, Task, TaskEventRecord, TaskStore, Transition, browser_use::{BrowserReferenceRecord,valid_ref}, computer_use::UnknownReason, work_focus::FocusFailure};

pub struct SqliteTaskStore(Connection);
// 保留已有加密调用与验证名称，共用同一存储实现。
pub type SqlCipherTaskStore = SqliteTaskStore;

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

fn reason(value: UnknownReason) -> &'static str {
    match value { UnknownReason::InvalidInput => "invalid-input", UnknownReason::DependencyUnavailable => "dependency-unavailable", UnknownReason::WorkerFailed => "worker-failed", UnknownReason::TimedOut => "timed-out", UnknownReason::InvalidResponse => "invalid-response", UnknownReason::IdentityMismatch => "identity-mismatch", UnknownReason::ObserveFailed => "observe-failed", UnknownReason::UserInput => "user-input" }
}

fn parse_reason(value: &str) -> Result<UnknownReason, Error> {
    match value { "invalid-input" => Ok(UnknownReason::InvalidInput), "dependency-unavailable" => Ok(UnknownReason::DependencyUnavailable), "worker-failed" => Ok(UnknownReason::WorkerFailed), "timed-out" => Ok(UnknownReason::TimedOut), "invalid-response" => Ok(UnknownReason::InvalidResponse), "identity-mismatch" => Ok(UnknownReason::IdentityMismatch), "observe-failed" => Ok(UnknownReason::ObserveFailed), "user-input" => Ok(UnknownReason::UserInput), _ => Err(Error::StorageUnavailable) }
}

fn focus_failure(value: FocusFailure) -> &'static str { match value { FocusFailure::PermissionUnavailable=>"permission-unavailable",FocusFailure::ProcessChanged=>"process-changed",FocusFailure::WindowMissing=>"window-missing",FocusFailure::MappingNotUnique=>"mapping-not-unique",FocusFailure::ActivationFailed=>"activation-failed",FocusFailure::VerificationFailed=>"verification-failed",FocusFailure::GeometryChanged=>"geometry-changed",FocusFailure::ReferenceUnavailable=>"reference-unavailable" } }
fn parse_focus_failure(value:&str)->Result<FocusFailure,Error>{match value{"permission-unavailable"=>Ok(FocusFailure::PermissionUnavailable),"process-changed"=>Ok(FocusFailure::ProcessChanged),"window-missing"=>Ok(FocusFailure::WindowMissing),"mapping-not-unique"=>Ok(FocusFailure::MappingNotUnique),"activation-failed"=>Ok(FocusFailure::ActivationFailed),"verification-failed"=>Ok(FocusFailure::VerificationFailed),"geometry-changed"=>Ok(FocusFailure::GeometryChanged),"reference-unavailable"=>Ok(FocusFailure::ReferenceUnavailable),_=>Err(Error::StorageUnavailable)}}

impl SqliteTaskStore {
    pub fn open_unencrypted(path: &Path) -> Result<Self, Error> {
        Self::initialize(Connection::open(path).map_err(storage)?, true)
    }

    pub fn open(path: &Path, database_key: &[u8; 32]) -> Result<Self, Error> {
        let db = Connection::open(path).map_err(storage)?;
        // SAFETY: 连接句柄有效；同步调用期间 key 的 32 字节保持存活。
        let result = unsafe {
            rusqlite::ffi::sqlite3_key(db.handle(), database_key.as_ptr().cast(), 32)
        };
        if result != rusqlite::ffi::SQLITE_OK { return Err(Error::StorageUnavailable); }
        let version: String = db.query_row("PRAGMA cipher_version", [], |r| r.get(0)).map_err(storage)?;
        if version.is_empty() { return Err(Error::StorageUnavailable); }
        Self::initialize(db, false)
    }

    fn initialize(mut db: Connection, migrate_plaintext: bool) -> Result<Self, Error> {
        // 在任何初始化写入前拒绝不可读格式及未知版本，包括已有加密库。
        let schema: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(storage)?;
        if ![0,2,3,4,5,6,7,8,9,10,11,12,13,14].contains(&schema) { return Err(Error::StorageUnavailable); }
        if schema != 0 && schema != 14 {
            if !migrate_plaintext { return Err(Error::StorageUnavailable); }
            if let Some(path) = db.path().filter(|path| !path.is_empty()) {
                let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|_| Error::StorageUnavailable)?.as_nanos();
                let backup = format!("{path}.pre-attempt-v{schema}-{}-{nonce}.db", std::process::id());
                // SQLite自己复制一致快照；目标已存在或磁盘失败时拒绝升级。
                db.execute("VACUUM INTO ?1", [&backup]).map_err(storage)?;
            }
        }
        db.busy_timeout(Duration::from_secs(2)).map_err(storage)?;
        db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;").map_err(storage)?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let schema: i64 = tx.query_row("PRAGMA user_version", [], |r| r.get(0)).map_err(storage)?;
        if schema == 0 {
            let objects: i64 = tx.query_row("SELECT count(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'", [], |r| r.get(0)).map_err(storage)?;
            if objects != 0 { return Err(Error::StorageUnavailable); }
            tx.execute_batch(include_str!("task_schema.sql")).map_err(storage)?;
        } else if ![2,3,4,5,6,7,8,9,10,11,12,13,14].contains(&schema) { return Err(Error::StorageUnavailable); }
        if schema == 0 || schema == 2 { tx.execute_batch(include_str!("task_registration_schema.sql")).map_err(storage)?; }
        if schema == 4 {
            // AD-TM-06：仅兼容未使用实验列，绝不清理或伪造恢复记录。
            let unsafe_records: i64 = tx.query_row("SELECT (SELECT count(*) FROM tasks WHERE deleted!=0)+(SELECT count(*) FROM events WHERE kind!='transition')", [], |r| r.get(0)).map_err(storage)?;
            if unsafe_records != 0 { return Err(Error::StorageUnavailable); }
            tx.execute_batch("ALTER TABLE tasks DROP COLUMN deleted; ALTER TABLE events DROP COLUMN kind; PRAGMA user_version=3;").map_err(storage)?;
        }
        if schema < 5 { tx.execute_batch(include_str!("task_name_schema.sql")).map_err(storage)?; }
        if schema < 6 { tx.execute_batch(include_str!("task_step_schema.sql")).map_err(storage)?; }
        if schema < 7 { tx.execute_batch(include_str!("task_attempt_schema.sql")).map_err(storage)?; }
        if schema < 8 { tx.execute_batch(include_str!("task_attempt_result_schema.sql")).map_err(storage)?; }
        if schema < 9 { tx.execute_batch(include_str!("task_stop_schema.sql")).map_err(storage)?; }
        if schema < 10 { tx.execute_batch(include_str!("task_control_schema.sql")).map_err(storage)?; }
        if schema < 11 { tx.execute_batch(include_str!("task_advance_schema.sql")).map_err(storage)?; }
        if schema < 12 { tx.execute_batch(include_str!("task_browser_reference_schema.sql")).map_err(storage)?; }
        if schema < 13 { tx.execute_batch(include_str!("task_focus_schema.sql")).map_err(storage)?; }
        if schema < 14 { tx.execute_batch(include_str!("task_wait_schema.sql")).map_err(storage)?; }
        tx.commit().map_err(storage)?;
        Ok(Self(db))
    }

    fn update_focus(&mut self,task_id:&str,control_id:&str,result:Option<Option<FocusFailure>>)->Result<(Task,ControlRequestRecord),Error>{
        if !yonder_application::valid_id(task_id)||!yonder_application::valid_id(control_id){return Err(Error::InvalidInput);}
        let tx=self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let (state,sequence):(String,i64)=tx.query_row("SELECT state,sequence FROM tasks WHERE id=?1",[task_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(storage)?.ok_or(Error::NotFound)?;
        if state!="paused"{return Err(Error::StopRequired);}
        let (kind,phase,current,current_failure):(String,String,Option<String>,Option<String>)=tx.query_row("SELECT kind,phase,focus_phase,focus_failure FROM task_controls WHERE task_id=?1 AND control_id=?2",params![task_id,control_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(storage)?.ok_or(Error::StopRequired)?;
        if kind!="takeover"||phase!="stopped"{return Err(Error::StopRequired);}
        let (next,failure)=match result{None=>("locating",None),Some(None)=>("focused",None),Some(Some(value))=>("failed",Some(focus_failure(value)))};
        if current.as_deref()==Some(next)&&current_failure.as_deref()==failure{tx.commit().map_err(storage)?;let task=TaskStore::get(self,task_id)?;let control=TaskStore::get_control(self,task_id)?.ok_or(Error::StorageUnavailable)?;return Ok((task,control));}
        if (result.is_none()&&current.is_some())||(result.is_some()&&current.as_deref()!=Some("locating")){return Err(Error::Conflict);}
        let next_sequence=sequence.checked_add(1).ok_or(Error::StorageUnavailable)?;
        if tx.execute("UPDATE tasks SET sequence=?1 WHERE id=?2 AND state='paused' AND sequence=?3",params![next_sequence,task_id,sequence]).map_err(storage)?!=1{return Err(Error::Conflict);}
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'paused','paused')",params![task_id,next_sequence]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",params![task_id,next_sequence]).map_err(storage)?;
        tx.execute("UPDATE task_controls SET focus_phase=?1,focus_failure=?2,focus_sequence=?3 WHERE task_id=?4 AND control_id=?5",params![next,failure,next_sequence,task_id,control_id]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        let task=TaskStore::get(self,task_id)?;let control=TaskStore::get_control(self,task_id)?.ok_or(Error::StorageUnavailable)?;Ok((task,control))
    }
}

impl TaskStore for SqliteTaskStore {
    fn supports_pending_cancel(&self) -> bool { true }
    fn supports_running_filter(&self) -> bool { true }
    fn supports_registration(&self) -> bool { true }
    fn supports_step_declarations(&self) -> bool { true }
    fn supports_execution_attempts(&self) -> bool { true }
    fn supports_browser_references(&self) -> bool { true }
    fn supports_controls(&self) -> bool { true }
    fn supports_wait_for_user(&self) -> bool { true }

    fn wait_for_user(&mut self, task_id: &str, expected: u64, reason: &str) -> Result<Task, Error> {
        if !yonder_application::valid_id(task_id) || expected == 0 || expected >= i64::MAX as u64 || reason.trim()!=reason || reason.is_empty() || reason.as_bytes().len()>512 || reason.chars().any(char::is_control) { return Err(Error::InvalidInput); }
        let tx=self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let row:Option<(String,i64,String,Option<String>)>=tx.query_row("SELECT state,sequence,owner_agent_id,name FROM tasks WHERE id=?1",[task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(storage)?;
        let (state,sequence,owner_agent_id,task_name)=row.ok_or(Error::NotFound)?;
        if state!="running" || sequence!=expected as i64 { return Err(Error::Conflict); }
        let pending:i64=tx.query_row("SELECT count(*) FROM task_controls WHERE task_id=?1 AND phase='pending'",[task_id],|r|r.get(0)).map_err(storage)?;
        let phase:Option<String>=tx.query_row("SELECT phase FROM task_attempts WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1",[task_id],|r|r.get(0)).optional().map_err(storage)?;
        if pending!=0 || phase.as_deref()!=Some("stopped") { return Err(Error::StopRequired); }
        let next=expected+1;
        if tx.execute("UPDATE tasks SET state='waiting-for-user',sequence=?1 WHERE id=?2 AND state='running' AND sequence=?3",params![next as i64,task_id,expected as i64]).map_err(storage)?!=1{return Err(Error::Conflict);}
        tx.execute("INSERT INTO events(task_id,sequence,previous,state,wait_reason) VALUES (?1,?2,'running','waiting-for-user',?3)",params![task_id,next as i64,reason]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",params![task_id,next as i64]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Task{id:task_id.into(),owner_agent_id,name:task_name,status:Status::WaitingForUser,sequence:next})
    }

    fn prepare_attempt(&mut self, requested: &ExecutionAttempt, expected: u64) -> Result<(Task, ExecutionAttempt), Error> {
        if expected == 0 || expected >= i64::MAX as u64 || requested.phase != AttemptPhase::Prepared || requested.accepted_sequence != 0 || [requested.task_id.as_str(),requested.step_id.as_str(),requested.attempt_id.as_str(),requested.worker_instance_id.as_str(),requested.host_session_id.as_str()].iter().any(|id| !yonder_application::valid_id(id)) { return Err(Error::InvalidInput); }
        let tx = self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let row: Option<(String,i64,String,Option<String>)> = tx.query_row("SELECT state,sequence,owner_agent_id,name FROM tasks WHERE id=?1", [&requested.task_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(storage)?;
        let (state,sequence,owner_agent_id,task_name) = row.ok_or(Error::NotFound)?;
        let task = Task { id:requested.task_id.clone(), owner_agent_id, name:task_name, status:status(&state)?, sequence:u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? };
        let existing: Option<(String,String,String,String,String,i64)> = tx.query_row("SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence FROM task_attempts WHERE task_id=?1 AND attempt_id=?2", params![requested.task_id,requested.attempt_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional().map_err(storage)?;
        if let Some((step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted)) = existing {
            let accepted_sequence = u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)?;
            if phase != "prepared" || step_id != requested.step_id || worker_instance_id != requested.worker_instance_id || host_session_id != requested.host_session_id || accepted_sequence != expected + 1 { return Err(Error::Conflict); }
            tx.commit().map_err(storage)?;
            return Ok((task, ExecutionAttempt { task_id:requested.task_id.clone(), step_id, attempt_id, worker_instance_id, host_session_id, phase:AttemptPhase::Prepared, accepted_sequence }));
        }
        if !matches!(task.status,Status::Created|Status::Running) || task.sequence != expected { return Err(Error::Conflict); }
        let current_step: Option<(String,i64)> = tx.query_row("SELECT step_id,accepted_sequence FROM task_steps WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1", [&requested.task_id], |r| Ok((r.get(0)?,r.get(1)?))).optional().map_err(storage)?;
        if current_step.as_ref().map(|value|value.0.as_str()) != Some(requested.step_id.as_str()) { return Err(Error::StopRequired); }
        if task.status == Status::Running {
            let previous: Option<(String,Option<String>,i64)> = tx.query_row("SELECT phase,control_id,COALESCE(stop_sequence,0) FROM task_attempts WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1",[&requested.task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage)?;
            let step_sequence = u64::try_from(current_step.as_ref().unwrap().1).map_err(|_|Error::StorageUnavailable)?;
            if !matches!(previous,Some((ref phase,None,stop)) if phase=="stopped" && step_sequence > u64::try_from(stop).unwrap_or(u64::MAX)) { return Err(Error::StopRequired); }
        }
        let accepted = expected + 1;
        if tx.execute("UPDATE tasks SET state='running',sequence=?1 WHERE id=?2 AND state=?3 AND sequence=?4", params![accepted as i64,requested.task_id,state,expected as i64]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,?3,'running')", params![requested.task_id,accepted as i64,state]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)", params![requested.task_id,accepted as i64]).map_err(storage)?;
        tx.execute("INSERT INTO task_attempts(task_id,step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence) VALUES (?1,?2,?3,?4,?5,'prepared',?6)", params![requested.task_id,requested.step_id,requested.attempt_id,requested.worker_instance_id,requested.host_session_id,accepted as i64]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        let accepted_attempt = ExecutionAttempt { accepted_sequence:accepted, ..requested.clone() };
        Ok((Task { status:Status::Running, sequence:accepted, ..task }, accepted_attempt))
    }

    fn get_attempt(&mut self, task_id: &str) -> Result<Option<ExecutionAttempt>, Error> {
        let row: Option<(String,String,String,String,String,i64)> = self.0.query_row("SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence FROM task_attempts WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1", [task_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional().map_err(storage)?;
        row.map(|(step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted)| {
            let phase = match phase.as_str() { "prepared" => AttemptPhase::Prepared, "observed" => AttemptPhase::Observed, "unknown" => AttemptPhase::Unknown, "stopped" => AttemptPhase::Stopped, _ => return Err(Error::StorageUnavailable) };
            Ok(ExecutionAttempt { task_id:task_id.into(), step_id, attempt_id, worker_instance_id, host_session_id, phase, accepted_sequence:u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)? })
        }).transpose()
    }

    fn get_attempt_result(&mut self,task_id:&str)->Result<Option<AttemptResultRecord>,Error>{
        let row:Option<(String,String,String,String,String,Option<i64>,Option<i64>,Option<String>,Option<i64>)>=self.0.query_row("SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,action_succeeded,observe_valid,unknown_reason,result_sequence FROM task_attempts WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1",[task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?))).optional().map_err(storage)?;
        row.map(|(step_id,attempt_id,worker_instance_id,host_session_id,phase,action,observe,reason_value,result)|{
            let conclusion=match (phase.as_str(),action,observe,reason_value.as_deref()){
                ("observed"|"stopped",Some(value),Some(1),None)=>AttemptConclusion::Observed{action_succeeded:value!=0},
                ("unknown",None,Some(0),Some(value))=>AttemptConclusion::Unknown{reason:parse_reason(value)?},
                _=>return Err(Error::StopRequired),
            };
            Ok(AttemptResultRecord{task_id:task_id.into(),step_id,attempt_id,worker_instance_id,host_session_id,conclusion,result_sequence:u64::try_from(result.ok_or(Error::StorageUnavailable)?).map_err(|_|Error::StorageUnavailable)?})
        }).transpose()
    }

    fn record_attempt_result(&mut self, attempt: &ExecutionAttempt, expected: u64, conclusion: AttemptConclusion) -> Result<(Task, AttemptResultRecord), Error> {
        if expected == 0 || expected >= i64::MAX as u64 || attempt.accepted_sequence == 0 || [attempt.task_id.as_str(),attempt.step_id.as_str(),attempt.attempt_id.as_str(),attempt.worker_instance_id.as_str(),attempt.host_session_id.as_str()].iter().any(|id| !yonder_application::valid_id(id)) { return Err(Error::InvalidInput); }
        let tx = self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let task_row: Option<(String,i64,String,Option<String>)> = tx.query_row("SELECT state,sequence,owner_agent_id,name FROM tasks WHERE id=?1", [&attempt.task_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(storage)?;
        let (task_state,task_sequence,owner,name_value) = task_row.ok_or(Error::NotFound)?;
        let task = Task { id:attempt.task_id.clone(), owner_agent_id:owner, name:name_value, status:status(&task_state)?, sequence:u64::try_from(task_sequence).map_err(|_| Error::StorageUnavailable)? };
        if task.status != Status::Running || task.sequence != expected { return Err(Error::Conflict); }
        let row: Option<(String,String,String,String,String,i64,Option<i64>,Option<i64>,Option<i64>,Option<String>)> = tx.query_row(
            "SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence,result_sequence,action_succeeded,observe_valid,unknown_reason FROM task_attempts WHERE task_id=?1 AND attempt_id=?2",
            params![attempt.task_id,attempt.attempt_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?)),
        ).optional().map_err(storage)?;
        let (step_id,attempt_id,worker_id,host_id,phase,accepted,result_sequence,action_succeeded,observe_valid,unknown_reason) = row.ok_or(Error::Conflict)?;
        if step_id != attempt.step_id || worker_id != attempt.worker_instance_id || host_id != attempt.host_session_id || u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)? != attempt.accepted_sequence { return Err(Error::Conflict); }
        if phase != "prepared" {
            if phase == "stopped" { return Err(Error::Conflict); }
            let existing = match (phase.as_str(),action_succeeded,observe_valid,unknown_reason.as_deref()) {
                ("observed",Some(value),Some(1),None) => AttemptConclusion::Observed { action_succeeded:value != 0 },
                ("unknown",None,Some(0),Some(value)) => AttemptConclusion::Unknown { reason:parse_reason(value)? },
                _ => return Err(Error::StorageUnavailable),
            };
            if existing != conclusion || result_sequence != Some(task_sequence) { return Err(Error::Conflict); }
            tx.commit().map_err(storage)?;
            return Ok((task, AttemptResultRecord { task_id:attempt.task_id.clone(), step_id, attempt_id, worker_instance_id:worker_id, host_session_id:host_id, conclusion:existing, result_sequence:u64::try_from(task_sequence).map_err(|_| Error::StorageUnavailable)? }));
        }
        let sequence = expected + 1;
        let (phase_name,action_value,observe_value,reason_value) = match conclusion {
            AttemptConclusion::Observed { action_succeeded } => ("observed",Some(i64::from(action_succeeded)),1,None),
            AttemptConclusion::Unknown { reason:unknown } => ("unknown",None,0,Some(reason(unknown))),
        };
        if tx.execute("UPDATE tasks SET sequence=?1 WHERE id=?2 AND state='running' AND sequence=?3", params![sequence as i64,attempt.task_id,expected as i64]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'running','running')", params![attempt.task_id,sequence as i64]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)", params![attempt.task_id,sequence as i64]).map_err(storage)?;
        if tx.execute("UPDATE task_attempts SET phase=?1,result_sequence=?2,action_succeeded=?3,observe_valid=?4,unknown_reason=?5 WHERE task_id=?6 AND attempt_id=?7 AND phase='prepared'", params![phase_name,sequence as i64,action_value,observe_value,reason_value,attempt.task_id,attempt.attempt_id]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.commit().map_err(storage)?;
        Ok((Task { sequence, ..task }, AttemptResultRecord { task_id:attempt.task_id.clone(), step_id, attempt_id, worker_instance_id:worker_id, host_session_id:host_id, conclusion, result_sequence:sequence }))
    }

    fn record_browser_result(&mut self, attempt:&ExecutionAttempt, expected:u64, reference:&BrowserReferenceRecord)->Result<(Task,AttemptResultRecord),Error>{
        if expected==0 || expected>=i64::MAX as u64 || attempt.accepted_sequence==0 || reference.task_id!=attempt.task_id || !valid_ref(&reference.external_task_ref)
            || !matches!(reference.ownership.as_str(),"agent"|"agentDelegatedToUser"|"user") || reference.managed_pages>100 || reference.updated_sequence!=0 { return Err(Error::InvalidInput); }
        let tx=self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let row:Option<(String,i64,String,Option<String>)>=tx.query_row("SELECT state,sequence,owner_agent_id,name FROM tasks WHERE id=?1",[&attempt.task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(storage)?;
        let (state,sequence,owner,name_value)=row.ok_or(Error::NotFound)?;
        let task=Task{id:attempt.task_id.clone(),owner_agent_id:owner,name:name_value,status:status(&state)?,sequence:u64::try_from(sequence).map_err(|_|Error::StorageUnavailable)?};
        if task.status!=Status::Running || task.sequence!=expected { return Err(Error::Conflict); }
        let current:Option<(String,String,String,String,String,i64,Option<i64>)>=tx.query_row("SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence,result_sequence FROM task_attempts WHERE task_id=?1 AND attempt_id=?2",params![attempt.task_id,attempt.attempt_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?))).optional().map_err(storage)?;
        let (step_id,attempt_id,worker_id,host_id,phase,accepted,result_sequence)=current.ok_or(Error::Conflict)?;
        if step_id!=attempt.step_id || worker_id!=attempt.worker_instance_id || host_id!=attempt.host_session_id || u64::try_from(accepted).map_err(|_|Error::StorageUnavailable)?!=attempt.accepted_sequence{return Err(Error::Conflict);}
        if phase!="prepared" {
            if phase!="observed" || result_sequence!=Some(sequence) { return Err(Error::Conflict); }
            let stored:Option<(String,String,i64,i64,i64)>=tx.query_row("SELECT external_task_ref,ownership,managed_pages,finished,updated_sequence FROM task_browser_refs WHERE task_id=?1",[&attempt.task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(storage)?;
            if !matches!(stored,Some((ref external,ref ownership,pages,finished,updated)) if external==&reference.external_task_ref && ownership==&reference.ownership && usize::try_from(pages).ok()==Some(reference.managed_pages) && (finished!=0)==reference.finished && updated==sequence){return Err(Error::Conflict);}
            tx.commit().map_err(storage)?;
            return Ok((task,AttemptResultRecord{task_id:attempt.task_id.clone(),step_id,attempt_id,worker_instance_id:worker_id,host_session_id:host_id,conclusion:AttemptConclusion::Observed{action_succeeded:true},result_sequence:u64::try_from(sequence).map_err(|_|Error::StorageUnavailable)?}));
        }
        let next=expected+1;
        if tx.execute("UPDATE tasks SET sequence=?1 WHERE id=?2 AND state='running' AND sequence=?3",params![next as i64,attempt.task_id,expected as i64]).map_err(storage)?!=1{return Err(Error::Conflict);}
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'running','running')",params![attempt.task_id,next as i64]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",params![attempt.task_id,next as i64]).map_err(storage)?;
        if tx.execute("UPDATE task_attempts SET phase='observed',result_sequence=?1,action_succeeded=1,observe_valid=1 WHERE task_id=?2 AND attempt_id=?3 AND phase='prepared'",params![next as i64,attempt.task_id,attempt.attempt_id]).map_err(storage)?!=1{return Err(Error::Conflict);}
        tx.execute("INSERT INTO task_browser_refs(task_id,external_task_ref,ownership,managed_pages,finished,updated_sequence) VALUES (?1,?2,?3,?4,?5,?6) ON CONFLICT(task_id) DO UPDATE SET external_task_ref=excluded.external_task_ref,ownership=excluded.ownership,managed_pages=excluded.managed_pages,finished=excluded.finished,updated_sequence=excluded.updated_sequence",params![attempt.task_id,reference.external_task_ref,reference.ownership,reference.managed_pages as i64,i64::from(reference.finished),next as i64]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok((Task{sequence:next,..task},AttemptResultRecord{task_id:attempt.task_id.clone(),step_id,attempt_id,worker_instance_id:worker_id,host_session_id:host_id,conclusion:AttemptConclusion::Observed{action_succeeded:true},result_sequence:next}))
    }

    fn get_browser_reference(&mut self,task_id:&str)->Result<Option<BrowserReferenceRecord>,Error>{
        if !yonder_application::valid_id(task_id){return Err(Error::InvalidInput);}
        let row:Option<(String,String,i64,i64,i64)>=self.0.query_row("SELECT external_task_ref,ownership,managed_pages,finished,updated_sequence FROM task_browser_refs WHERE task_id=?1",[task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional().map_err(storage)?;
        row.map(|(external_task_ref,ownership,managed_pages,finished,updated_sequence)|Ok(BrowserReferenceRecord{task_id:task_id.into(),external_task_ref,ownership,managed_pages:usize::try_from(managed_pages).map_err(|_|Error::StorageUnavailable)?,finished:finished!=0,updated_sequence:u64::try_from(updated_sequence).map_err(|_|Error::StorageUnavailable)?})).transpose()
    }

    fn advance_attempt(&mut self, attempt: &ExecutionAttempt, expected: u64) -> Result<(Task, StepBoundaryRecord), Error> {
        if expected == 0 || expected >= i64::MAX as u64 || !matches!(attempt.phase,AttemptPhase::Observed|AttemptPhase::Stopped) { return Err(Error::InvalidInput); }
        let tx = self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let row: Option<(String,i64,String,Option<String>)> = tx.query_row("SELECT state,sequence,owner_agent_id,name FROM tasks WHERE id=?1",[&attempt.task_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(storage)?;
        let (state,sequence,owner,name_value)=row.ok_or(Error::NotFound)?;
        let task=Task{id:attempt.task_id.clone(),owner_agent_id:owner,name:name_value,status:status(&state)?,sequence:u64::try_from(sequence).map_err(|_|Error::StorageUnavailable)?};
        if task.status!=Status::Running || task.sequence!=expected { return Err(Error::Conflict); }
        let current: Option<(String,String,String,String,String,i64,Option<i64>,Option<String>)> = tx.query_row("SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence,stop_sequence,control_id FROM task_attempts WHERE task_id=?1 AND attempt_id=?2",params![attempt.task_id,attempt.attempt_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).optional().map_err(storage)?;
        let (step_id,attempt_id,worker_id,host_id,phase,accepted,stopped,control_id)=current.ok_or(Error::Conflict)?;
        if step_id!=attempt.step_id || worker_id!=attempt.worker_instance_id || host_id!=attempt.host_session_id || u64::try_from(accepted).map_err(|_|Error::StorageUnavailable)?!=attempt.accepted_sequence { return Err(Error::Conflict); }
        if phase=="stopped" {
            if control_id.is_some() { return Err(Error::StopRequired); }
            let stop_sequence=u64::try_from(stopped.ok_or(Error::StorageUnavailable)?).map_err(|_|Error::StorageUnavailable)?;
            tx.commit().map_err(storage)?;
            return Ok((task,StepBoundaryRecord{task_id:attempt.task_id.clone(),step_id,attempt_id,worker_instance_id:worker_id,host_session_id:host_id,stop_sequence}));
        }
        if phase!="observed" { return Err(Error::StopRequired); }
        let pending:i64=tx.query_row("SELECT count(*) FROM task_controls WHERE task_id=?1 AND phase='pending'",[&attempt.task_id],|r|r.get(0)).map_err(storage)?;
        if pending!=0 { return Err(Error::StopRequired); }
        let next=expected+1;
        if tx.execute("UPDATE tasks SET sequence=?1 WHERE id=?2 AND state='running' AND sequence=?3",params![next as i64,attempt.task_id,expected as i64]).map_err(storage)?!=1{return Err(Error::Conflict);}
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'running','running')",params![attempt.task_id,next as i64]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",params![attempt.task_id,next as i64]).map_err(storage)?;
        if tx.execute("UPDATE task_attempts SET phase='stopped',stop_sequence=?1 WHERE task_id=?2 AND attempt_id=?3 AND phase='observed'",params![next as i64,attempt.task_id,attempt.attempt_id]).map_err(storage)?!=1{return Err(Error::Conflict);}
        tx.commit().map_err(storage)?;
        Ok((Task{sequence:next,..task},StepBoundaryRecord{task_id:attempt.task_id.clone(),step_id,attempt_id,worker_instance_id:worker_id,host_session_id:host_id,stop_sequence:next}))
    }

    fn request_control(&mut self, attempt: &ExecutionAttempt, expected: u64, control_id: &str, kind: ControlKind) -> Result<(Task, ControlRequestRecord), Error> {
        if expected == 0 || expected >= i64::MAX as u64 || !yonder_application::valid_id(control_id) { return Err(Error::InvalidInput); }
        let kind_name = match kind { ControlKind::Pause => "pause", ControlKind::Cancel => "cancel", ControlKind::Takeover => "takeover" };
        let tx = self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let row: Option<(String,i64,String,Option<String>)> = tx.query_row("SELECT state,sequence,owner_agent_id,name FROM tasks WHERE id=?1", [&attempt.task_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(storage)?;
        let (state,sequence,owner,name_value) = row.ok_or(Error::NotFound)?;
        let task = Task { id:attempt.task_id.clone(), owner_agent_id:owner, name:name_value, status:status(&state)?, sequence:u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? };
        let existing: Option<(String,String,String,String,i64,Option<i64>,Option<String>,Option<String>)> = tx.query_row("SELECT attempt_id,control_id,kind,phase,accepted_sequence,stopped_sequence,focus_phase,focus_failure FROM task_controls WHERE task_id=?1", [&attempt.task_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).optional().map_err(storage)?;
        if let Some((attempt_id,stored_id,stored_kind,phase,accepted,stopped,focus,focus_error)) = existing {
            let accepted = u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)?;
            if attempt_id != attempt.attempt_id || stored_id != control_id || stored_kind != kind_name || (expected != accepted && expected.checked_add(1) != Some(accepted)) { return Err(Error::Conflict); }
            let phase = match phase.as_str() { "pending" => ControlPhase::Pending, "stopped" => ControlPhase::Stopped, _ => return Err(Error::StorageUnavailable) };
            tx.commit().map_err(storage)?;
            let focus_phase=match focus.as_deref(){Some("locating")=>Some(FocusPhase::Locating),Some("focused")=>Some(FocusPhase::Focused),Some("failed")=>Some(FocusPhase::Failed),None=>None,_=>return Err(Error::StorageUnavailable)};
            let focus_failure=focus_error.as_deref().map(parse_focus_failure).transpose()?;
            return Ok((task,ControlRequestRecord { task_id:attempt.task_id.clone(), attempt_id, control_id:stored_id, kind, phase, accepted_sequence:accepted, stopped_sequence:stopped.map(u64::try_from).transpose().map_err(|_| Error::StorageUnavailable)?,focus_phase,focus_failure }));
        }
        if task.status != Status::Running || task.sequence != expected { return Err(Error::Conflict); }
        let accepted = expected + 1;
        let direct=attempt.phase==AttemptPhase::Stopped;
        let next_state=if direct { if kind==ControlKind::Cancel{"cancelled"}else{"paused"} } else {"running"};
        if direct {
            let uncontrolled:i64=tx.query_row("SELECT count(*) FROM task_attempts WHERE task_id=?1 AND attempt_id=?2 AND phase='stopped' AND control_id IS NULL",params![attempt.task_id,attempt.attempt_id],|r|r.get(0)).map_err(storage)?;
            if uncontrolled!=1{return Err(Error::StopRequired);}
        }
        if tx.execute("UPDATE tasks SET state=?1,sequence=?2 WHERE id=?3 AND state='running' AND sequence=?4",params![next_state,accepted as i64,attempt.task_id,expected as i64]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'running',?3)",params![attempt.task_id,accepted as i64,next_state]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",params![attempt.task_id,accepted as i64]).map_err(storage)?;
        let control_phase=if direct{"stopped"}else{"pending"};
        tx.execute("INSERT INTO task_controls(task_id,attempt_id,control_id,kind,phase,accepted_sequence,stopped_sequence) VALUES (?1,?2,?3,?4,?5,?6,?7)",params![attempt.task_id,attempt.attempt_id,control_id,kind_name,control_phase,accepted as i64,if direct{Some(accepted as i64)}else{None}]).map_err(storage)?;
        if direct{tx.execute("UPDATE task_attempts SET control_id=?1,control_kind=?2 WHERE task_id=?3 AND attempt_id=?4 AND phase='stopped' AND control_id IS NULL",params![control_id,kind_name,attempt.task_id,attempt.attempt_id]).map_err(storage)?;}
        tx.commit().map_err(storage)?;
        Ok((Task { status:if direct{if kind==ControlKind::Cancel{Status::Cancelled}else{Status::Paused}}else{task.status}, sequence:accepted, ..task },ControlRequestRecord { task_id:attempt.task_id.clone(), attempt_id:attempt.attempt_id.clone(), control_id:control_id.into(), kind, phase:if direct{ControlPhase::Stopped}else{ControlPhase::Pending}, accepted_sequence:accepted, stopped_sequence:if direct{Some(accepted)}else{None},focus_phase:None,focus_failure:None }))
    }

    fn get_control(&mut self, task_id: &str) -> Result<Option<ControlRequestRecord>, Error> {
        let row: Option<(String,String,String,String,i64,Option<i64>,Option<String>,Option<String>)> = self.0.query_row("SELECT attempt_id,control_id,kind,phase,accepted_sequence,stopped_sequence,focus_phase,focus_failure FROM task_controls WHERE task_id=?1", [task_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?))).optional().map_err(storage)?;
        row.map(|(attempt_id,control_id,kind,phase,accepted,stopped,focus,focus_error)| Ok(ControlRequestRecord {
            task_id:task_id.into(), attempt_id, control_id,
            kind:match kind.as_str() { "pause"=>ControlKind::Pause,"cancel"=>ControlKind::Cancel,"takeover"=>ControlKind::Takeover,_=>return Err(Error::StorageUnavailable) },
            phase:match phase.as_str() { "pending"=>ControlPhase::Pending,"stopped"=>ControlPhase::Stopped,_=>return Err(Error::StorageUnavailable) },
            accepted_sequence:u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)?, stopped_sequence:stopped.map(u64::try_from).transpose().map_err(|_| Error::StorageUnavailable)?,
            focus_phase:match focus.as_deref(){Some("locating")=>Some(FocusPhase::Locating),Some("focused")=>Some(FocusPhase::Focused),Some("failed")=>Some(FocusPhase::Failed),None=>None,_=>return Err(Error::StorageUnavailable)},focus_failure:focus_error.as_deref().map(parse_focus_failure).transpose()?,
        })).transpose()
    }

    fn begin_focus(&mut self,task_id:&str,control_id:&str)->Result<(Task,ControlRequestRecord),Error>{self.update_focus(task_id,control_id,None)}
    fn finish_focus(&mut self,task_id:&str,control_id:&str,failure:Option<FocusFailure>)->Result<(Task,ControlRequestRecord),Error>{self.update_focus(task_id,control_id,Some(failure))}

    fn stop_attempt(&mut self, attempt: &ExecutionAttempt, expected: u64, control_id: &str, kind: ControlKind) -> Result<(Task, StopRecord), Error> {
        if expected == 0 || expected >= i64::MAX as u64 || !yonder_application::valid_id(control_id) || attempt.accepted_sequence == 0 || [attempt.task_id.as_str(),attempt.step_id.as_str(),attempt.attempt_id.as_str(),attempt.worker_instance_id.as_str(),attempt.host_session_id.as_str()].iter().any(|id| !yonder_application::valid_id(id)) { return Err(Error::InvalidInput); }
        let kind_name = match kind { ControlKind::Pause => "pause", ControlKind::Cancel => "cancel", ControlKind::Takeover => "takeover" };
        let action = match kind { ControlKind::Pause | ControlKind::Takeover => Action::Pause, ControlKind::Cancel => Action::Cancel };
        let tx = self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let task_row: Option<(String,i64,String,Option<String>)> = tx.query_row("SELECT state,sequence,owner_agent_id,name FROM tasks WHERE id=?1", [&attempt.task_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(storage)?;
        let (task_state,task_sequence,owner,name_value) = task_row.ok_or(Error::NotFound)?;
        let task = Task { id:attempt.task_id.clone(), owner_agent_id:owner, name:name_value, status:status(&task_state)?, sequence:u64::try_from(task_sequence).map_err(|_| Error::StorageUnavailable)? };
        if task.sequence != expected { return Err(Error::Conflict); }
        let control: Option<(String,String,String)> = tx.query_row("SELECT attempt_id,kind,phase FROM task_controls WHERE task_id=?1 AND control_id=?2",params![attempt.task_id,control_id],|r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage)?;
        if control.as_ref().is_none_or(|(attempt_id,stored_kind,phase)| attempt_id != &attempt.attempt_id || stored_kind != kind_name || !matches!(phase.as_str(),"pending"|"stopped")) { return Err(Error::StopRequired); }
        let row: Option<(String,String,String,String,String,i64,Option<i64>,Option<i64>,Option<String>,Option<String>)> = tx.query_row(
            "SELECT step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence,stop_sequence,action_succeeded,control_id,control_kind FROM task_attempts WHERE task_id=?1 AND attempt_id=?2",
            params![attempt.task_id,attempt.attempt_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?)),
        ).optional().map_err(storage)?;
        let (step_id,attempt_id,worker_id,host_id,phase,accepted,stop_sequence,action_succeeded,stored_control,stored_kind) = row.ok_or(Error::Conflict)?;
        if step_id != attempt.step_id || worker_id != attempt.worker_instance_id || host_id != attempt.host_session_id || u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)? != attempt.accepted_sequence { return Err(Error::Conflict); }
        if phase == "stopped" {
            let stop_sequence = stop_sequence.ok_or(Error::StorageUnavailable)?;
            let expected_status = if kind == ControlKind::Cancel { Status::Cancelled } else { Status::Paused };
            if stored_control.as_deref() != Some(control_id) || stored_kind.as_deref() != Some(kind_name) || stop_sequence != task_sequence || task.status != expected_status || control.as_ref().is_none_or(|(_,_,phase)| phase != "stopped") { return Err(Error::Conflict); }
            tx.commit().map_err(storage)?;
            return Ok((task,StopRecord { task_id:attempt.task_id.clone(), step_id, attempt_id, worker_instance_id:worker_id, host_session_id:host_id, control_id:control_id.into(), kind, stop_sequence:u64::try_from(stop_sequence).map_err(|_| Error::StorageUnavailable)? }));
        }
        if phase != "observed" || action_succeeded.is_none() || task.status != Status::Running { return Err(Error::StopRequired); }
        let transition = task.status.transition(expected,action).map_err(Error::InvalidTransition)?;
        if tx.execute("UPDATE tasks SET state=?1,sequence=?2 WHERE id=?3 AND state='running' AND sequence=?4",params![name(transition.next),transition.sequence as i64,attempt.task_id,expected as i64]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,'running',?3)",params![attempt.task_id,transition.sequence as i64,name(transition.next)]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",params![attempt.task_id,transition.sequence as i64]).map_err(storage)?;
        if tx.execute("UPDATE task_attempts SET phase='stopped',stop_sequence=?1,control_id=?2,control_kind=?3 WHERE task_id=?4 AND attempt_id=?5 AND phase='observed'",params![transition.sequence as i64,control_id,kind_name,attempt.task_id,attempt.attempt_id]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        if tx.execute("UPDATE task_controls SET phase='stopped',stopped_sequence=?1 WHERE task_id=?2 AND control_id=?3 AND phase='pending'",params![transition.sequence as i64,attempt.task_id,control_id]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.commit().map_err(storage)?;
        Ok((Task { status:transition.next, sequence:transition.sequence, ..task },StopRecord { task_id:attempt.task_id.clone(), step_id, attempt_id, worker_instance_id:worker_id, host_session_id:host_id, control_id:control_id.into(), kind, stop_sequence:transition.sequence }))
    }

    fn declare_step(&mut self, owner: &str, task_id: &str, expected: u64, step_id: &str, label: &str) -> Result<(Task, StepDeclaration), Error> {
        if [owner,task_id,step_id].iter().any(|id| !yonder_application::valid_id(id)) || expected == 0 || expected >= i64::MAX as u64 || !yonder_application::valid_step_label(label) { return Err(Error::InvalidInput); }
        let tx = self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let row: Option<(String,i64,String,Option<String>)> = tx.query_row("SELECT state,sequence,owner_agent_id,name FROM tasks WHERE id=?1", [task_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(storage)?;
        let (state, sequence, actual_owner, task_name) = row.ok_or(Error::NotFound)?;
        if actual_owner != owner { return Err(Error::NotFound); }
        let current = Task { id: task_id.into(), owner_agent_id: actual_owner, name: task_name, status: status(&state)?, sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? };
        let existing: Option<(String,i64)> = tx.query_row("SELECT label,accepted_sequence FROM task_steps WHERE task_id=?1 AND step_id=?2", params![task_id,step_id], |r| Ok((r.get(0)?,r.get(1)?))).optional().map_err(storage)?;
        if let Some((original, accepted)) = existing {
            if original != label { return Err(Error::StepConflict); }
            tx.commit().map_err(storage)?;
            return Ok((current, StepDeclaration { step_id: step_id.into(), label: original, accepted_sequence: u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)? }));
        }
        if !matches!(current.status,Status::Created|Status::Running) { return Err(Error::StopRequired); }
        if current.sequence != expected { return Err(Error::Conflict); }
        if current.status == Status::Running {
            let boundary: Option<(String,Option<String>)> = tx.query_row("SELECT phase,control_id FROM task_attempts WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1",[task_id],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(storage)?;
            let pending:i64=tx.query_row("SELECT count(*) FROM task_controls WHERE task_id=?1 AND phase='pending'",[task_id],|r|r.get(0)).map_err(storage)?;
            if !matches!(boundary,Some((ref phase,None)) if phase=="stopped") || pending!=0 { return Err(Error::StopRequired); }
        }
        let task_count: i64 = tx.query_row("SELECT count(*) FROM task_steps WHERE task_id=?1", [task_id], |r| r.get(0)).map_err(storage)?;
        let total_count: i64 = tx.query_row("SELECT count(*) FROM task_steps", [], |r| r.get(0)).map_err(storage)?;
        if task_count >= 1024 || total_count >= 10000 { return Err(Error::QuotaExceeded); }
        let accepted = expected + 1;
        if tx.execute("UPDATE tasks SET sequence=?1 WHERE id=?2 AND owner_agent_id=?3 AND state=?4 AND sequence=?5", params![accepted as i64,task_id,owner,state,expected as i64]).map_err(storage)? != 1 { return Err(Error::Conflict); }
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,?3,?3)", params![task_id,accepted as i64,state]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)", params![task_id,accepted as i64]).map_err(storage)?;
        tx.execute("INSERT INTO task_steps(task_id,step_id,label,accepted_sequence) VALUES (?1,?2,?3,?4)", params![task_id,step_id,label,accepted as i64]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok((Task { sequence: accepted, ..current }, StepDeclaration { step_id: step_id.into(), label: label.into(), accepted_sequence: accepted }))
    }

    fn get_with_step(&mut self, id: &str) -> Result<(Task, Option<StepDeclaration>), Error> {
        let tx = self.0.transaction().map_err(storage)?;
        let row: Option<(String,i64,String,Option<String>)> = tx.query_row("SELECT state,sequence,owner_agent_id,name FROM tasks WHERE id=?1", [id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(storage)?;
        let (state, sequence, owner_agent_id, name) = row.ok_or(Error::NotFound)?;
        let step: Option<(String,String,i64)> = tx.query_row("SELECT step_id,label,accepted_sequence FROM task_steps WHERE task_id=?1 ORDER BY accepted_sequence DESC LIMIT 1", [id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(storage)?;
        tx.commit().map_err(storage)?;
        let step = match step { Some((step_id,label,accepted)) => Some(StepDeclaration { step_id, label, accepted_sequence:u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)? }), None => None };
        Ok((Task { id:id.into(), owner_agent_id, name, status:status(&state)?, sequence:u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? }, step))
    }

    fn events_with_steps(&mut self, id: &str, after: u64, limit: usize) -> Result<Vec<TaskEventRecord>, Error> {
        if !(1..=100).contains(&limit) { return Err(Error::InvalidInput); }
        if after >= i64::MAX as u64 { return Ok(vec![]); }
        let mut stmt = self.0.prepare("SELECT e.previous,e.state,e.sequence,s.step_id,s.label,s.accepted_sequence,a.step_id,a.attempt_id,a.worker_instance_id,a.host_session_id,a.phase,a.action_succeeded,a.observe_valid,a.unknown_reason,e.wait_reason FROM events e LEFT JOIN task_steps s ON s.task_id=e.task_id AND s.accepted_sequence=e.sequence LEFT JOIN task_attempts a ON a.task_id=e.task_id AND a.result_sequence=e.sequence WHERE e.task_id=?1 AND e.sequence>?2 ORDER BY e.sequence LIMIT ?3").map_err(storage)?;
        let rows = stmt.query_map(params![id,after as i64,limit as i64], |r| Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?,r.get::<_,Option<String>>(3)?,r.get::<_,Option<String>>(4)?,r.get::<_,Option<i64>>(5)?,r.get::<_,Option<String>>(6)?,r.get::<_,Option<String>>(7)?,r.get::<_,Option<String>>(8)?,r.get::<_,Option<String>>(9)?,r.get::<_,Option<String>>(10)?,r.get::<_,Option<i64>>(11)?,r.get::<_,Option<i64>>(12)?,r.get::<_,Option<String>>(13)?,r.get::<_,Option<String>>(14)?))).map_err(storage)?;
        rows.map(|row| {
            let (previous,next,sequence,step_id,label,accepted,result_step,attempt_id,worker_id,host_id,phase,action_succeeded,observe_valid,unknown_reason,wait_reason) = row.map_err(storage)?;
            let step_declaration = match (step_id,label,accepted) { (Some(step_id),Some(label),Some(accepted)) => Some(StepDeclaration { step_id,label,accepted_sequence:u64::try_from(accepted).map_err(|_| Error::StorageUnavailable)? }), (None,None,None) => None, _ => return Err(Error::StorageUnavailable) };
            let attempt_result = match (result_step,attempt_id,worker_id,host_id,phase,action_succeeded,observe_valid,unknown_reason) {
                (Some(step_id),Some(attempt_id),Some(worker_instance_id),Some(host_session_id),Some(phase),action,observe,reason_value) => {
                    let conclusion = match (phase.as_str(),action,observe,reason_value.as_deref()) {
                        ("observed" | "stopped",Some(value),Some(1),None) => AttemptConclusion::Observed { action_succeeded:value != 0 },
                        ("unknown",None,Some(0),Some(value)) => AttemptConclusion::Unknown { reason:parse_reason(value)? },
                        _ => return Err(Error::StorageUnavailable),
                    };
                    Some(AttemptResultRecord { task_id:id.into(), step_id, attempt_id, worker_instance_id, host_session_id, conclusion, result_sequence:u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? })
                },
                (None,None,None,None,None,None,None,None) => None,
                _ => return Err(Error::StorageUnavailable),
            };
            Ok(TaskEventRecord { transition:Transition { previous:status(&previous)?, next:status(&next)?, sequence:u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? }, step_declaration, attempt_result, wait_reason })
        }).collect()
    }

    fn register(&mut self, owner: &str, key: &str, description: &str, task_name: Option<&str>) -> Result<Task, Error> {
        if [owner,key].iter().any(|id| id.is_empty() || id.len() > 128 || !id.bytes().all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))) || description.trim().is_empty() || description.len() > 4096 || task_name.is_some_and(|name| !yonder_application::valid_task_name(name)) {
            return Err(Error::InvalidInput);
        }
        let tx = self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let existing: Option<(String,String,String,i64,Option<String>,Option<String>)> = tx.query_row(
            "SELECT c.task_id,c.description,t.state,t.sequence,c.name,t.name FROM task_creations c JOIN tasks t ON t.id=c.task_id WHERE c.owner_agent_id=?1 AND c.idempotency_key=?2",
            params![owner,key], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)),
        ).optional().map_err(storage)?;
        let task = if let Some((id, original, state, sequence, original_name, current_name)) = existing {
            if original != description || original_name.as_deref() != task_name { return Err(Error::IdempotencyConflict); }
            Task { id, owner_agent_id: owner.into(), name: current_name, status: status(&state)?, sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? }
        } else {
            let id: String = tx.query_row("SELECT 'task_' || lower(hex(randomblob(16)))", [], |r| r.get(0)).map_err(storage)?;
            tx.execute("INSERT INTO tasks(id,owner_agent_id,state,sequence,name) VALUES (?1,?2,'created',1,?3)", params![id,owner,task_name]).map_err(storage)?;
            tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,1,'created','created')", [&id]).map_err(storage)?;
            tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,1)", [&id]).map_err(storage)?;
            tx.execute("INSERT INTO task_creations(owner_agent_id,idempotency_key,task_id,description,name) VALUES (?1,?2,?3,?4,?5)", params![owner,key,id,description,task_name]).map_err(storage)?;
            Task { id, owner_agent_id: owner.into(), name: task_name.map(str::to_owned), status: Status::Created, sequence: 1 }
        };
        tx.commit().map_err(storage)?;
        Ok(task)
    }

    fn list(&mut self, owner: Option<&str>, after: Option<&str>, include_finished: bool, limit: usize) -> Result<Vec<Task>, Error> {
        if !(1..=101).contains(&limit) { return Err(Error::InvalidInput); }
        let mut stmt = self.0.prepare("SELECT id,state,sequence,owner_agent_id,name FROM tasks WHERE id > ?1 COLLATE BINARY AND (?2 OR state NOT IN ('completed','failed','cancelled')) AND (?4 IS NULL OR owner_agent_id=?4) ORDER BY id COLLATE BINARY LIMIT ?3").map_err(storage)?;
        let rows = stmt.query_map(params![after.unwrap_or(""), include_finished, limit as i64, owner], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?, r.get::<_,String>(3)?, r.get::<_,Option<String>>(4)?))).map_err(storage)?;
        rows.map(|row| {
            let (id, state, sequence, owner_agent_id, name) = row.map_err(storage)?;
            Ok(Task { id, owner_agent_id, name, status: status(&state)?, sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? })
        }).collect()
    }
    fn list_running(&mut self, owner: Option<&str>, after: Option<&str>, limit: usize) -> Result<Vec<Task>, Error> {
        if !(1..=101).contains(&limit) { return Err(Error::InvalidInput); }
        let mut stmt = self.0.prepare("SELECT id,state,sequence,owner_agent_id,name FROM tasks WHERE id > ?1 COLLATE BINARY AND state='running' AND (?3 IS NULL OR owner_agent_id=?3) ORDER BY id COLLATE BINARY LIMIT ?2").map_err(storage)?;
        let rows = stmt.query_map(params![after.unwrap_or(""), limit as i64, owner], |r| Ok((r.get::<_,String>(0)?, r.get::<_,String>(1)?, r.get::<_,i64>(2)?, r.get::<_,String>(3)?, r.get::<_,Option<String>>(4)?))).map_err(storage)?;
        rows.map(|row| {
            let (id, state, sequence, owner_agent_id, name) = row.map_err(storage)?;
            Ok(Task { id, owner_agent_id, name, status: status(&state)?, sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? })
        }).collect()
    }
    fn create(&mut self, id: &str, owner_agent_id: &str) -> Result<Task, Error> {
        if [id, owner_agent_id].iter().any(|id| id.is_empty() || id.len() > 128 || !id.bytes().all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))) {
            return Err(Error::InvalidInput);
        }
        let tx = self.0.transaction_with_behavior(TransactionBehavior::Immediate).map_err(storage)?;
        let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM tasks WHERE id=?1)", [id], |r| r.get(0)).map_err(storage)?;
        if exists { return Err(Error::Conflict); }
        tx.execute("INSERT INTO tasks(id,owner_agent_id,state,sequence) VALUES (?1,?2,'created',1)", params![id,owner_agent_id]).map_err(storage)?;
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,1,'created','created')", [id]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,1)", [id]).map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Task { id: id.into(), owner_agent_id: owner_agent_id.into(), name: None, status: Status::Created, sequence: 1 })
    }

    fn get(&mut self, id: &str) -> Result<Task, Error> {
        let row: Option<(String,i64,String,Option<String>)> = self.0.query_row("SELECT state,sequence,owner_agent_id,name FROM tasks WHERE id=?1", [id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(storage)?;
        let (state, sequence, owner_agent_id, name) = row.ok_or(Error::NotFound)?;
        Ok(Task { id: id.into(), owner_agent_id, name, status: status(&state)?, sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? })
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
        let mut stmt = self.0.prepare("SELECT id,sequence,owner_agent_id,name FROM tasks WHERE state='running' ORDER BY id LIMIT ?1").map_err(storage)?;
        let rows = stmt.query_map([limit as i64], |r| Ok((r.get::<_,String>(0)?, r.get::<_,i64>(1)?, r.get::<_,String>(2)?, r.get::<_,Option<String>>(3)?))).map_err(storage)?;
        rows.map(|row| {
            let (id, sequence, owner_agent_id, name) = row.map_err(storage)?;
            Ok(Task { id, owner_agent_id, name, status: Status::Running, sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? })
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
        tx.execute("INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,?3,?4)", params![id,change.sequence as i64,name(change.previous),name(change.next)]).map_err(storage)?;
        tx.execute("INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)", params![id,change.sequence as i64]).map_err(storage)?;
        tx.commit().map_err(storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yonder_application::{Action, AuthContext, transition};

    fn create(store: &mut impl TaskStore, id: &str) -> Result<Task, Error> {
        yonder_application::create(store, id, AuthContext::Agent("a1"))
    }


    #[test]
    fn agent_names_are_versioned_persistent_idempotent_and_bounded() {
        use yonder_application::gateway::{GatewaySession, Platform};
        use yonder_protocol::{QueryResult, Response};
        let mut store = SqliteTaskStore::initialize(Connection::open_in_memory().unwrap(), true).unwrap();
        let mut session = GatewaySession::new(AuthContext::Agent("a1"), Platform::Macos);
        let hello = |minor| format!(r#"{{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{{"major":1,"minor":{minor}}}}}}}"#);
        let create = |name: Option<&str>| {
            let extra = name.map(|name| format!(",\"name\":\"{name}\"")).unwrap_or_default();
            format!(r#"{{"jsonrpc":"2.0","id":"c","method":"task.create","params":{{"agent_id":"a1","capability":"task.create","deadline":2000,"idempotency_key":"named","description":"test"{extra}}}}}"#)
        };
        let failure = |response: Response, code| assert!(matches!(response, Response::Failure { error, .. } if error.code == code));
        session.handle(&mut store, hello(2).as_bytes(), 1000);
        failure(session.handle(&mut store, create(Some("整理文件")).as_bytes(), 1000), -32010);
        session.handle(&mut store, hello(3).as_bytes(), 1000);
        failure(session.handle(&mut store, create(None).as_bytes(), 1000), -32602);
        for name in ["", " ", &"龙".repeat(86)] {
            failure(session.handle(&mut store, create(Some(name)).as_bytes(), 1000), -32602);
        }
        assert!(!yonder_application::valid_task_name("a\n"));
        let first = match session.handle(&mut store, create(Some("整理文件")).as_bytes(), 1000) {
            Response::Success { result: QueryResult::Snapshot { task }, .. } => task,
            other => panic!("{other:?}"),
        };
        assert_eq!(first.name.as_deref(), Some("整理文件"));
        let task = yonder_application::cancel_pending(&mut store, AuthContext::LocalUser("desktop"), &first.task_id, 1).unwrap();
        assert_eq!(task.name, first.name);
        failure(session.handle(&mut store, create(Some("其他名称")).as_bytes(), 1000), -32009);
        assert!(matches!(session.handle(&mut store, create(Some("整理文件")).as_bytes(), 1000), Response::Success { result: QueryResult::Snapshot { task }, .. } if task.name == first.name && task.status == yonder_protocol::TaskStatus::Cancelled));
        let get = format!(r#"{{"jsonrpc":"2.0","id":"g","method":"task.get","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"{}"}}}}"#, first.task_id);
        let list = br#"{"jsonrpc":"2.0","id":"l","method":"task.list","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"include_finished":true,"limit":20}}"#;
        for bytes in [get.as_bytes(), list.as_slice()] {
            let encoded = String::from_utf8(session.handle_encoded(&mut store, bytes, 1000).unwrap()).unwrap();
            assert!(encoded.contains("整理文件"));
        }
        for minor in [0,1,2] {
            session.handle(&mut store, hello(minor).as_bytes(), 1000);
            for bytes in [get.as_bytes(), list.as_slice()] {
                let encoded = String::from_utf8(session.handle_encoded(&mut store, bytes, 1000).unwrap()).unwrap();
                assert!(!encoded.contains("\"name\""));
            }
        }
        assert_eq!(store.get(&first.task_id).unwrap().name, first.name);
        assert_eq!(store.events(&first.task_id, 0, 100).unwrap().len(), 2);
        assert_eq!(store.0.query_row("SELECT count(*) FROM outbox", [], |r| r.get::<_,i64>(0)).unwrap(), 2);
    }

    #[test]
    fn name_migration_backs_up_preserves_legacy_and_rolls_back_failed_ddl() {
        let directory = std::env::temp_dir().join(format!("yonder-name-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let db = Connection::open(&path).unwrap();
        db.execute_batch(include_str!("task_schema.sql")).unwrap();
        db.execute_batch(include_str!("task_registration_schema.sql")).unwrap();
        db.execute_batch("INSERT INTO tasks VALUES ('old','a1','cancelled',2); INSERT INTO events VALUES ('old',1,'created','created'),('old',2,'created','cancelled'); INSERT INTO outbox VALUES ('old',1,1),('old',2,0); INSERT INTO task_creations VALUES ('a1','old','old','retained');").unwrap();
        drop(db);
        let mut store = SqliteTaskStore::open_unencrypted(&path).unwrap();
        assert_eq!(store.get("old").unwrap().name, None);
        assert_eq!(store.get("old").unwrap().status, Status::Cancelled);
        assert_eq!(store.events("old",0,100).unwrap().len(),2);
        let backups: Vec<_> = std::fs::read_dir(&directory).unwrap().map(|e| e.unwrap().path()).filter(|p| p != &path).collect();
        assert_eq!(backups.len(),1);
        let backup = Connection::open(&backups[0]).unwrap();
        assert_eq!(backup.query_row("PRAGMA user_version", [], |r| r.get::<_,i64>(0)).unwrap(),3);
        assert_eq!(backup.query_row("SELECT description FROM task_creations", [], |r| r.get::<_,String>(0)).unwrap(),"retained");
        assert_eq!(backup.query_row("SELECT sum(delivered) FROM outbox", [], |r| r.get::<_,i64>(0)).unwrap(),1);
        drop(backup);
        let named = yonder_application::register(&mut store, AuthContext::Agent("a1"), "new", "test", Some("整理文件")).unwrap();
        drop(store);
        let mut store = SqliteTaskStore::open_unencrypted(&path).unwrap();
        assert_eq!(store.get(&named.id).unwrap(),named);
        assert_eq!(std::fs::read_dir(&directory).unwrap().count(),2);
        drop(store);
        let fail_path = directory.join("failed.db");
        let db = Connection::open(&fail_path).unwrap();
        db.execute_batch(include_str!("task_schema.sql")).unwrap();
        db.execute_batch(include_str!("task_registration_schema.sql")).unwrap();
        db.execute_batch("ALTER TABLE task_creations ADD COLUMN name TEXT;").unwrap();
        drop(db);
        assert!(SqliteTaskStore::open_unencrypted(&fail_path).is_err());
        let db = Connection::open(&fail_path).unwrap();
        assert_eq!(db.query_row("PRAGMA user_version", [], |r| r.get::<_,i64>(0)).unwrap(),3);
        assert_eq!(db.query_row("SELECT count(*) FROM pragma_table_info('tasks') WHERE name='name'", [], |r| r.get::<_,i64>(0)).unwrap(),0);
        drop(db);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn unused_experimental_format_rolls_back_without_losing_task_data() {
        let directory = std::env::temp_dir().join(format!("yonder-retain-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let mut store = SqliteTaskStore::open_unencrypted(&path).unwrap();
        let task = yonder_application::register(&mut store, AuthContext::Agent("a1"), "one", "保留任务说明", None).unwrap();
        let cancelled = yonder_application::cancel_pending(&mut store, AuthContext::LocalUser("desktop"), &task.id, 1).unwrap();
        drop(store);
        let db = Connection::open(&path).unwrap();
        let experimental = "DROP TABLE task_browser_refs; DROP TABLE task_controls; DROP TABLE task_attempts; DROP TABLE task_steps; ALTER TABLE tasks DROP COLUMN name; ALTER TABLE task_creations DROP COLUMN name; ALTER TABLE events DROP COLUMN wait_reason; ALTER TABLE tasks ADD COLUMN deleted INTEGER NOT NULL DEFAULT 0 CHECK(deleted IN (0,1)); ALTER TABLE events ADD COLUMN kind TEXT NOT NULL DEFAULT 'transition' CHECK(kind IN ('transition','deleted')); PRAGMA user_version=4;";
        db.execute_batch(experimental).unwrap();
        drop(db);
        let mut store = SqliteTaskStore::open_unencrypted(&path).unwrap();
        assert_eq!(store.get(&task.id), Ok(cancelled.clone()));
        assert_eq!(store.events(&task.id,0,100).unwrap().len(),2);
        assert_eq!(store.0.query_row("SELECT count(*) FROM outbox WHERE task_id=?1", [&task.id], |r| r.get::<_,i64>(0)).unwrap(),2);
        assert_eq!(store.0.query_row("SELECT description FROM task_creations WHERE task_id=?1", [&task.id], |r| r.get::<_,String>(0)).unwrap(),"保留任务说明");
        assert_eq!(yonder_application::register(&mut store, AuthContext::Agent("a1"), "one", "保留任务说明", None),Ok(cancelled));
        assert_eq!(store.0.query_row("PRAGMA user_version", [], |r| r.get::<_,i64>(0)).unwrap(),14);
        drop(store);
        for rejected in ["UPDATE tasks SET deleted=1;", "CREATE TRIGGER retain_kind AFTER INSERT ON events BEGIN SELECT NEW.kind; END;"] {
            let db = Connection::open(&path).unwrap();
            db.execute_batch(experimental).unwrap();
            db.execute_batch(rejected).unwrap();
            drop(db);
            let before = std::fs::read(&path).unwrap();
            assert!(SqliteTaskStore::open_unencrypted(&path).is_err());
            assert_eq!(std::fs::read(&path).unwrap(),before);
            let db = Connection::open(&path).unwrap();
            db.execute_batch("UPDATE tasks SET deleted=0; DROP TRIGGER IF EXISTS retain_kind;").unwrap();
            drop(db);
            drop(SqliteTaskStore::open_unencrypted(&path).unwrap());
        }
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn pending_cancel_is_authorized_versioned_atomic_and_does_not_resurrect() {
        use yonder_application::{cancel_pending, register};
        use yonder_application::gateway::{GatewaySession, Platform};
        use yonder_protocol::Response;
        let path = std::env::temp_dir().join(format!("yonder-cancel-{}-{}.db", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let mut store = SqliteTaskStore::open_unencrypted(&path).unwrap();
        let first = register(&mut store, AuthContext::Agent("a1"), "one", "取消测试", None).unwrap();
        let other = register(&mut store, AuthContext::Agent("b2"), "two", "其他任务", None).unwrap();
        let request = format!(r#"{{"jsonrpc":"2.0","id":"cancel","method":"task.cancel","params":{{"agent_id":"a1","capability":"task.cancel","deadline":2000,"task_id":"{}","expected_sequence":"1"}}}}"#, first.id);
        let mut session = GatewaySession::new(AuthContext::Agent("a1"), Platform::Macos);
        let failed = |response: Response, code| assert!(matches!(response, Response::Failure { error, .. } if error.code == code));
        failed(session.handle(&mut store, request.as_bytes(), 1000), -32002);
        for minor in [0,1] {
            let hello = format!(r#"{{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{{"major":1,"minor":{minor}}}}}}}"#);
            session.handle(&mut store, hello.as_bytes(), 1000);
            failed(session.handle(&mut store, request.as_bytes(), 1000), -32010);
        }
        assert_eq!(cancel_pending(&mut store, AuthContext::Agent("b2"), &first.id, 1), Err(Error::NotFound));
        assert_eq!(cancel_pending(&mut store, AuthContext::Agent("a1"), &first.id, 9), Err(Error::Conflict));
        let hello = br#"{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":2}}}"#;
        session.handle(&mut store, hello, 1000);
        failed(session.handle(&mut store, request.as_bytes(), 2000), -32001);
        store.0.execute_batch("CREATE TRIGGER reject_cancel BEFORE INSERT ON outbox WHEN NEW.sequence=2 BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
        failed(session.handle(&mut store, request.as_bytes(), 1000), -32603);
        assert_eq!(store.get(&first.id).unwrap(), first);
        assert_eq!(store.events(&first.id, 0, 100).unwrap().len(), 1);
        store.0.execute_batch("DROP TRIGGER reject_cancel;").unwrap();
        let response = session.handle(&mut store, request.as_bytes(), 1000);
        assert!(matches!(response, Response::Success { .. }));
        let cancelled = store.get(&first.id).unwrap();
        assert_eq!(cancelled.status, Status::Cancelled);
        assert_eq!(cancel_pending(&mut store, AuthContext::LocalUser("desktop"), &first.id, 1), Ok(cancelled.clone()));
        assert_eq!(cancel_pending(&mut store, AuthContext::Agent("a1"), &first.id, 2), Ok(cancelled.clone()));
        assert_eq!(store.events(&first.id, 0, 100).unwrap().len(), 2);
        assert_eq!(register(&mut store, AuthContext::Agent("a1"), "one", "取消测试", None), Ok(cancelled));
        assert_eq!(transition(&mut store, &first.id, 1, Action::Start), Err(Error::Conflict));
        transition(&mut store, &other.id, 1, Action::Start).unwrap();
        assert_eq!(cancel_pending(&mut store, AuthContext::LocalUser("desktop"), &other.id, 2), Err(Error::StopRequired));
        assert_eq!(store.get(&other.id).unwrap().status, Status::Running);
        let count: i64 = store.0.query_row("SELECT count(*) FROM outbox WHERE task_id=?1", [&first.id], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
        drop(store);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn registration_migrates_is_idempotent_isolated_and_atomic() {
        use yonder_application::gateway::{GatewaySession, Platform};
        use yonder_protocol::{QueryResult, Response};
        let path = std::env::temp_dir().join(format!("yonder-registration-{}-{}.db", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let db = Connection::open(&path).unwrap();
        db.execute_batch(include_str!("task_schema.sql")).unwrap();
        db.execute("INSERT INTO tasks(id,owner_agent_id,state,sequence) VALUES ('legacy','a1','created',1)", []).unwrap();
        drop(db);
        let mut store = SqliteTaskStore::open_unencrypted(&path).unwrap();
        assert_eq!(store.get("legacy").unwrap().sequence, 1);
        assert_eq!(store.0.query_row("PRAGMA user_version", [], |r| r.get::<_,i64>(0)).unwrap(), 14);
        fn hello(agent: &str, minor: u16) -> Vec<u8> {
            format!(r#"{{"jsonrpc":"2.0","id":"hello","method":"gateway.hello","params":{{"agent_id":"{agent}","capability":"task.read","deadline":2000,"protocol_version":{{"major":1,"minor":{minor}}}}}}}"#).into_bytes()
        }
        fn create(agent: &str, key: &str, description: &str) -> Vec<u8> {
            format!(r#"{{"jsonrpc":"2.0","id":"create","method":"task.create","params":{{"agent_id":"{agent}","capability":"task.create","deadline":2000,"idempotency_key":"{key}","description":"{description}"}}}}"#).into_bytes()
        }
        fn failed(response: Response, code: i32) {
            assert!(matches!(response, Response::Failure { error, .. } if error.code == code));
        }
        fn snapshot(response: Response) -> yonder_protocol::TaskSnapshot {
            match response { Response::Success { result: QueryResult::Snapshot { task }, .. } => task, _ => panic!("预期任务快照") }
        }
        let mut a = GatewaySession::new(AuthContext::Agent("a1"), Platform::Macos);
        let request = create("a1", "same", "本地 Agent 任务");
        failed(a.handle(&mut store, &request, 1000), -32002);
        a.handle(&mut store, &hello("a1", 0), 1000);
        failed(a.handle(&mut store, &request, 1000), -32010);
        a.handle(&mut store, &hello("a1", 1), 1000);
        failed(a.handle(&mut store, &request, 2000), -32001);
        failed(a.handle(&mut store, &create("b2", "same", "说明"), 1000), -32003);
        failed(a.handle(&mut store, &create("a1", "empty", " "), 1000), -32602);
        failed(a.handle(&mut store, &create("a1", "large", &"龙".repeat(1366)), 1000), -32602);
        let invalid = String::from_utf8(request.clone()).unwrap().replace("task.create\",\"deadline", "task.read\",\"deadline");
        failed(a.handle(&mut store, invalid.as_bytes(), 1000), -32602);
        let invalid = String::from_utf8(request.clone()).unwrap().replace("\"description\":", "\"task_id\":\"forged\",\"description\":");
        failed(a.handle(&mut store, invalid.as_bytes(), 1000), -32600);
        let mut user = GatewaySession::new(AuthContext::LocalUser("desktop"), Platform::Macos);
        user.handle(&mut store, &hello("desktop", 1), 1000);
        failed(user.handle(&mut store, &create("desktop", "same", "说明"), 1000), -32003);
        let first = snapshot(a.handle(&mut store, &request, 1000));
        assert!(first.task_id.starts_with("task_"));
        assert_eq!(first.sequence, "1");
        assert_eq!(snapshot(a.handle(&mut store, &request, 1000)), first);
        failed(a.handle(&mut store, &create("a1", "same", "修改说明"), 1000), -32009);
        transition(&mut store, &first.task_id, 1, Action::Start).unwrap();
        let replay = snapshot(a.handle(&mut store, &request, 1000));
        assert_eq!(replay.sequence, "2");
        assert_eq!(replay.status, yonder_protocol::TaskStatus::Running);
        let mut b = GatewaySession::new(AuthContext::Agent("b2"), Platform::Macos);
        b.handle(&mut store, &hello("b2", 1), 1000);
        let second = snapshot(b.handle(&mut store, &create("b2", "same", "本地 Agent 任务"), 1000));
        assert_ne!(first.task_id, second.task_id);
        assert_eq!(yonder_application::list(&mut store, AuthContext::Agent("b2"), None, true, 100).unwrap().tasks.len(), 1);
        let foreign = format!(r#"{{"jsonrpc":"2.0","id":"get","method":"task.get","params":{{"agent_id":"b2","capability":"task.read","deadline":2000,"task_id":"{}"}}}}"#, first.task_id);
        failed(b.handle(&mut store, foreign.as_bytes(), 1000), -32004);
        let counts = |store: &SqliteTaskStore| store.0.query_row("SELECT (SELECT count(*) FROM tasks),(SELECT count(*) FROM events),(SELECT count(*) FROM outbox),(SELECT count(*) FROM task_creations)", [], |r| Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?,r.get::<_,i64>(3)?))).unwrap();
        assert_eq!(counts(&store), (3,3,3,2));
        store.0.execute_batch("CREATE TRIGGER reject_registration BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
        failed(a.handle(&mut store, &create("a1", "rollback", "说明"), 1000), -32603);
        assert_eq!(counts(&store), (3,3,3,2));
        store.0.execute_batch("DROP TRIGGER reject_registration;").unwrap();
        drop(store);
        let mut reopened = SqliteTaskStore::open_unencrypted(&path).unwrap();
        assert_eq!(snapshot(a.handle(&mut reopened, &request, 1000)), replay);
        assert_eq!(counts(&reopened), (3,3,3,2));
        drop(reopened);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn unencrypted_storage_persists_recovers_rolls_back_and_preserves_rejected_files() {
        use yonder_application::{list, recover_running};
        let directory = std::env::temp_dir().join(format!("yonder-plain-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let mut store = SqliteTaskStore::open_unencrypted(&path).unwrap();
        create(&mut store, "a-task").unwrap();
        yonder_application::create(&mut store, "b-task", AuthContext::Agent("b2")).unwrap();
        transition(&mut store, "a-task", 1, Action::Start).unwrap();
        transition(&mut store, "b-task", 1, Action::Start).unwrap();
        store.0.execute_batch("CREATE TRIGGER reject_outbox BEFORE INSERT ON outbox WHEN NEW.sequence=3 BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
        assert_eq!(transition(&mut store, "a-task", 2, Action::Complete), Err(Error::StorageUnavailable));
        assert_eq!(store.get("a-task").unwrap().sequence, 2);
        assert_eq!(store.get("a-task").unwrap().status, Status::Running);
        assert_eq!(store.events("a-task", 0, 100).unwrap().len(), 2);
        let count: i64 = store.0.query_row("SELECT count(*) FROM outbox WHERE task_id='a-task'", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
        store.0.execute_batch("DROP TRIGGER reject_outbox;").unwrap();
        drop(store);
        assert!(std::fs::read(&path).unwrap().starts_with(b"SQLite format 3\0"));
        let mut store = SqliteTaskStore::open_unencrypted(&path).unwrap();
        let page = list(&mut store, AuthContext::LocalUser("desktop"), None, false, 1).unwrap();
        assert_eq!(page.tasks[0].id, "a-task");
        let second = list(&mut store, AuthContext::LocalUser("desktop"), page.next_after_task_id.as_deref(), false, 1).unwrap();
        assert_eq!(second.tasks[0].id, "b-task");
        assert!(second.next_after_task_id.is_none());
        assert_eq!(list(&mut store, AuthContext::Agent("a1"), None, true, 100).unwrap().tasks.len(), 1);
        assert_eq!(recover_running(&mut store, 100), Ok(2));
        assert_eq!(recover_running(&mut store, 100), Ok(0));
        assert_eq!(store.get("a-task").unwrap().status, Status::Interrupted);
        assert_eq!(store.events("a-task", 0, 100).unwrap().len(), 3);
        let count: i64 = store.0.query_row("SELECT count(*) FROM outbox WHERE task_id='a-task'", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 3);
        store.0.execute_batch("PRAGMA user_version=999;").unwrap();
        drop(store);
        let before = std::fs::read(&path).unwrap();
        assert!(SqliteTaskStore::open_unencrypted(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        let invalid = directory.join("invalid.db");
        std::fs::write(&invalid, b"not a task database").unwrap();
        assert!(SqliteTaskStore::open_unencrypted(&invalid).is_err());
        assert_eq!(std::fs::read(&invalid).unwrap(), b"not a task database");
        let encrypted = directory.join("encrypted.db");
        let mut original = SqlCipherTaskStore::open(&encrypted, &[53; 32]).unwrap();
        create(&mut original, "protected").unwrap();
        drop(original);
        let before = std::fs::read(&encrypted).unwrap();
        assert!(SqliteTaskStore::open_unencrypted(&encrypted).is_err());
        assert_eq!(std::fs::read(&encrypted).unwrap(), before);
        assert_eq!(SqlCipherTaskStore::open(&encrypted, &[53; 32]).unwrap().get("protected").unwrap().id, "protected");
        for file in [path, invalid, encrypted] { std::fs::remove_file(file).unwrap(); }
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn rest_reservation_checks_storage_and_blocks_start_without_writing() {
        use yonder_application::admission::{Admission, Denied, RestDenied, StartError, reserve_rest, start};
        let directory = std::env::temp_dir().join(format!("yonder-rest-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let mut store = SqlCipherTaskStore::open(&path, &[47; 32]).unwrap();
        let gate = Admission::new(1).unwrap();
        create(&mut store, "task").unwrap();
        assert_eq!(reserve_rest(&mut store, None).err(), Some(RestDenied::Unavailable));
        let rest = reserve_rest(&mut store, Some(&gate)).unwrap();
        assert_eq!(start(&mut store, &gate, "task", 1, &[]).err(), Some(StartError::Admission(Denied::PresentationBusy)));
        assert_eq!(store.get("task").unwrap().status, Status::Created);
        assert_eq!(store.events("task", 0, 100).unwrap().len(), 1);
        let count: i64 = store.0.query_row("SELECT count(*) FROM outbox", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        rest.release_after_wake().unwrap();
        let (_, execution) = start(&mut store, &gate, "task", 1, &[]).unwrap();
        assert_eq!(reserve_rest(&mut store, Some(&gate)).err(), Some(RestDenied::Busy));
        execution.release_after_stop().unwrap();
        // 即使没有瞬时占用，数据库 running 仍否决预约，且撤销未展示预约。
        assert_eq!(reserve_rest(&mut store, Some(&gate)).err(), Some(RestDenied::Busy));
        gate.try_acquire("probe", &[]).unwrap().release_after_stop().unwrap();
        transition(&mut store, "task", 2, Action::Interrupt).unwrap();
        store.0.execute_batch("ALTER TABLE tasks RENAME TO unavailable_tasks;").unwrap();
        assert_eq!(reserve_rest(&mut store, Some(&gate)).err(), Some(RestDenied::Unavailable));
        gate.try_acquire("probe", &[]).unwrap().release_after_stop().unwrap();
        store.0.execute_batch("ALTER TABLE unavailable_tasks RENAME TO tasks;").unwrap();
        reserve_rest(&mut store, Some(&gate)).unwrap().release_after_wake().unwrap();
        drop(store);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn activity_includes_unreleased_background_work_and_unknown_sources() {
        use yonder_application::{activity_state, ActivityState};
        use yonder_application::admission::{Admission, start};
        let directory = std::env::temp_dir().join(format!("yonder-activity-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let mut store = SqlCipherTaskStore::open(&path, &[43; 32]).unwrap();
        let gate = Admission::new(2).unwrap();
        assert_eq!(activity_state(&mut store, None), ActivityState::Unknown);
        assert_eq!(activity_state(&mut store, Some(&gate)), ActivityState::NoKnownWork);
        create(&mut store, "background").unwrap();
        let (_, permit) = start(&mut store, &gate, "background", 1, &[]).unwrap();
        assert_eq!(activity_state(&mut store, Some(&gate)), ActivityState::Busy);
        // 模拟终态已提交、停止确认后的显式释放尚未完成这一窗口。
        transition(&mut store, "background", 2, Action::Complete).unwrap();
        for _ in 0..2 { assert_eq!(activity_state(&mut store, Some(&gate)), ActivityState::Busy); }
        assert_eq!(store.get("background").unwrap().sequence, 3);
        assert_eq!(store.events("background", 0, 100).unwrap().len(), 3);
        let count: i64 = store.0.query_row("SELECT count(*) FROM outbox", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 3);
        permit.release_after_stop().unwrap();
        assert_eq!(activity_state(&mut store, Some(&gate)), ActivityState::NoKnownWork);
        // 无占用也不能忽略持久化 running（例如恢复前的遗留状态）。
        create(&mut store, "persisted").unwrap();
        transition(&mut store, "persisted", 1, Action::Start).unwrap();
        assert_eq!(activity_state(&mut store, Some(&gate)), ActivityState::Busy);
        assert_eq!(activity_state(&mut store, None), ActivityState::Unknown);
        transition(&mut store, "persisted", 2, Action::Interrupt).unwrap();
        store.0.execute_batch("ALTER TABLE tasks RENAME TO unavailable_tasks;").unwrap();
        assert_eq!(activity_state(&mut store, Some(&gate)), ActivityState::Unknown);
        let permit = gate.try_acquire("unknown-execution", &[]).unwrap();
        assert_eq!(activity_state(&mut store, Some(&gate)), ActivityState::Busy);
        drop(permit); // unknown 不得通过丢弃凭证清除。
        store.0.execute_batch("ALTER TABLE unavailable_tasks RENAME TO tasks;").unwrap();
        assert_eq!(activity_state(&mut store, Some(&gate)), ActivityState::Busy);
        drop(store);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn global_running_state_ignores_pages_and_owners_and_fails_unknown() {
        use yonder_application::{list, recover_running, running_state, RunningState};
        let directory = std::env::temp_dir().join(format!("yonder-running-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let key = [41; 32]; // 仅合成测试库。
        let mut writer = SqlCipherTaskStore::open(&path, &key).unwrap();
        let mut reader = SqlCipherTaskStore::open(&path, &key).unwrap();
        assert_eq!(running_state(&mut reader), RunningState::NoRunningTask);
        for index in 0..105 { create(&mut writer, &format!("t-{index:03}")).unwrap(); }
        yonder_application::create(&mut writer, "z-other", AuthContext::Agent("other")).unwrap();
        transition(&mut writer, "z-other", 1, Action::Start).unwrap();
        let page = list(&mut reader, AuthContext::LocalUser("local"), None, false, 100).unwrap();
        assert_eq!(page.tasks.len(), 100);
        assert!(page.tasks.iter().all(|task| task.status != Status::Running));
        let facts = || writer.0.query_row(
            "SELECT (SELECT sum(sequence) FROM tasks), (SELECT count(*) FROM events), (SELECT count(*) FROM outbox)",
            [], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?)),
        ).unwrap();
        let before = facts();
        for _ in 0..3 { assert_eq!(running_state(&mut reader), RunningState::Running); }
        assert_eq!(facts(), before);
        transition(&mut writer, "z-other", 2, Action::Complete).unwrap();
        assert_eq!(running_state(&mut reader), RunningState::NoRunningTask);
        transition(&mut writer, "t-104", 1, Action::Start).unwrap();
        drop(reader);
        let mut reader = SqlCipherTaskStore::open(&path, &key).unwrap();
        assert_eq!(running_state(&mut reader), RunningState::Running);
        assert_eq!(recover_running(&mut writer, 1), Ok(1));
        assert_eq!(running_state(&mut reader), RunningState::NoRunningTask);
        // 只在合成库注入真实读取失败；不得返回上次成功的“没有运行任务”。
        writer.0.execute_batch("ALTER TABLE tasks RENAME TO unavailable_tasks;").unwrap();
        assert_eq!(running_state(&mut reader), RunningState::Unknown);
        writer.0.execute_batch("ALTER TABLE unavailable_tasks RENAME TO tasks;").unwrap();
        transition(&mut writer, "t-103", 1, Action::Start).unwrap();
        assert_eq!(running_state(&mut reader), RunningState::Running);
        drop(reader); drop(writer);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn stopped_task_finishes_before_releasing_and_failure_keeps_permit() {
        use yonder_application::admission::{Admission, Denied, FinishError, Outcome, Resource, start};
        let directory = std::env::temp_dir().join(format!("yonder-finish-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let mut store = SqlCipherTaskStore::open(&path, &[37; 32]).unwrap();
        let mut reader = SqlCipherTaskStore::open(&path, &[37; 32]).unwrap();
        let gate = Admission::new(2).unwrap();
        for id in ["first", "other"] { create(&mut store, id).unwrap(); }
        let (_, permit) = start(&mut store, &gate, "first", 1, &[Resource::Browser]).unwrap();
        let (other, other_permit) = start(&mut store, &gate, "other", 1, &[Resource::Desktop]).unwrap();
        let permit = match permit.finish_after_stop(&mut store, 1, Outcome::Completed) {
            Err(FinishError::Task { error: Error::Conflict, permit }) => permit,
            _ => panic!("旧序号应保留凭证"),
        };
        store.0.execute_batch("CREATE TRIGGER fail_finish BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        let permit = match permit.finish_after_stop(&mut store, 2, Outcome::Completed) {
            Err(FinishError::Task { error: Error::StorageUnavailable, permit }) => permit,
            _ => panic!("事务失败应保留凭证"),
        };
        assert_eq!(gate.try_acquire("blocked", &[Resource::Browser]).err(), Some(Denied::ResourceBusy(Resource::Browser)));
        assert_eq!(reader.get("first").unwrap().status, Status::Running);
        assert!(reader.events("first", 2, 100).unwrap().is_empty());
        let count: i64 = reader.0.query_row("SELECT count(*) FROM outbox WHERE task_id='first'", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);
        store.0.execute_batch("DROP TRIGGER fail_finish;").unwrap();
        let completed = match permit.finish_after_stop(&mut store, 2, Outcome::Completed) { Ok(task) => task, _ => panic!("终态应成功提交") };
        assert_eq!(reader.get("first").unwrap(), completed);
        assert_eq!((completed.status, completed.sequence), (Status::Completed, 3));
        assert_eq!(reader.events("first", 2, 100).unwrap().len(), 1);
        let count: i64 = reader.0.query_row("SELECT count(*) FROM outbox WHERE task_id='first'", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 3);
        gate.try_acquire("next", &[Resource::Browser]).unwrap().release_after_stop().unwrap();
        assert_eq!(reader.get("other").unwrap(), other);
        assert_eq!(gate.try_acquire("blocked", &[Resource::Desktop]).err(), Some(Denied::ResourceBusy(Resource::Desktop)));
        let failed = match other_permit.finish_after_stop(&mut store, 2, Outcome::Failed) { Ok(task) => task, _ => panic!("失败结果应持久化") };
        assert_eq!(reader.get("other").unwrap(), failed);
        assert_eq!(failed.status, Status::Failed);
        gate.try_acquire("next", &[Resource::Desktop]).unwrap().release_after_stop().unwrap();
        drop(reader); drop(store);
        std::fs::remove_file(path).unwrap(); std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn admitted_start_commits_before_return_and_releases_only_failed_attempt() {
        use yonder_application::admission::{Admission, Denied, Resource, StartError, start};
        let directory = std::env::temp_dir().join(format!("yonder-start-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let mut store = SqlCipherTaskStore::open(&path, &[31; 32]).unwrap();
        let mut reader = SqlCipherTaskStore::open(&path, &[31; 32]).unwrap();
        for id in ["first", "second", "third"] { create(&mut store, id).unwrap(); }
        let gate = Admission::new(2).unwrap();
        let file = Resource::File("file-1".into());
        let (first, permit) = start(&mut store, &gate, "first", 1, &[file.clone()]).unwrap();
        assert_eq!(reader.get("first").unwrap(), first);
        assert_eq!((first.status, first.sequence), (Status::Running, 2));
        assert_eq!(reader.events("first", 1, 100).unwrap().len(), 1);
        assert_eq!(start(&mut store, &gate, "second", 1, &[file.clone()]).err(), Some(StartError::Admission(Denied::ResourceBusy(file.clone()))));
        assert_eq!(reader.get("second").unwrap().status, Status::Created);

        store.0.execute_batch("CREATE TRIGGER fail_start_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        assert_eq!(start(&mut store, &gate, "second", 1, &[Resource::Browser]).err(), Some(StartError::Task(Error::StorageUnavailable)));
        let second = reader.get("second").unwrap();
        assert_eq!((second.status, second.sequence), (Status::Created, 1));
        assert!(reader.events("second", 1, 100).unwrap().is_empty());
        let count: i64 = reader.0.query_row("SELECT count(*) FROM outbox WHERE task_id='second'", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
        // 仅失败申请被释放；first 仍占用文件，Browser 与剩余容量可再次使用。
        assert_eq!(gate.try_acquire("blocked", &[file.clone()]).err(), Some(Denied::ResourceBusy(file)));
        gate.try_acquire("probe", &[Resource::Browser]).unwrap().release_after_stop().unwrap();
        store.0.execute_batch("DROP TRIGGER fail_start_outbox;").unwrap();
        assert_eq!(start(&mut store, &gate, "second", 0, &[Resource::Browser]).err(), Some(StartError::Task(Error::Conflict)));
        let (_, second_permit) = start(&mut store, &gate, "second", 1, &[Resource::Browser]).unwrap();
        assert_eq!(reader.running(100).unwrap().len(), 2);
        assert_eq!(start(&mut store, &gate, "third", 1, &[]).err(), Some(StartError::Admission(Denied::Capacity)));
        transition(&mut store, "second", 2, Action::Interrupt).unwrap();
        // 写 interrupted 不等于执行已停止，不能隐式释放。
        assert_eq!(gate.try_acquire("blocked", &[Resource::Browser]).err(), Some(Denied::ResourceBusy(Resource::Browser)));
        second_permit.release_after_stop().unwrap();
        assert!(matches!(start(&mut store, &gate, "second", 3, &[Resource::Browser]).err(), Some(StartError::Task(Error::InvalidTransition(_)))));
        let (_, third_permit) = start(&mut store, &gate, "third", 1, &[Resource::Browser]).unwrap();
        assert_eq!(reader.get("first").unwrap(), first);
        third_permit.release_after_stop().unwrap();
        permit.release_after_stop().unwrap();
        drop(reader); drop(store);
        std::fs::remove_file(path).unwrap(); std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn task_ownership_applies_to_every_query_and_preserves_legacy_data() {
        use yonder_protocol::{QueryResult, Response};
        let directory = std::env::temp_dir().join(format!("yonder-owners-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let key = [29; 32];
        let mut store = SqlCipherTaskStore::open(&path, &key).unwrap();
        for (id, owner) in [("a-first", "b2"), ("b-middle", "a1"), ("c-last", "a1"), ("d-last", "b2")] {
            let task = yonder_application::create(&mut store, id, AuthContext::Agent(owner)).unwrap();
            assert_eq!(task.owner_agent_id, owner);
        }
        let list = br#"{"jsonrpc":"2.0","id":"r1","method":"task.list","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"limit":1}}"#;
        let mut session = yonder_application::gateway::GatewaySession::new(AuthContext::Agent("a1"), yonder_protocol::Platform::Macos);
        let hello = br#"{"jsonrpc":"2.0","id":"hello","method":"gateway.hello","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":0}}}"#;
        assert!(matches!(session.handle(&mut store, hello, 1000), Response::Success { result: QueryResult::Hello { .. }, .. }));
        match session.handle(&mut store, list, 1000) {
            Response::Success { result: QueryResult::Tasks { tasks, next_after_task_id }, .. } => {
                assert_eq!(tasks.len(), 1); assert_eq!(tasks[0].task_id, "b-middle");
                assert_eq!(tasks[0].owner_agent_id, "a1"); assert_eq!(next_after_task_id.as_deref(), Some("b-middle"));
            }
            response => panic!("归属筛选失败：{response:?}"),
        }
        let page = yonder_application::list(&mut store, AuthContext::Agent("a1"), Some("b-middle"), true, 1).unwrap();
        assert_eq!(page.tasks[0].id, "c-last"); assert!(page.next_after_task_id.is_none());
        let b = yonder_application::list(&mut store, AuthContext::Agent("b2"), None, true, 100).unwrap();
        assert_eq!(b.tasks.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), vec!["a-first", "d-last"]);
        let local = yonder_application::list(&mut store, AuthContext::LocalUser("desktop"), None, true, 100).unwrap();
        assert_eq!(local.tasks.len(), 4);
        match yonder_application::query::handle(&mut store, AuthContext::LocalUser("a1"), list, 1000) {
            Response::Success { result: QueryResult::Tasks { tasks, .. }, .. } => assert_eq!(tasks[0].task_id, "a-first"),
            response => panic!("本机用户总览失败：{response:?}"),
        }
        fn failure(response: Response) -> (Option<String>, i32) {
            match response { Response::Failure { id, error, .. } => (id, error.code), response => panic!("应拒绝：{response:?}") }
        }
        assert_eq!(failure(yonder_application::query::handle(&mut store, AuthContext::Agent("b2"), list, 1000)), (Some("r1".into()), -32003));
        for method in ["task.get", "task.events"] {
            for task_id in ["a-first", "missing", "b-middle"] {
                let extra = if method == "task.events" { ",\"after_sequence\":\"0\",\"limit\":100" } else { "" };
                let request = format!(r#"{{"jsonrpc":"2.0","id":"r1","method":"{method}","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"{task_id}"{extra}}}}}"#);
                let result = session.handle(&mut store, request.as_bytes(), 1000);
                if task_id == "b-middle" { assert!(matches!(result, Response::Success { .. })); }
                else { assert_eq!(failure(result), (Some("r1".into()), -32004)); }
                assert_eq!(failure(yonder_application::query::handle(&mut store, AuthContext::Agent("b2"), request.as_bytes(), 1000)), (Some("r1".into()), -32003));
            }
        }
        transition(&mut store, "b-middle", 1, Action::Start).unwrap();
        yonder_application::recover_running(&mut store, 100).unwrap();
        assert_eq!(store.get("b-middle").unwrap().owner_agent_id, "a1");
        drop(store);
        let mut store = SqlCipherTaskStore::open(&path, &key).unwrap();
        assert_eq!(store.get("b-middle").unwrap().owner_agent_id, "a1");
        // 合成真实 v1 布局，验证拒绝时数据库字节不变，不触碰用户文件。
        store.0.execute_batch("DROP INDEX tasks_owner_id; ALTER TABLE tasks DROP COLUMN owner_agent_id; PRAGMA user_version=1;").unwrap();
        drop(store);
        let before = std::fs::read(&path).unwrap();
        assert!(SqlCipherTaskStore::open(&path, &key).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        std::fs::remove_file(path).unwrap(); std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn task_list_pages_committed_states_without_duplicates_or_writes() {
        use yonder_application::{list, list_running};
        use yonder_protocol::{QueryResult, Response};
        let directory = std::env::temp_dir().join(format!("yonder-list-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("tasks.db");
        let mut writer = SqlCipherTaskStore::open(&path, &[23; 32]).unwrap();
        let mut reader = SqlCipherTaskStore::open(&path, &[23; 32]).unwrap();
        assert!(list(&mut reader, AuthContext::LocalUser("local-test"), None, false, 100).unwrap().tasks.is_empty());
        for index in (0..105).rev() { create(&mut writer, &format!("t-{index:03}")).unwrap(); }
        for (id, action) in [("t-000", Action::Complete), ("t-001", Action::Fail), ("t-002", Action::Cancel), ("t-003", Action::Pause), ("t-004", Action::WaitForUser), ("t-005", Action::Interrupt)] {
            transition(&mut writer, id, 1, Action::Start).unwrap();
            transition(&mut writer, id, 2, action).unwrap();
        }
        transition(&mut writer, "t-006", 1, Action::Start).unwrap();
        let running = list_running(&mut reader, AuthContext::LocalUser("local-test"), None, 1).unwrap();
        assert_eq!(running.tasks.iter().map(|task| task.id.as_str()).collect::<Vec<_>>(), vec!["t-006"]);
        assert!(running.next_after_task_id.is_none());
        let outbox_before: i64 = writer.0.query_row("SELECT count(*) FROM outbox", [], |r| r.get(0)).unwrap();
        let first = list(&mut reader, AuthContext::LocalUser("local-test"), None, false, 100).unwrap();
        assert_eq!(first.tasks.len(), 100);
        assert_eq!(first.tasks[0].id, "t-003");
        assert_eq!(first.next_after_task_id.as_deref(), Some("t-102"));
        assert_eq!(first.tasks[..4].iter().map(|t| t.status).collect::<Vec<_>>(), vec![Status::Paused, Status::WaitingForUser, Status::Interrupted, Status::Running]);
        let last = list(&mut reader, AuthContext::LocalUser("local-test"), first.next_after_task_id.as_deref(), false, 100).unwrap();
        assert_eq!(last.tasks.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(), vec!["t-103", "t-104"]);
        assert!(last.next_after_task_id.is_none());
        let exact = list(&mut reader, AuthContext::LocalUser("local-test"), Some("t-004"), false, 100).unwrap();
        assert_eq!(exact.tasks.len(), 100);
        assert!(exact.next_after_task_id.is_none());
        assert!(list(&mut reader, AuthContext::LocalUser("local-test"), Some("z"), false, 1).unwrap().tasks.is_empty());
        assert!(list(&mut reader, AuthContext::LocalUser("local-test"), Some("../bad"), false, 1).is_err());
        assert!(list(&mut reader, AuthContext::LocalUser("local-test"), None, false, 0).is_err());
        assert!(list(&mut reader, AuthContext::LocalUser("local-test"), None, false, 101).is_err());
        assert_eq!(list(&mut reader, AuthContext::LocalUser("local-test"), None, true, 1).unwrap().tasks[0].status, Status::Completed);
        // 游标前的新任务需从首页刷新，已提交终态立即从非终态筛选中消失。
        create(&mut writer, "a-new").unwrap();
        transition(&mut writer, "t-104", 1, Action::Start).unwrap();
        transition(&mut writer, "t-104", 2, Action::Complete).unwrap();
        assert_eq!(list(&mut reader, AuthContext::LocalUser("local-test"), Some("t-102"), false, 100).unwrap().tasks.len(), 1);
        let raw = br#"{"jsonrpc":"2.0","id":"list-1","method":"task.list","params":{"agent_id":"local-test","capability":"task.read","deadline":2000,"limit":1}}"#;
        match yonder_application::query::handle(&mut reader, AuthContext::LocalUser("local-test"), raw, 1000) {
            Response::Success { id, result: QueryResult::Tasks { tasks, next_after_task_id }, .. } => {
                assert_eq!(id, "list-1"); assert_eq!(tasks[0].task_id, "a-new"); assert_eq!(tasks[0].sequence, "1");
                assert_eq!(next_after_task_id.as_deref(), Some("a-new"));
            }
            response => panic!("列表响应不匹配：{response:?}"),
        }
        let filtered = br#"{"jsonrpc":"2.0","id":"list-running","method":"task.list","params":{"agent_id":"local-test","capability":"task.read","deadline":2000,"running_only":true,"limit":1}}"#;
        assert!(matches!(yonder_application::query::handle(&mut reader, AuthContext::LocalUser("local-test"), filtered, 1000), Response::Success { result: QueryResult::Tasks { tasks, next_after_task_id: None }, .. } if tasks.len() == 1 && tasks[0].task_id == "t-006"));

        use yonder_application::gateway::{GatewaySession, Platform};
        let hello = |minor| format!(r#"{{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{{"major":1,"minor":{minor}}}}}}}"#);
        let agent_filtered = br#"{"jsonrpc":"2.0","id":"list-running","method":"task.list","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"running_only":true,"limit":1}}"#;
        let mut old = GatewaySession::new(AuthContext::Agent("a1"), Platform::Macos);
        old.handle(&mut reader, hello(15).as_bytes(), 1000);
        assert!(matches!(old.handle(&mut reader, agent_filtered, 1000), Response::Failure { error, .. } if error.code == -32010));
        let mut current = GatewaySession::new(AuthContext::Agent("a1"), Platform::Macos);
        assert!(matches!(current.handle(&mut reader, hello(16).as_bytes(), 1000), Response::Success { result: QueryResult::Hello { protocol_version, .. }, .. } if protocol_version.minor == 16));
        assert!(matches!(current.handle(&mut reader, agent_filtered, 1000), Response::Success { result: QueryResult::Tasks { tasks, .. }, .. } if tasks.len() == 1 && tasks[0].task_id == "t-006"));
        let outbox_after: i64 = writer.0.query_row("SELECT count(*) FROM outbox", [], |r| r.get(0)).unwrap();
        assert_eq!(outbox_after, outbox_before + 3); // 只有创建及两次迁移追加事件。
        drop(reader); drop(writer);
        let mut reopened = SqlCipherTaskStore::open(&path, &[23; 32]).unwrap();
        assert_eq!(list(&mut reopened, AuthContext::LocalUser("local-test"), None, false, 1).unwrap().tasks[0].id, "a-new");
        drop(reopened);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }

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
            assert_eq!(store.get(id).unwrap(), Task { id: id.into(), owner_agent_id: "a1".into(), name: None, status: Status::Interrupted, sequence: 3 });
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
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn agent_step_declaration_is_versioned_idempotent_and_auditable() {
        use yonder_application::gateway::{GatewaySession, Platform};
        use yonder_protocol::{QueryResult, Response, TaskStatus};
        let mut store = SqliteTaskStore::initialize(Connection::open_in_memory().unwrap(), true).unwrap();
        let mut session = GatewaySession::new(AuthContext::Agent("a1"), Platform::Macos);
        let hello = |minor| format!(r#"{{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{{"major":1,"minor":{minor}}}}}}}"#);
        let create = r#"{"jsonrpc":"2.0","id":"c","method":"task.create","params":{"agent_id":"a1","capability":"task.create","deadline":2000,"idempotency_key":"step-task","description":"step test","name":"步骤测试"}}"#;
        session.handle(&mut store, hello(4).as_bytes(), 1000);
        let task = match session.handle(&mut store, create.as_bytes(), 1000) { Response::Success { result:QueryResult::Snapshot { task }, .. } => task, other => panic!("{other:?}") };
        let declare = |label: &str, expected: &str| format!(r#"{{"jsonrpc":"2.0","id":"d","method":"task.step.declare","params":{{"agent_id":"a1","capability":"task.step.declare","deadline":2000,"task_id":"{}","expected_sequence":"{expected}","step_id":"step-1","label":"{label}"}}}}"#, task.task_id);
        let first = match session.handle(&mut store, declare("打开文档", "1").as_bytes(), 1000) { Response::Success { result:QueryResult::Step { task, step:Some(step) }, .. } => (task,step), other => panic!("{other:?}") };
        assert_eq!((first.0.status,first.0.sequence.as_str(),first.1.accepted_sequence.as_str()),(TaskStatus::Created,"2","2"));
        assert!(matches!(session.handle(&mut store, declare("打开文档", "1").as_bytes(), 1000), Response::Success { result:QueryResult::Step { task, step:Some(_) }, .. } if task.sequence == "2"));
        assert!(matches!(session.handle(&mut store, declare("其他步骤", "2").as_bytes(), 1000), Response::Failure { error, .. } if error.code == -32013));
        let get = format!(r#"{{"jsonrpc":"2.0","id":"g","method":"task.step.get","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"{}"}}}}"#, task.task_id);
        assert!(matches!(session.handle(&mut store, get.as_bytes(), 1000), Response::Success { result:QueryResult::Step { step:Some(step), .. }, .. } if step.label == "打开文档"));
        let events = format!(r#"{{"jsonrpc":"2.0","id":"e","method":"task.events","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"{}","after_sequence":"0","limit":100}}}}"#, task.task_id);
        let encoded = String::from_utf8(session.handle_encoded(&mut store, events.as_bytes(), 1000).unwrap()).unwrap();
        assert!(encoded.contains("step_declaration"));
        session.handle(&mut store, hello(3).as_bytes(), 1000);
        assert!(!String::from_utf8(session.handle_encoded(&mut store, events.as_bytes(), 1000).unwrap()).unwrap().contains("step_declaration"));
        assert!(matches!(session.handle(&mut store, get.as_bytes(), 1000), Response::Failure { error, .. } if error.code == -32010));
        session.handle(&mut store, hello(4).as_bytes(), 1000);
        let cancelled = yonder_application::cancel_pending(&mut store, AuthContext::LocalUser("desktop"), &task.task_id, 2).unwrap();
        assert!(matches!(session.handle(&mut store, declare("打开文档", "1").as_bytes(), 1000), Response::Success { result:QueryResult::Step { task, .. }, .. } if task.status == TaskStatus::Cancelled));
        assert_eq!(store.events(&task.task_id,0,100).unwrap().len(),3);
        assert_eq!(store.0.query_row("SELECT count(*) FROM task_steps", [], |r| r.get::<_,i64>(0)).unwrap(),1);
        assert_eq!(cancelled.sequence,3);
        session.handle(&mut store, hello(4).as_bytes(), 1000);
        let second_create = create.replace("step-task", "atomic-task").replace("步骤测试", "原子回滚");
        let second = match session.handle(&mut store, second_create.as_bytes(), 1000) { Response::Success { result:QueryResult::Snapshot { task }, .. } => task, other => panic!("{other:?}") };
        store.0.execute_batch("CREATE TRIGGER fail_step_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'step failure'); END;").unwrap();
        let second_declare = declare("打开文档", "1").replace(&task.task_id, &second.task_id);
        assert!(matches!(session.handle(&mut store, second_declare.as_bytes(), 1000), Response::Failure { error, .. } if error.code == -32603));
        assert_eq!(store.get(&second.task_id).unwrap().sequence,1);
        assert_eq!(store.get_with_step(&second.task_id).unwrap().1,None);
        store.0.execute_batch("DROP TRIGGER fail_step_outbox;").unwrap();
    }

    #[test]
    fn execution_attempt_preparation_is_atomic_idempotent_and_step_bound() {
        use yonder_application::{AttemptConclusion, AttemptPhase, ControlKind, ExecutionAttempt, prepare_attempt, request_control, stop_at_boundary};
        use yonder_application::admission::BoundaryStopError;
        use yonder_application::computer_use::{ComputerAction, ComputerUsePort, DispatchOutcome, UnknownReason, WorkTarget, dispatch_prepared, record_dispatch_outcome};
        use yonder_application::admission::{Admission, Resource, start_attempt};
        let mut store = SqliteTaskStore::initialize(Connection::open_in_memory().unwrap(), true).unwrap();
        let task = store.register("a1","attempt-task","attempt",Some("执行尝试")).unwrap();
        let (task, _) = store.declare_step("a1",&task.id,task.sequence,"open-document","打开文档").unwrap();
        let requested = ExecutionAttempt { task_id:task.id.clone(), step_id:"open-document".into(), attempt_id:"attempt-1".into(), worker_instance_id:"worker-1".into(), host_session_id:"host-1".into(), phase:AttemptPhase::Prepared, accepted_sequence:0 };
        let gate = Admission::new(1).unwrap();
        let (running, accepted, permit) = start_attempt(&mut store,&gate,&requested,task.sequence,&[Resource::Desktop]).unwrap();
        assert_eq!((running.status,running.sequence,accepted.accepted_sequence),(Status::Running,3,3));
        assert_eq!(store.get_attempt(&task.id).unwrap(),Some(accepted.clone()));
        assert_eq!(prepare_attempt(&mut store,&requested,task.sequence).unwrap(),(running.clone(),accepted.clone()));
        let different = ExecutionAttempt { attempt_id:"attempt-2".into(), ..requested.clone() };
        assert_eq!(prepare_attempt(&mut store,&different,task.sequence),Err(Error::Conflict));
        assert_eq!(store.events(&task.id,0,100).unwrap().len(),3);

        let (observed, result) = record_dispatch_outcome(&mut store,&task.id,&accepted.attempt_id,DispatchOutcome::Known { action_succeeded:true,observation:None }).unwrap();
        assert_eq!((observed.sequence,result.result_sequence,result.conclusion),(4,4,AttemptConclusion::Observed { action_succeeded:true }));
        assert_eq!(store.get_attempt(&task.id).unwrap().unwrap().phase,AttemptPhase::Observed);
        assert_eq!(record_dispatch_outcome(&mut store,&task.id,&accepted.attempt_id,DispatchOutcome::Known { action_succeeded:true,observation:None }).unwrap(),(observed.clone(),result.clone()));
        assert_eq!(record_dispatch_outcome(&mut store,&task.id,&accepted.attempt_id,DispatchOutcome::Unknown(UnknownReason::TimedOut)),Err(Error::Conflict));
        assert_eq!(store.events(&task.id,0,100).unwrap().len(),4);
        assert_eq!(store.events_with_steps(&task.id,3,1).unwrap()[0].attempt_result,Some(result));
        for (minor,visible) in [(4,false),(5,true)] {
            let mut session = yonder_application::gateway::GatewaySession::new(AuthContext::Agent("a1"),yonder_application::gateway::Platform::Macos);
            let hello = format!(r#"{{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{{"major":1,"minor":{minor}}}}}}}"#);
            session.handle(&mut store,hello.as_bytes(),1000);
            let events = format!(r#"{{"jsonrpc":"2.0","id":"e","method":"task.events","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"{}","after_sequence":"3","limit":10}}}}"#,task.id);
            let encoded = String::from_utf8(yonder_protocol::encode(&session.handle(&mut store,events.as_bytes(),1000)).unwrap()).unwrap();
            assert_eq!(encoded.contains("attempt_result"),visible);
        }
        let stale = ExecutionAttempt { worker_instance_id:"old-worker".into(), ..accepted.clone() };
        let (controlled,control) = request_control(&mut store,AuthContext::LocalUser("desktop"),&task.id,observed.sequence,ControlKind::Takeover).unwrap();
        assert_eq!((controlled.status,controlled.sequence,control.phase),(Status::Running,5,ControlPhase::Pending));
        assert_eq!(request_control(&mut store,AuthContext::LocalUser("desktop"),&task.id,observed.sequence,ControlKind::Takeover).unwrap(),(controlled.clone(),control.clone()));
        assert_eq!(store.stop_attempt(&stale,controlled.sequence,"control_3",ControlKind::Takeover),Err(Error::Conflict));
        assert!(gate.has_occupancy().unwrap());
        let (paused,stop) = match permit.stop_at_boundary(&mut store,&accepted.attempt_id,ControlKind::Takeover) { Ok(value) => value, _ => panic!("observed应停止") };
        assert_eq!((paused.status,paused.sequence,stop.kind,stop.stop_sequence),(Status::Paused,6,ControlKind::Takeover,6));
        assert_eq!(store.get_attempt(&task.id).unwrap().unwrap().phase,AttemptPhase::Stopped);
        assert_eq!(stop_at_boundary(&mut store,&task.id,&accepted.attempt_id,ControlKind::Takeover).unwrap(),(paused.clone(),stop));
        assert_eq!(stop_at_boundary(&mut store,&task.id,&accepted.attempt_id,ControlKind::Cancel),Err(Error::StopRequired));
        use yonder_application::work_focus::{FocusFailure,FocusOutcome,WorkFocusPort,WorkRef,focus_after_takeover};
        struct Focus;
        impl WorkFocusPort for Focus {
            fn capture(&mut self,_:&ExecutionAttempt,_:&WorkTarget)->Result<WorkRef,FocusFailure>{Err(FocusFailure::ReferenceUnavailable)}
            fn focus(&mut self,_:&WorkRef)->FocusOutcome{FocusOutcome::Focused}
            fn release(&mut self,_:&WorkRef){}
        }
        let reference=WorkRef { work_ref_id:"work_3".into(),task_id:task.id.clone(),step_id:accepted.step_id.clone(),attempt_id:accepted.attempt_id.clone(),worker_instance_id:accepted.worker_instance_id.clone(),host_session_id:accepted.host_session_id.clone(),pid:1,window_id:1,process_start_seconds:1,process_start_microseconds:1 };
        assert_eq!(focus_after_takeover(&mut store,&mut Focus,&reference),Ok(FocusOutcome::Focused));
        assert_eq!(focus_after_takeover(&mut store,&mut Focus,&WorkRef { worker_instance_id:"old-worker".into(),..reference }),Err(Error::StopRequired));
        assert!(!gate.has_occupancy().unwrap());

        let no_step = store.register("a1","no-step","no step",Some("无步骤")).unwrap();
        let missing = ExecutionAttempt { task_id:no_step.id.clone(), ..requested.clone() };
        assert_eq!(prepare_attempt(&mut store,&missing,no_step.sequence),Err(Error::StopRequired));
        let rollback = store.register("a1","rollback-attempt","rollback",Some("回滚")).unwrap();
        let (rollback,_) = store.declare_step("a1",&rollback.id,rollback.sequence,"open-document","打开文档").unwrap();
        store.0.execute_batch("CREATE TRIGGER fail_attempt_outbox BEFORE INSERT ON outbox BEGIN SELECT RAISE(ABORT,'attempt failure'); END;").unwrap();
        let failed = ExecutionAttempt { task_id:rollback.id.clone(), attempt_id:"attempt-rollback".into(), ..requested };
        assert_eq!(prepare_attempt(&mut store,&failed,rollback.sequence),Err(Error::StorageUnavailable));
        assert_eq!((store.get(&rollback.id).unwrap().status,store.get_attempt(&rollback.id).unwrap()),(Status::Created,None));
        store.0.execute_batch("DROP TRIGGER fail_attempt_outbox;").unwrap();
        let gate = Admission::new(1).unwrap();
        let (_,accepted,unknown_permit) = start_attempt(&mut store,&gate,&failed,rollback.sequence,&[Resource::Desktop]).unwrap();
        store.0.execute_batch("CREATE TRIGGER fail_result_outbox BEFORE INSERT ON outbox WHEN NEW.task_id IN (SELECT task_id FROM task_attempts WHERE attempt_id='attempt-rollback') BEGIN SELECT RAISE(ABORT,'result failure'); END;").unwrap();
        assert_eq!(record_dispatch_outcome(&mut store,&rollback.id,&accepted.attempt_id,DispatchOutcome::Unknown(UnknownReason::TimedOut)),Err(Error::StorageUnavailable));
        assert_eq!((store.get(&rollback.id).unwrap().sequence,store.get_attempt(&rollback.id).unwrap().unwrap().phase),(3,AttemptPhase::Prepared));
        store.0.execute_batch("DROP TRIGGER fail_result_outbox;").unwrap();
        let (unknown_task,unknown) = record_dispatch_outcome(&mut store,&rollback.id,&accepted.attempt_id,DispatchOutcome::Unknown(UnknownReason::TimedOut)).unwrap();
        assert_eq!((unknown_task.sequence,unknown.conclusion,store.get_attempt(&rollback.id).unwrap().unwrap().phase),(4,AttemptConclusion::Unknown { reason:UnknownReason::TimedOut },AttemptPhase::Unknown));
        assert_eq!(yonder_application::advance_after_observe(&mut store,&rollback.id,&accepted.attempt_id),Err(Error::StopRequired));
        assert_eq!(request_control(&mut store,AuthContext::LocalUser("desktop"),&rollback.id,unknown_task.sequence,ControlKind::Pause),Err(Error::StopRequired));
        drop(unknown_permit);
        assert!(gate.has_occupancy().unwrap());

        let stop_task = store.register("a1","stop-rollback","stop rollback",Some("停止回滚")).unwrap();
        let (stop_task,_) = store.declare_step("a1",&stop_task.id,stop_task.sequence,"stop-step","停止步骤").unwrap();
        let stop_attempt = ExecutionAttempt { task_id:stop_task.id.clone(), step_id:"stop-step".into(), attempt_id:"stop-attempt".into(), worker_instance_id:"stop-worker".into(), host_session_id:"stop-host".into(), phase:AttemptPhase::Prepared, accepted_sequence:0 };
        let stop_gate = Admission::new(1).unwrap();
        let (_,stop_attempt,stop_permit) = start_attempt(&mut store,&stop_gate,&stop_attempt,stop_task.sequence,&[Resource::Desktop]).unwrap();
        struct MustNotDispatch;
        impl ComputerUsePort for MustNotDispatch { fn dispatch(&self,_:&ExecutionAttempt,_:&WorkTarget,_:&ComputerAction)->DispatchOutcome { panic!("pending控制后不得派发") } }
        request_control(&mut store,AuthContext::LocalUser("desktop"),&stop_task.id,3,ControlKind::Cancel).unwrap();
        assert_eq!(dispatch_prepared(&mut store,&MustNotDispatch,&stop_task.id,&stop_attempt.attempt_id,&WorkTarget { pid:1,window_id:1 },&ComputerAction{tool_name:"type_text".into(),arguments_json:r#"{"text":"x"}"#.into()}),Err(Error::StopRequired));
        let (stop_running,_) = record_dispatch_outcome(&mut store,&stop_task.id,&stop_attempt.attempt_id,DispatchOutcome::Known { action_succeeded:false,observation:None }).unwrap();
        store.0.execute_batch(&format!("CREATE TRIGGER fail_boundary_outbox BEFORE INSERT ON outbox WHEN NEW.task_id='{}' BEGIN SELECT RAISE(ABORT,'stop failure'); END;",stop_task.id)).unwrap();
        let stop_permit = match stop_permit.stop_at_boundary(&mut store,&stop_attempt.attempt_id,ControlKind::Cancel) { Err(BoundaryStopError::Task { error:Error::StorageUnavailable,permit }) => permit, _ => panic!("停止事务应回滚") };
        assert_eq!((store.get(&stop_task.id).unwrap().sequence,store.get_attempt(&stop_task.id).unwrap().unwrap().phase),(stop_running.sequence,AttemptPhase::Observed));
        assert!(stop_gate.has_occupancy().unwrap());
        store.0.execute_batch("DROP TRIGGER fail_boundary_outbox;").unwrap();
        let (cancelled,_) = match stop_permit.stop_at_boundary(&mut store,&stop_attempt.attempt_id,ControlKind::Cancel) { Ok(value) => value, _ => panic!("重试停止应提交") };
        assert_eq!(cancelled.status,Status::Cancelled);
        assert!(!stop_gate.has_occupancy().unwrap());
        assert_eq!(store.0.query_row("PRAGMA user_version",[],|r| r.get::<_,i64>(0)).unwrap(),14);
    }

    #[test]
    fn takeover_at_safe_boundary_persists_focus_without_recording() {
        use yonder_application::{AttemptPhase,ControlKind,ControlPhase,FocusPhase,ExecutionAttempt,AuthContext,computer_use::DispatchOutcome,work_focus::{FocusOutcome,WorkFocusPort,WorkRef,focus_takeover}};
        use yonder_application::admission::{Admission,Resource,start_attempt};
        let mut store=SqliteTaskStore::initialize(Connection::open_in_memory().unwrap(),true).unwrap();
        let task=store.register("a1","focus-task","focus",Some("接管定位")).unwrap();
        let (task,_)=store.declare_step("a1",&task.id,task.sequence,"edit","编辑内容").unwrap();
        let requested=ExecutionAttempt{task_id:task.id.clone(),step_id:"edit".into(),attempt_id:"attempt-focus".into(),worker_instance_id:"worker-focus".into(),host_session_id:"host-focus".into(),phase:AttemptPhase::Prepared,accepted_sequence:0};
        let gate=Admission::new(1).unwrap();
        let (_,attempt,_permit)=start_attempt(&mut store,&gate,&requested,task.sequence,&[Resource::Desktop]).unwrap();
        yonder_application::computer_use::record_dispatch_outcome(&mut store,&task.id,&attempt.attempt_id,DispatchOutcome::Known{action_succeeded:true,observation:None}).unwrap();
        let (boundary,_)=yonder_application::advance_after_observe(&mut store,&task.id,&attempt.attempt_id).unwrap();
        let (paused,control)=yonder_application::request_control(&mut store,AuthContext::LocalUser("desktop"),&task.id,boundary.sequence,ControlKind::Takeover).unwrap();
        assert_eq!((paused.status,control.phase),(Status::Paused,ControlPhase::Stopped));
        gate.release_task_after_stop(&task.id).unwrap();gate.set_desktop_taken_over(true).unwrap();
        struct Focus;impl WorkFocusPort for Focus{fn capture(&mut self,_:&ExecutionAttempt,_:&yonder_application::computer_use::WorkTarget)->Result<WorkRef,yonder_application::work_focus::FocusFailure>{unreachable!()}fn focus(&mut self,_:&WorkRef)->FocusOutcome{FocusOutcome::Focused}fn release(&mut self,_:&WorkRef){}}
        let reference=WorkRef{work_ref_id:format!("work_{}",attempt.accepted_sequence),task_id:task.id.clone(),step_id:attempt.step_id.clone(),attempt_id:attempt.attempt_id.clone(),worker_instance_id:attempt.worker_instance_id.clone(),host_session_id:attempt.host_session_id.clone(),pid:1,window_id:1,process_start_seconds:1,process_start_microseconds:1};
        let (focused,control,outcome)=focus_takeover(&mut store,&mut Focus,&reference).unwrap();
        assert_eq!((focused.status,control.focus_phase,control.focus_failure,outcome),(Status::Paused,Some(FocusPhase::Focused),None,FocusOutcome::Focused));
        assert!(gate.try_acquire("other",&[Resource::Desktop]).is_err());
        assert_eq!(store.events(&task.id,0,100).unwrap().len(),8);
    }

    #[test]
    fn start_execution_routes_created_and_running_tasks() {
        use yonder_application::{AttemptPhase, ExecutionAttempt, advance_after_observe};
        use yonder_application::admission::{Admission, Resource, start_execution};
        use yonder_application::computer_use::{DispatchOutcome, record_dispatch_outcome};
        let mut store = SqliteTaskStore::initialize(Connection::open_in_memory().unwrap(), true).unwrap();
        let task = store.register("a1","start-execution","start",Some("统一启动")).unwrap();
        let (task,_) = store.declare_step("a1",&task.id,task.sequence,"step-1","第一步").unwrap();
        let attempt = |step:&str,id:&str| ExecutionAttempt { task_id:task.id.clone(), step_id:step.into(), attempt_id:id.into(), worker_instance_id:"worker".into(), host_session_id:"host".into(), phase:AttemptPhase::Prepared, accepted_sequence:0 };
        let gate = Admission::new(1).unwrap();
        let (running, first) = start_execution(&mut store,&gate,&task,&attempt("step-1","attempt-1"),task.sequence,&[Resource::Desktop]).unwrap();
        assert_eq!((running.status,running.sequence,first.accepted_sequence),(Status::Running,task.sequence+1,task.sequence+1));
        assert_eq!(gate.holds_resource(&task.id,Resource::Desktop),Ok(true));
        let (task,_) = record_dispatch_outcome(&mut store,&task.id,&first.attempt_id,DispatchOutcome::Known { action_succeeded:true,observation:None }).unwrap();
        let (task,_) = advance_after_observe(&mut store,&task.id,&first.attempt_id).unwrap();
        let (task,_) = store.declare_step("a1",&task.id,task.sequence,"step-2","第二步").unwrap();
        let (next, second) = start_execution(&mut store,&gate,&task,&attempt("step-2","attempt-2"),task.sequence,&[Resource::Desktop]).unwrap();
        assert_eq!((next.status,next.sequence,second.accepted_sequence),(Status::Running,task.sequence+1,task.sequence+1));
        assert_eq!(gate.holds_resource(&task.id,Resource::Desktop),Ok(true));
    }

    #[test]
    fn browser_reference_commits_with_observe_and_survives_later_unknown() {
        use yonder_application::{AttemptPhase,ExecutionAttempt,advance_after_observe,prepare_next_attempt};
        use yonder_application::admission::{Admission,Resource,start_attempt};
        use yonder_application::browser_use::{BrowserAction,BrowserOutcome,BrowserTaskRef,BrowserUsePort,execute_agent_action,record_outcome};
        use yonder_application::computer_use::{DispatchOutcome,UnknownReason,record_dispatch_outcome};
        use yonder_application::gateway::{GatewaySession,Platform};
        use yonder_protocol::{QueryResult,Response};
        let mut store=SqliteTaskStore::initialize(Connection::open_in_memory().unwrap(),true).unwrap();
        let task=store.register("a1","browser-ref","browser",Some("浏览器引用")).unwrap();
        let (task,_)=store.declare_step("a1",&task.id,task.sequence,"create","创建空间").unwrap();
        let attempt=|step:&str,id:&str|ExecutionAttempt{task_id:task.id.clone(),step_id:step.into(),attempt_id:id.into(),worker_instance_id:"worker".into(),host_session_id:"host".into(),phase:AttemptPhase::Prepared,accepted_sequence:0};
        let gate=Admission::new(1).unwrap();
        let (_,first,_permit)=start_attempt(&mut store,&gate,&attempt("create","attempt-1"),task.sequence,&[Resource::Browser]).unwrap();
        let hello=br#"{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":15}}}"#;
        let get=format!(r#"{{"jsonrpc":"2.0","id":"g","method":"task.browser.get","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"{}"}}}}"#,task.id);
        let mut session=GatewaySession::new(AuthContext::Agent("a1"),Platform::Macos);
        assert!(matches!(session.handle(&mut store,hello,1000),Response::Success{result:QueryResult::Hello{protocol_version,..},..} if protocol_version.minor==15));
        assert!(matches!(session.handle(&mut store,get.as_bytes(),1000),Response::Success{result:QueryResult::BrowserState{reference:None,..},..}));
        let mut other=GatewaySession::new(AuthContext::Agent("a2"),Platform::Macos);
        let other_hello=br#"{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{"agent_id":"a2","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":15}}}"#;
        let other_get=get.replace("\"agent_id\":\"a1\"","\"agent_id\":\"a2\"");
        other.handle(&mut store,other_hello,1000);
        assert!(matches!(other.handle(&mut store,other_get.as_bytes(),1000),Response::Failure{error,..} if error.code==-32004));
        store.0.execute_batch("CREATE TRIGGER fail_browser_outbox BEFORE INSERT ON outbox WHEN NEW.task_id IN (SELECT task_id FROM task_attempts WHERE attempt_id='attempt-1') BEGIN SELECT RAISE(ABORT,'browser failure'); END;").unwrap();
        let outcome=BrowserOutcome::Observed(BrowserTaskRef{external_task_ref:"ego:41".into(),ownership:"agent".into(),managed_pages:1});
        assert_eq!(record_outcome(&mut store,&task.id,&first.attempt_id,&outcome),Err(Error::StorageUnavailable));
        assert_eq!(store.get_browser_reference(&task.id).unwrap(),None);
        store.0.execute_batch("DROP TRIGGER fail_browser_outbox;").unwrap();
        let (observed,_)=record_outcome(&mut store,&task.id,&first.attempt_id,&outcome).unwrap();
        let reference=store.get_browser_reference(&task.id).unwrap().unwrap();
        assert_eq!((reference.external_task_ref.as_str(),reference.ownership.as_str(),reference.managed_pages,reference.finished,reference.updated_sequence),("ego:41","agent",1,false,observed.sequence));
        assert!(matches!(session.handle(&mut store,get.as_bytes(),1000),Response::Success{result:QueryResult::BrowserState{reference:Some(value),..},..} if value.external_task_ref=="ego:41"&&value.updated_sequence==observed.sequence.to_string()));
        let (advanced,_)=advance_after_observe(&mut store,&task.id,&first.attempt_id).unwrap();
        let (declared,_)=store.declare_step("a1",&task.id,advanced.sequence,"handoff","交给用户").unwrap();
        struct Handoff; impl BrowserUsePort for Handoff { fn dispatch(&self,_:&ExecutionAttempt,action:&BrowserAction)->BrowserOutcome { match action { BrowserAction::HandOff{external_task_ref}=>BrowserOutcome::Observed(BrowserTaskRef{external_task_ref:external_task_ref.clone(),ownership:"agentDelegatedToUser".into(),managed_pages:1}), _=>BrowserOutcome::Unknown(UnknownReason::InvalidInput) } } }
        let (handed,reference)=execute_agent_action(&mut store,&gate,&Handoff,AuthContext::LocalUser("desktop"),&task.id,declared.sequence,"hand-off","host").unwrap();
        assert_eq!(reference.ownership,"agentDelegatedToUser");
        let current=store.get_attempt(&task.id).unwrap().unwrap();
        let (advanced,_)=advance_after_observe(&mut store,&task.id,&current.attempt_id).unwrap();
        let (declared,_)=store.declare_step("a1",&task.id,advanced.sequence,"finish","结束空间").unwrap();
        let (_,second)=prepare_next_attempt(&mut store,&attempt("finish","attempt-2"),declared.sequence).unwrap();
        let (finished_task,_)=record_outcome(&mut store,&task.id,&second.attempt_id,&BrowserOutcome::Finished{external_task_ref:"ego:41".into()}).unwrap();
        let finished=store.get_browser_reference(&task.id).unwrap().unwrap();
        assert!(finished.finished); assert_eq!(finished.updated_sequence,finished_task.sequence);
        let (advanced,_)=advance_after_observe(&mut store,&task.id,&second.attempt_id).unwrap();
        let (declared,_)=store.declare_step("a1",&task.id,advanced.sequence,"wait","等待").unwrap();
        let (_,third)=prepare_next_attempt(&mut store,&attempt("wait","attempt-3"),declared.sequence).unwrap();
        record_dispatch_outcome(&mut store,&task.id,&third.attempt_id,DispatchOutcome::Unknown(UnknownReason::TimedOut)).unwrap();
        assert_eq!(store.get_browser_reference(&task.id).unwrap().unwrap(),finished);
        assert_eq!(handed.status,Status::Running);
    }

    #[test]
    fn agent_cua_supervisor_completes_and_user_input_interrupts() {
        use yonder_application::{AttemptConclusion,AuthContext,advance_after_observe};
        use yonder_application::admission::{Admission,Resource};
        use yonder_application::computer_use::{ComputerAction,ComputerUsePort,DispatchOutcome,UnknownReason,WorkTarget,WorkTargetPort,complete_agent_task,execute_agent_action};
        use yonder_application::gateway::{GatewaySession,Platform};
        use yonder_protocol::{QueryResult,Response,TaskStatus};
        struct Target; impl WorkTargetPort for Target{fn frontmost(&self)->Result<WorkTarget,UnknownReason>{Ok(WorkTarget{pid:1,window_id:1})}}
        struct Port(DispatchOutcome); impl ComputerUsePort for Port{fn dispatch(&self,_:&ExecutionAttempt,_:&WorkTarget,_:&ComputerAction)->DispatchOutcome{self.0.clone()}}
        let mut store=SqliteTaskStore::initialize(Connection::open_in_memory().unwrap(),true).unwrap();let gate=Admission::new(2).unwrap();
        let task=store.register("a1","cua-complete","cua",Some("桌面动作")).unwrap();let(task,_)=store.declare_step("a1",&task.id,task.sequence,"type","输入文本").unwrap();
        let(observed,result,_)=execute_agent_action(&mut store,&gate,&Port(DispatchOutcome::Known{action_succeeded:true,observation:None}),&Target,AuthContext::Agent("a1"),&task.id,task.sequence,"type_text",r#"{"text":"fixed"}"#,"host").unwrap();
        assert_eq!(result.conclusion,AttemptConclusion::Observed{action_succeeded:true});assert!(gate.holds_resource(&task.id,Resource::Desktop).unwrap());
        let attempt=store.get_attempt(&task.id).unwrap().unwrap();let(advanced,_)=advance_after_observe(&mut store,&task.id,&attempt.attempt_id).unwrap();
        let completed=complete_agent_task(&mut store,&gate,AuthContext::Agent("a1"),&task.id,advanced.sequence).unwrap();assert_eq!((completed.status,gate.has_occupancy()),(Status::Completed,Ok(false)));
        let failed=store.register("a1","cua-fail","cua",Some("动作失败")).unwrap();let(failed,_)=store.declare_step("a1",&failed.id,failed.sequence,"click","点击按钮").unwrap();
        let(_,result,_)=execute_agent_action(&mut store,&gate,&Port(DispatchOutcome::Known{action_succeeded:false,observation:None}),&Target,AuthContext::Agent("a1"),&failed.id,failed.sequence,"click",r#"{"x":1,"y":1}"#,"host").unwrap();
        assert_eq!(result.conclusion,AttemptConclusion::Observed{action_succeeded:false});let attempt=store.get_attempt(&failed.id).unwrap().unwrap();let(boundary,_)=advance_after_observe(&mut store,&failed.id,&attempt.attempt_id).unwrap();
        assert_eq!(complete_agent_task(&mut store,&gate,AuthContext::Agent("a1"),&failed.id,boundary.sequence),Err(Error::StopRequired));
        let mut session=GatewaySession::new(AuthContext::Agent("a1"),Platform::Macos);
        let hello=br#"{"jsonrpc":"2.0","id":"hello","method":"gateway.hello","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":18}}}"#;
        let _=session.handle_encoded_with_runtimes(&mut store,&gate,None,Some(&Port(DispatchOutcome::Known{action_succeeded:false,observation:None})),Some(&Target),false,"host",hello,1000).unwrap();
        let request=format!(r#"{{"jsonrpc":"2.0","id":"fail","method":"task.fail","params":{{"agent_id":"a1","capability":"task.fail","deadline":2000,"task_id":"{}","expected_sequence":"{}"}}}}"#,failed.id,boundary.sequence);
        let (response,_)=session.handle_encoded_with_runtimes(&mut store,&gate,None,Some(&Port(DispatchOutcome::Known{action_succeeded:false,observation:None})),Some(&Target),false,"host",request.as_bytes(),1000).unwrap();
        assert!(matches!(yonder_protocol::decode_response(&response),Ok(Response::Success{result:QueryResult::Snapshot{task},..}) if task.status==TaskStatus::Failed));assert_eq!(gate.has_occupancy(),Ok(false));
        let interrupted=store.register("a1","cua-interrupt","cua",Some("用户输入")).unwrap();let(interrupted,_)=store.declare_step("a1",&interrupted.id,interrupted.sequence,"type","输入文本").unwrap();
        let(task,result,_)=execute_agent_action(&mut store,&gate,&Port(DispatchOutcome::Unknown(UnknownReason::UserInput)),&Target,AuthContext::Agent("a1"),&interrupted.id,interrupted.sequence,"type_text",r#"{"text":"fixed"}"#,"host").unwrap();
        assert_eq!((task.status,result.conclusion,gate.has_occupancy()),(Status::Interrupted,AttemptConclusion::Unknown{reason:UnknownReason::UserInput},Ok(false)));
        assert_eq!(observed.status,Status::Running);
    }

    #[test]
    fn observed_boundary_allows_next_step_without_releasing_task_permit() {
        use yonder_application::{AttemptPhase,ExecutionAttempt,advance_after_observe,prepare_next_attempt};
        use yonder_application::admission::{Admission,Outcome,Resource,start_attempt};
        use yonder_application::computer_use::{DispatchOutcome,record_dispatch_outcome};
        let mut store=SqliteTaskStore::initialize(Connection::open_in_memory().unwrap(),true).unwrap();
        let task=store.register("a1","next-step","next",Some("连续步骤")).unwrap();
        let (task,_)=store.declare_step("a1",&task.id,task.sequence,"step-1","第一步").unwrap();
        let attempt=|step:&str,id:&str|ExecutionAttempt{task_id:task.id.clone(),step_id:step.into(),attempt_id:id.into(),worker_instance_id:"worker".into(),host_session_id:"host".into(),phase:AttemptPhase::Prepared,accepted_sequence:0};
        let gate=Admission::new(1).unwrap();
        let (_,first,permit)=start_attempt(&mut store,&gate,&attempt("step-1","attempt-1"),task.sequence,&[Resource::Browser]).unwrap();
        let (observed,_)=record_dispatch_outcome(&mut store,&task.id,&first.attempt_id,DispatchOutcome::Known{action_succeeded:true,observation:None}).unwrap();
        store.0.execute_batch("CREATE TRIGGER fail_advance_outbox BEFORE INSERT ON outbox WHEN NEW.task_id IN (SELECT task_id FROM task_attempts WHERE attempt_id='attempt-1') BEGIN SELECT RAISE(ABORT,'advance failure'); END;").unwrap();
        assert_eq!(advance_after_observe(&mut store,&task.id,&first.attempt_id),Err(Error::StorageUnavailable));
        assert_eq!((store.get(&task.id).unwrap().sequence,store.get_attempt(&task.id).unwrap().unwrap().phase),(observed.sequence,AttemptPhase::Observed));
        store.0.execute_batch("DROP TRIGGER fail_advance_outbox;").unwrap();
        let (advanced,boundary)=advance_after_observe(&mut store,&task.id,&first.attempt_id).unwrap();
        assert_eq!((advanced.status,boundary.stop_sequence,gate.has_occupancy()),(Status::Running,advanced.sequence,Ok(true)));
        assert!(store.events_with_steps(&task.id,0,100).unwrap().iter().any(|event|event.attempt_result.is_some()));
        assert_eq!(advance_after_observe(&mut store,&task.id,&first.attempt_id).unwrap(),(advanced.clone(),boundary));
        let (declared,_)=store.declare_step("a1",&task.id,advanced.sequence,"step-2","第二步").unwrap();
        let (_,second)=prepare_next_attempt(&mut store,&attempt("step-2","attempt-2"),declared.sequence).unwrap();
        let (_observed,_)=record_dispatch_outcome(&mut store,&task.id,&second.attempt_id,DispatchOutcome::Known{action_succeeded:true,observation:None}).unwrap();
        let (advanced,_)=advance_after_observe(&mut store,&task.id,&second.attempt_id).unwrap();
        assert_eq!(store.0.query_row("SELECT count(*) FROM task_attempts WHERE task_id=?1",[&task.id],|r|r.get::<_,i64>(0)).unwrap(),2);
        assert_eq!(store.events(&task.id,0,100).unwrap().len(),advanced.sequence as usize);
        let completed=permit.finish_after_stop(&mut store,advanced.sequence,Outcome::Completed).ok().unwrap();
        assert_eq!((completed.status,gate.has_occupancy()),(Status::Completed,Ok(false)));

        let blocked=store.register("a1","blocked-next","blocked",Some("控制阻断")).unwrap();
        let (blocked,_)=store.declare_step("a1",&blocked.id,blocked.sequence,"step","步骤").unwrap();
        let request=ExecutionAttempt{task_id:blocked.id.clone(),step_id:"step".into(),attempt_id:"attempt".into(),worker_instance_id:"worker".into(),host_session_id:"host".into(),phase:AttemptPhase::Prepared,accepted_sequence:0};
        let gate=Admission::new(1).unwrap();
        let (_,accepted,_permit)=start_attempt(&mut store,&gate,&request,blocked.sequence,&[Resource::Browser]).unwrap();
        let (observed,_)=record_dispatch_outcome(&mut store,&blocked.id,&accepted.attempt_id,DispatchOutcome::Known{action_succeeded:true,observation:None}).unwrap();
        yonder_application::request_control(&mut store,AuthContext::LocalUser("desktop"),&blocked.id,observed.sequence,ControlKind::Pause).unwrap();
        assert_eq!(advance_after_observe(&mut store,&blocked.id,&accepted.attempt_id),Err(Error::StopRequired));
    }

    #[test]
    fn agent_waits_only_at_observed_boundary_and_persists_reason() {
        use yonder_application::{AuthContext,AttemptPhase,ExecutionAttempt,advance_after_observe,wait_for_user};
        use yonder_application::admission::{Admission,Resource,start_attempt};
        use yonder_application::computer_use::{DispatchOutcome,record_dispatch_outcome};
        use yonder_application::gateway::{GatewaySession,Platform};
        use yonder_protocol::{QueryResult,Response,TaskStatus};
        let mut store=SqliteTaskStore::initialize(Connection::open_in_memory().unwrap(),true).unwrap();
        let task=store.register("a1","wait-user","wait",Some("等待用户")).unwrap();
        let (task,_)=store.declare_step("a1",&task.id,task.sequence,"step","准备询问").unwrap();
        let attempt=ExecutionAttempt{task_id:task.id.clone(),step_id:"step".into(),attempt_id:"attempt".into(),worker_instance_id:"worker".into(),host_session_id:"host".into(),phase:AttemptPhase::Prepared,accepted_sequence:0};
        let gate=Admission::new(1).unwrap();
        let (_,attempt,permit)=start_attempt(&mut store,&gate,&attempt,task.sequence,&[Resource::Desktop]).unwrap();drop(permit);
        assert_eq!(wait_for_user(&mut store,AuthContext::Agent("a1"),&task.id,attempt.accepted_sequence,"请确认"),Err(Error::StopRequired));
        assert_eq!(gate.has_occupancy(),Ok(true));
        let (observed,_)=record_dispatch_outcome(&mut store,&task.id,&attempt.attempt_id,DispatchOutcome::Known{action_succeeded:true,observation:None}).unwrap();
        let (boundary,_)=advance_after_observe(&mut store,&task.id,&attempt.attempt_id).unwrap();
        assert!(boundary.sequence>observed.sequence);
        let request=format!(r#"{{"jsonrpc":"2.0","id":"w","method":"task.wait_for_user","params":{{"agent_id":"a1","capability":"task.wait-for-user","deadline":2000,"task_id":"{}","expected_sequence":"{}","reason":"请确认发送内容"}}}}"#,task.id,boundary.sequence);
        let mut old=GatewaySession::new(AuthContext::Agent("a1"),Platform::Macos);
        old.handle(&mut store,br#"{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":16}}}"#,1000);
        let (bytes,_)=old.handle_encoded_with_runtimes(&mut store,&gate,None,None,None,false,"host",request.as_bytes(),1000).unwrap();
        assert!(matches!(yonder_protocol::decode_response(&bytes).unwrap(),Response::Failure{error,..} if error.code==-32010));
        assert_eq!((store.get(&task.id).unwrap().status,gate.has_occupancy()),(Status::Running,Ok(true)));
        let mut current=GatewaySession::new(AuthContext::Agent("a1"),Platform::Macos);
        current.handle(&mut store,br#"{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":17}}}"#,1000);
        let (bytes,_)=current.handle_encoded_with_runtimes(&mut store,&gate,None,None,None,false,"host",request.as_bytes(),1000).unwrap();
        assert!(matches!(yonder_protocol::decode_response(&bytes).unwrap(),Response::Success{result:QueryResult::Snapshot{task},..} if task.status==TaskStatus::WaitingForUser));
        assert_eq!(gate.has_occupancy(),Ok(false));
        let event=store.events_with_steps(&task.id,boundary.sequence,1).unwrap().pop().unwrap();
        assert_eq!((event.transition.next,event.wait_reason.as_deref()),(Status::WaitingForUser,Some("请确认发送内容")));
        let events=format!(r#"{{"jsonrpc":"2.0","id":"e","method":"task.events","params":{{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"{}","after_sequence":"{}","limit":1}}}}"#,task.id,boundary.sequence);
        assert!(matches!(current.handle(&mut store,events.as_bytes(),1000),Response::Success{result:QueryResult::Events{events,..},..} if events[0].wait_reason.as_deref()==Some("请确认发送内容")));
        assert!(matches!(old.handle(&mut store,events.as_bytes(),1000),Response::Success{result:QueryResult::Events{events,..},..} if events[0].wait_reason.is_none()));
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
            match yonder_application::query::handle(&mut second, AuthContext::Agent("a1"), get, 1000) {
                Response::Success { id, result: QueryResult::Snapshot { task }, .. } => {
                    assert_eq!(id, "r1"); assert_eq!(task.status, TaskStatus::Running); assert_eq!(task.sequence, "2");
                }
                response => panic!("查询响应不匹配：{response:?}"),
            }
            let events = br#"{"jsonrpc":"2.0","id":"r2","method":"task.events","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"task-1","after_sequence":"1","limit":1}}"#;
            match yonder_application::query::handle(&mut second, AuthContext::Agent("a1"), events, 1000) {
                Response::Success { id, result: QueryResult::Events { events, .. }, .. } => {
                    assert_eq!(id, "r2"); assert_eq!(events.len(), 1); assert_eq!(events[0].sequence, "2");
                }
                response => panic!("事件响应不匹配：{response:?}"),
            }
            match yonder_application::query::handle(&mut second, AuthContext::Agent("a1"), get, 2000) {
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
        opened.0.execute_batch("PRAGMA user_version=999;").unwrap();
        drop(opened);
        assert!(SqlCipherTaskStore::open(&path, &key).is_err());
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }
}
