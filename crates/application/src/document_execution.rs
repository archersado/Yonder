//! 双文件授权的 OOXML 默认另存；不让路径或 OOXML 进入任务事实。
use crate::{
    AttemptConclusion, AttemptPhase, AuthContext, Error, ExecutionAttempt, Task, TaskStore,
    admission::{Admission, Resource, start_execution},
    document::{self, DocumentFileError, DocumentFileReceipt, DocumentPort, DocumentSaveAsRequest},
    file::FilePort,
    file_authorization::{FileAuthorizationRegistry, FileGrantError, FileGrantLocation, FileGrantPurpose},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentExecution { pub task: Task, pub attempt_result: crate::AttemptResultRecord, pub receipt: Option<DocumentFileReceipt> }

pub fn execute_agent_save_as(
    store: &mut impl TaskStore, admission: &Admission, grants: &FileAuthorizationRegistry,
    files: &dyn FilePort, documents: &dyn DocumentPort, auth: AuthContext<'_>, task_id: &str,
    expected_sequence: u64, source_grant_id: &str, output_grant_id: &str, expected_hash: &str,
    before: &str, after: &str, host_session_id: &str, now_ms: u64,
) -> Result<DocumentExecution, Error> {
    if !crate::valid_id(task_id) || !crate::valid_id(source_grant_id) || !crate::valid_id(output_grant_id)
        || !crate::valid_id(host_session_id) || source_grant_id == output_grant_id
        || expected_hash.len() != 64 || before.is_empty() || before.len() > 16 * 1024 || after.len() > 16 * 1024
    { return Err(Error::InvalidInput); }
    let (task, step) = crate::get_with_step(store, auth, task_id)?; let step = step.ok_or(Error::StopRequired)?;
    if task.sequence != expected_sequence { return Err(Error::Conflict); }
    let attempt = ExecutionAttempt { task_id:task_id.into(), step_id:step.step_id, attempt_id:format!("attempt_{}",expected_sequence+1), worker_instance_id:"document_worker".into(), host_session_id:host_session_id.into(), phase:AttemptPhase::Prepared, accepted_sequence:0 };
    let accepted=start_execution(store,admission,&task,&attempt,expected_sequence,&[Resource::File(format!("document_{source_grant_id}_{output_grant_id}"))]).map_err(|_|Error::StopRequired)?.1;
    let source=match grants.resolve(auth,&task,source_grant_id,FileGrantPurpose::Read,now_ms) {
        Ok(grant)=>grant,
        Err(error)=>return record_grant_failure(store,&accepted,task_id,error),
    };
    let output=match grants.resolve(auth,&task,output_grant_id,FileGrantPurpose::CreateNew,now_ms) {
        Ok(grant)=>grant,
        Err(error)=>return record_grant_failure(store,&accepted,task_id,error),
    };
    let operation=match (source.location,output.location) {
        (FileGrantLocation::Existing { request:source,.. }, FileGrantLocation::CreateTarget { request:output,.. }) =>
            document::save_as(files,documents,&DocumentSaveAsRequest { source, output_path:output.path, output_authorized_root:output.authorized_root, expected_sha256:expected_hash.into(), before:before.into(), after:after.into() }),
        _ => Err(DocumentFileError::PermissionDenied),
    };
    let current=store.get(task_id)?;
    let (task, attempt_result, receipt)=match operation {
        Ok(receipt)=>{ let (task,result)=store.record_attempt_result(&accepted,current.sequence,AttemptConclusion::Observed{action_succeeded:true}).map_err(|_|Error::StorageUnavailable)?; (task,result,Some(receipt)) }
        Err(DocumentFileError::Unknown|DocumentFileError::File(crate::file::FileError::Unknown))=>{ let (task,result)=store.record_attempt_result(&accepted,current.sequence,AttemptConclusion::Unknown{reason:crate::computer_use::UnknownReason::WorkerFailed}).map_err(|_|Error::StorageUnavailable)?; (task,result,None) }
        Err(_)=>{ let (task,result)=store.record_attempt_result(&accepted,current.sequence,AttemptConclusion::Observed{action_succeeded:false}).map_err(|_|Error::StorageUnavailable)?; (task,result,None) }
    };
    Ok(DocumentExecution { task, attempt_result, receipt })
}

pub fn execute_agent_save_as_runtime(
    store: &mut impl TaskStore,
    runtime: &crate::execution_runtime::ExecutionRuntimeHandle,
    admission: &Admission,
    grants: &FileAuthorizationRegistry,
    files: &dyn FilePort,
    documents: &dyn DocumentPort,
    auth: AuthContext<'_>,
    task_id: &str,
    expected_sequence: u64,
    source_grant_id: &str,
    output_grant_id: &str,
    expected_hash: &str,
    before: &str,
    after: &str,
    host_session_id: &str,
    now_ms: u64,
) -> Result<DocumentExecution, Error> {
    if !crate::valid_id(task_id)
        || !crate::valid_id(source_grant_id)
        || !crate::valid_id(output_grant_id)
        || !crate::valid_id(host_session_id)
        || source_grant_id == output_grant_id
        || expected_hash.len() != 64
        || before.is_empty()
        || before.len() > 16 * 1024
        || after.len() > 16 * 1024
    {
        return Err(Error::InvalidInput);
    }
    let (task, step) = crate::get_with_step(store, auth, task_id)?;
    let step = step.ok_or(Error::StopRequired)?;
    let attempt = crate::execution_runtime::begin_attempt(
        runtime,
        admission,
        &task,
        &step,
        expected_sequence,
        "document_worker",
        host_session_id,
        &[Resource::File(format!(
            "document_{source_grant_id}_{output_grant_id}"
        ))],
    )
    .map_err(runtime_error)?;
    let source = match grants.resolve(auth, &task, source_grant_id, FileGrantPurpose::Read, now_ms) {
        Ok(grant) => grant,
        Err(error) => return record_runtime_grant_failure(runtime, &task, &attempt, error),
    };
    let output = match grants.resolve(auth, &task, output_grant_id, FileGrantPurpose::CreateNew, now_ms) {
        Ok(grant) => grant,
        Err(error) => return record_runtime_grant_failure(runtime, &task, &attempt, error),
    };
    let operation = match (source.location, output.location) {
        (
            FileGrantLocation::Existing { request: source, .. },
            FileGrantLocation::CreateTarget { request: output, .. },
        ) => document::save_as(
            files,
            documents,
            &DocumentSaveAsRequest {
                source,
                output_path: output.path,
                output_authorized_root: output.authorized_root,
                expected_sha256: expected_hash.into(),
                before: before.into(),
                after: after.into(),
            },
        ),
        _ => Err(DocumentFileError::PermissionDenied),
    };
    let (conclusion, receipt) = match operation {
        Ok(receipt) => (
            AttemptConclusion::Observed {
                action_succeeded: true,
            },
            Some(receipt),
        ),
        Err(DocumentFileError::Unknown | DocumentFileError::File(crate::file::FileError::Unknown)) => (
            AttemptConclusion::Unknown {
                reason: crate::computer_use::UnknownReason::WorkerFailed,
            },
            None,
        ),
        Err(_) => (
            AttemptConclusion::Observed {
                action_succeeded: false,
            },
            None,
        ),
    };
    let (snapshot, attempt_result) =
        crate::execution_runtime::record_attempt_outcome(runtime, &attempt, conclusion)
            .map_err(runtime_error)?;
    Ok(DocumentExecution {
        task: Task {
            status: crate::Status::Running,
            sequence: snapshot.sequence,
            ..task
        },
        attempt_result,
        receipt,
    })
}

fn record_runtime_grant_failure(
    runtime: &crate::execution_runtime::ExecutionRuntimeHandle,
    task: &Task,
    attempt: &ExecutionAttempt,
    error: FileGrantError,
) -> Result<DocumentExecution, Error> {
    let conclusion = if matches!(error, FileGrantError::Unavailable) {
        AttemptConclusion::Unknown {
            reason: crate::computer_use::UnknownReason::WorkerFailed,
        }
    } else {
        AttemptConclusion::Observed {
            action_succeeded: false,
        }
    };
    let (snapshot, attempt_result) =
        crate::execution_runtime::record_attempt_outcome(runtime, attempt, conclusion)
            .map_err(runtime_error)?;
    Ok(DocumentExecution {
        task: Task {
            status: crate::Status::Running,
            sequence: snapshot.sequence,
            ..task.clone()
        },
        attempt_result,
        receipt: None,
    })
}

fn runtime_error(error: crate::execution_runtime::RuntimeError) -> Error {
    match error {
        crate::execution_runtime::RuntimeError::InvalidInput => Error::InvalidInput,
        crate::execution_runtime::RuntimeError::Conflict => Error::Conflict,
        crate::execution_runtime::RuntimeError::Backpressure => Error::StopRequired,
        crate::execution_runtime::RuntimeError::NotFound => Error::NotFound,
        crate::execution_runtime::RuntimeError::Unavailable => Error::StorageUnavailable,
    }
}

/// 授权在启动事务之后才可消费；解析失败也必须收束 attempt，不能留下伪运行状态。
fn record_grant_failure(
    store: &mut impl TaskStore,
    accepted: &ExecutionAttempt,
    task_id: &str,
    error: FileGrantError,
) -> Result<DocumentExecution, Error> {
    let current=store.get(task_id)?;
    let conclusion = if matches!(error, FileGrantError::Unavailable) {
        AttemptConclusion::Unknown{reason:crate::computer_use::UnknownReason::WorkerFailed}
    } else {
        AttemptConclusion::Observed{action_succeeded:false}
    };
    let (task,attempt_result)=store.record_attempt_result(
        accepted,current.sequence,conclusion
    ).map_err(|_|Error::StorageUnavailable)?;
    Ok(DocumentExecution{task,attempt_result,receipt:None})
}
