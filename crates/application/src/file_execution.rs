//! 受任务授权的有界文件执行用例；不把位置或正文写入任务事实。
use crate::{
    AttemptConclusion, AttemptResultRecord, AttemptPhase, AuthContext, Error, ExecutionAttempt,
    Task, TaskStore,
    admission::{Admission, Resource, start_execution},
    file::{self, AcceptAnyFile, FileError, FilePort, FileTrashRequest, FileWriteMode, FileWriteRequest},
    file_authorization::{FileAuthorizationRegistry, FileGrantError, FileGrantLocation, FileGrantPurpose},
};

pub const MAX_AGENT_FILE_BYTES: usize = 48 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileOperation { Read, CreateNew, Replace, Trash }

impl FileOperation {
    pub fn purpose(self) -> FileGrantPurpose {
        match self {
            Self::Read => FileGrantPurpose::Read,
            Self::CreateNew => FileGrantPurpose::CreateNew,
            Self::Replace => FileGrantPurpose::Replace,
            Self::Trash => FileGrantPurpose::Trash,
        }
    }
    fn needs_body(self) -> bool { matches!(self, Self::CreateNew | Self::Replace) }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileExecution {
    pub task: Task,
    pub attempt_result: AttemptResultRecord,
    /// 只在 read 成功时返回；Gateway 负责转为有界 Base64，调用方不得持久化。
    pub read_bytes: Option<Vec<u8>>,
    pub sha256: Option<String>,
    pub bytes_written: Option<u64>,
}

/// 首次副作用复用 TM-S7；操作完成后记录 observed/unknown，不自动推进或完成任务。
pub fn execute_agent_file(
    store: &mut impl TaskStore,
    admission: &Admission,
    registry: &FileAuthorizationRegistry,
    files: &dyn FilePort,
    auth: AuthContext<'_>,
    task_id: &str,
    expected_sequence: u64,
    grant_id: &str,
    operation: FileOperation,
    body: Option<&[u8]>,
    host_session_id: &str,
    now_ms: u64,
) -> Result<FileExecution, Error> {
    if !crate::valid_id(task_id)
        || !crate::valid_id(grant_id)
        || !crate::valid_id(host_session_id)
        || body.is_some_and(|value| value.len() > MAX_AGENT_FILE_BYTES)
        || (operation.needs_body() != body.is_some())
    { return Err(Error::InvalidInput); }
    let (task, step) = crate::get_with_step(store, auth, task_id)?;
    let step = step.ok_or(Error::StopRequired)?;
    if task.sequence != expected_sequence { return Err(Error::Conflict); }
    // 先以可信任务事实启动；资源精确身份仍由 File Port 的锁与提交复核守护。
    let attempt = ExecutionAttempt {
        task_id: task_id.into(), step_id: step.step_id, attempt_id: format!("attempt_{}", expected_sequence + 1),
        worker_instance_id: "file_worker".into(), host_session_id: host_session_id.into(),
        phase: AttemptPhase::Prepared, accepted_sequence: 0,
    };
    let accepted = start_execution(store, admission, &task, &attempt, expected_sequence, &[Resource::File(grant_id.into())])
        .map_err(|_| Error::StopRequired)?.1;
    let grant = registry.resolve(auth, &task, grant_id, operation.purpose(), now_ms)
        .map_err(file_grant_error)?;
    let outcome = perform(files, grant.location, operation, body);
    let result_task = store.get(task_id)?;
    let result = match outcome {
        Ok(value) => store.record_attempt_result(&accepted, result_task.sequence, AttemptConclusion::Observed { action_succeeded: true })
            .map(|(task, record)| (task, record, value)),
        Err(FileError::Unknown) => store.record_attempt_result(&accepted, result_task.sequence, AttemptConclusion::Unknown { reason: crate::computer_use::UnknownReason::WorkerFailed })
            .map(|(task, record)| (task, record, Performed::empty())),
        Err(_) => store.record_attempt_result(&accepted, result_task.sequence, AttemptConclusion::Observed { action_succeeded: false })
            .map(|(task, record)| (task, record, Performed::empty())),
    }.map_err(|_| Error::StorageUnavailable)?;
    Ok(FileExecution { task: result.0, attempt_result: result.1, read_bytes: result.2.read_bytes, sha256: result.2.sha256, bytes_written: result.2.bytes_written })
}

struct Performed { read_bytes: Option<Vec<u8>>, sha256: Option<String>, bytes_written: Option<u64> }
impl Performed { fn empty() -> Self { Self { read_bytes: None, sha256: None, bytes_written: None } } }

fn perform(files: &dyn FilePort, location: FileGrantLocation, operation: FileOperation, body: Option<&[u8]>) -> Result<Performed, FileError> {
    match (location, operation) {
        (FileGrantLocation::Existing { request, .. }, FileOperation::Read) => {
            let snapshot = file::read(files, &request)?;
            if snapshot.bytes.len() > MAX_AGENT_FILE_BYTES { return Err(FileError::TooLarge); }
            Ok(Performed { read_bytes: Some(snapshot.bytes), sha256: Some(snapshot.sha256), bytes_written: None })
        }
        (FileGrantLocation::CreateTarget { request, parent_identity, .. }, FileOperation::CreateNew) => {
            let receipt = file::write_atomic(files, &FileWriteRequest { path: request.path, authorized_root: request.authorized_root, bytes: body.expect("validated").to_vec(), mode: FileWriteMode::CreateNew { expected_parent_identity: parent_identity } }, &AcceptAnyFile)?;
            Ok(Performed { read_bytes: None, sha256: Some(receipt.sha256), bytes_written: Some(receipt.bytes_written) })
        }
        (FileGrantLocation::Existing { request, identity, sha256, .. }, FileOperation::Replace) => {
            let receipt = file::write_atomic(files, &FileWriteRequest { path: request.path, authorized_root: request.authorized_root, bytes: body.expect("validated").to_vec(), mode: FileWriteMode::Replace { expected_identity: identity, expected_sha256: sha256 } }, &AcceptAnyFile)?;
            Ok(Performed { read_bytes: None, sha256: Some(receipt.sha256), bytes_written: Some(receipt.bytes_written) })
        }
        (FileGrantLocation::Existing { request, identity, .. }, FileOperation::Trash) => {
            file::trash_granted(files, &FileTrashRequest { path: request.path, authorized_root: request.authorized_root, expected_identity: identity })?;
            Ok(Performed::empty())
        }
        _ => Err(FileError::InvalidInput),
    }
}

fn file_grant_error(error: FileGrantError) -> Error {
    match error { FileGrantError::InvalidInput => Error::InvalidInput, FileGrantError::Unavailable => Error::StorageUnavailable, _ => Error::StopRequired }
}
