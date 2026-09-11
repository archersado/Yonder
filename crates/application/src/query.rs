//! 进程内只读分派；调用方须先绑定认证身份，本模块不对外开放传输。
use crate::{Error, Status, TaskStore, events, get};
use yonder_protocol::{QueryResult, Request, Response, RpcError, TaskEvent, TaskSnapshot, TaskStatus, Version};

fn status(value: Status) -> TaskStatus {
    match value {
        Status::Created => TaskStatus::Created, Status::Running => TaskStatus::Running,
        Status::WaitingForUser => TaskStatus::WaitingForUser, Status::Paused => TaskStatus::Paused,
        Status::Interrupted => TaskStatus::Interrupted, Status::Completed => TaskStatus::Completed,
        Status::Failed => TaskStatus::Failed, Status::Cancelled => TaskStatus::Cancelled,
    }
}

fn error(value: Error) -> RpcError {
    match value {
        Error::NotFound => RpcError::new(-32004, "任务不存在"),
        Error::InvalidInput => RpcError::new(-32602, "非法请求参数"),
        _ => RpcError::new(-32603, "任务存储不可用"),
    }
}

pub fn handle(store: &mut impl TaskStore, bytes: &[u8], now_ms: u64) -> Response {
    let request = match yonder_protocol::decode(bytes) {
        Ok(request) => request,
        Err(error) => return Response::Failure { jsonrpc: Version::V2, id: None, error },
    };
    let id = request.request_id().to_owned();
    let result = request.validate(now_ms).and_then(|()| match request {
        Request::Get { params, .. } => {
            let task = get(store, &params.task_id).map_err(error)?;
            Ok(QueryResult::Snapshot { task: TaskSnapshot { task_id: task.id, status: status(task.status), sequence: task.sequence.to_string() } })
        }
        Request::Events { params, .. } => {
            let after = yonder_protocol::sequence(&params.after_sequence)?;
            let events = events(store, &params.task_id, after, usize::from(params.limit)).map_err(error)?;
            Ok(QueryResult::Events { task_id: params.task_id, events: events.into_iter().map(|e| TaskEvent { previous: status(e.previous), status: status(e.next), sequence: e.sequence.to_string() }).collect() })
        }
    });
    match result {
        Ok(result) => Response::Success { jsonrpc: Version::V2, id, result },
        Err(error) => Response::Failure { jsonrpc: Version::V2, id: Some(id), error },
    }
}
