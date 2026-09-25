//! 进程内只读分派；调用方须先绑定认证身份，本模块不对外开放传输。
use crate::{AuthContext, Error, Status, Task, TaskStore, events, get};
use yonder_protocol::{
    AttemptResult as ProtocolAttemptResult, AttemptResultPhase as ProtocolAttemptPhase,
    AttemptUnknownReason as ProtocolUnknownReason, BrowserReference,
    ControlKind as ProtocolControlKind, ControlPhase as ProtocolControlPhase,
    ControlRecord as ProtocolControl, FocusFailure as ProtocolFocusFailure,
    FocusPhase as ProtocolFocusPhase, QueryResult, Request, Response, RpcError,
    StepDeclaration as ProtocolStep, TaskArtifactManifest as ProtocolManifest, TaskEvent,
    TaskObservation as ProtocolObservation, TaskObservationResult as ProtocolObservationResult,
    TaskSnapshot, TaskSource as ProtocolSource, TaskStatus,
    TaskUserConfirmation as ProtocolConfirmation, Version,
};

fn bounded_events_result(
    request_id: &str,
    task_id: String,
    events: Vec<TaskEvent>,
) -> Result<QueryResult, RpcError> {
    let empty = Response::Success {
        jsonrpc: Version::V2,
        id: request_id.to_owned(),
        result: QueryResult::Events {
            task_id: task_id.clone(),
            events: Vec::new(),
        },
    };
    let mut response_bytes = yonder_protocol::encode(&empty)
        .map_err(|_| RpcError::new(-32603, "事件响应编码失败"))?
        .len();
    if response_bytes > yonder_protocol::MAX_TASK_EVENTS_RESPONSE_BYTES {
        return Err(RpcError::new(-32015, "事件响应超过编码预算"));
    }
    let mut selected = Vec::new();
    for event in events {
        let event_bytes = yonder_protocol::encoded_task_event_len(&event)
            .map_err(|_| RpcError::new(-32603, "事件响应编码失败"))?;
        if event_bytes > yonder_protocol::MAX_TASK_EVENT_BYTES {
            return Err(RpcError::new(-32015, "历史事件超过单项编码预算"));
        }
        let next_bytes = response_bytes + event_bytes + usize::from(!selected.is_empty());
        if next_bytes > yonder_protocol::MAX_TASK_EVENTS_RESPONSE_BYTES {
            if selected.is_empty() {
                return Err(RpcError::new(-32015, "首条历史事件无法放入响应预算"));
            }
            break;
        }
        response_bytes = next_bytes;
        selected.push(event);
    }
    Ok(QueryResult::Events {
        task_id,
        events: selected,
    })
}

fn check_event_continuity(
    after: u64,
    task_sequence: u64,
    limit: usize,
    records: &[crate::TaskEventRecord],
) -> Result<(), RpcError> {
    let incomplete = || RpcError::new(-32016, "任务历史不完整，请刷新后重试");
    let mut last = after;
    for record in records {
        let expected = last.checked_add(1).ok_or_else(incomplete)?;
        if record.transition.sequence != expected {
            return Err(incomplete());
        }
        last = expected;
    }
    if records.len() < limit && last < task_sequence {
        return Err(incomplete());
    }
    Ok(())
}

fn observation(value: crate::TaskObservation) -> ProtocolObservation {
    ProtocolObservation {
        step_id: value.step_id,
        result: match value.result {
            crate::TaskObservationResult::Matched => ProtocolObservationResult::Matched,
            crate::TaskObservationResult::NotMatched => ProtocolObservationResult::NotMatched,
            crate::TaskObservationResult::Unknown => ProtocolObservationResult::Unknown,
        },
        summary: value.summary,
    }
}

pub(crate) fn status(value: Status) -> TaskStatus {
    match value {
        Status::Created => TaskStatus::Created,
        Status::Running => TaskStatus::Running,
        Status::WaitingForUser => TaskStatus::WaitingForUser,
        Status::Paused => TaskStatus::Paused,
        Status::Interrupted => TaskStatus::Interrupted,
        Status::Completed => TaskStatus::Completed,
        Status::Failed => TaskStatus::Failed,
        Status::Cancelled => TaskStatus::Cancelled,
    }
}

fn source(value: crate::TaskSource) -> ProtocolSource {
    match value {
        crate::TaskSource::LocalAgent => ProtocolSource::LocalAgent,
        crate::TaskSource::CloudAgent => ProtocolSource::CloudAgent,
        crate::TaskSource::Legacy => ProtocolSource::Legacy,
    }
}

pub(crate) fn summary(task: crate::Task) -> TaskSnapshot {
    TaskSnapshot {
        task_id: task.id,
        owner_agent_id: task.owner_agent_id,
        name: task.name,
        source: Some(source(task.source)),
        status: status(task.status),
        sequence: task.sequence.to_string(),
        current_step: None,
        observation: None,
        next_intent: None,
        artifact_manifest: None,
        user_confirmation: None,
    }
}

pub(crate) fn full_with_audit(
    task: crate::Task,
    presentation: crate::TaskPresentation,
    audit: crate::TaskAudit,
) -> TaskSnapshot {
    TaskSnapshot {
        task_id: task.id,
        owner_agent_id: task.owner_agent_id,
        name: task.name,
        source: Some(source(task.source)),
        status: status(task.status),
        sequence: task.sequence.to_string(),
        current_step: presentation.current_step.map(|step| ProtocolStep {
            step_id: step.step_id,
            label: step.label,
            accepted_sequence: step.accepted_sequence.to_string(),
        }),
        observation: presentation.observation.map(observation),
        next_intent: presentation.next_intent,
        artifact_manifest: audit.manifest.map(|manifest| ProtocolManifest {
            task_id: manifest.task_id,
            version: manifest.version.to_string(),
            item_count: manifest.item_count,
        }),
        user_confirmation: audit.confirmation.map(|confirmation| ProtocolConfirmation {
            task_id: confirmation.task_id,
            confirmation_id: confirmation.confirmation_id,
            result_sequence: confirmation.result_sequence.to_string(),
            manifest_version: confirmation.manifest_version.to_string(),
            comment: confirmation.comment,
            confirmed_by: confirmation.confirmed_by,
        }),
    }
}

pub(crate) fn error(value: Error) -> RpcError {
    match value {
        Error::Conflict => RpcError::new(-32011, "任务状态已更新，请刷新"),
        Error::StopRequired => {
            RpcError::new(-32012, "当前步骤结果待核实或尚未到达安全边界，无法取消")
        }
        Error::IdempotencyConflict => RpcError::new(-32009, "幂等键对应不同任务名称或说明"),
        Error::StepConflict => RpcError::new(-32013, "步骤标识对应不同标签"),
        Error::QuotaExceeded => RpcError::new(-32014, "审计容量不足"),
        Error::PermissionDenied => RpcError::new(-32003, "此身份不允许执行该操作"),
        Error::NotFound => RpcError::new(-32004, "任务不存在"),
        Error::InvalidInput => RpcError::new(-32602, "非法请求参数"),
        _ => RpcError::new(-32603, "任务存储不可用"),
    }
}

fn readable(store: &mut impl TaskStore, auth: AuthContext<'_>, id: &str) -> Result<Task, RpcError> {
    let task = get(store, id).map_err(error)?;
    if !auth.can_read(&task) {
        return Err(error(Error::NotFound));
    }
    Ok(task)
}

/// 组合根复用协议编码，不需引用具体协议库或创建第二份响应模型。
pub fn handle_encoded(
    store: &mut impl TaskStore,
    auth: AuthContext<'_>,
    bytes: &[u8],
    now_ms: u64,
) -> Result<Vec<u8>, Error> {
    yonder_protocol::encode(&handle(store, auth, bytes, now_ms))
        .map_err(|_| Error::StorageUnavailable)
}

/// 可信同版本组合根读取当前事件形状；外部 Agent 仍由 Gateway 协商版本。
pub fn handle_encoded_current(
    store: &mut impl TaskStore,
    auth: AuthContext<'_>,
    bytes: &[u8],
    now_ms: u64,
) -> Result<Vec<u8>, Error> {
    let response = match yonder_protocol::decode(bytes) {
        Ok(request)
            if matches!(request, Request::Cancel { .. })
                && matches!(auth, AuthContext::Agent(_)) =>
        {
            Response::Failure {
                jsonrpc: Version::V2,
                id: Some(request.request_id().into()),
                error: RpcError::new(-32002, "取消须通过Gateway会话"),
            }
        }
        Ok(request) => handle_request_versioned(
            store,
            auth,
            request,
            now_ms,
            true,
            true,
            true,
            store.supports_audit(),
            true,
            true,
            true,
            true,
        ),
        Err(error) => Response::Failure {
            jsonrpc: Version::V2,
            id: None,
            error,
        },
    };
    yonder_protocol::encode(&response).map_err(|_| Error::StorageUnavailable)
}

pub fn handle(
    store: &mut impl TaskStore,
    auth: AuthContext<'_>,
    bytes: &[u8],
    now_ms: u64,
) -> Response {
    let request = match yonder_protocol::decode(bytes) {
        Ok(request) => request,
        Err(error) => {
            return Response::Failure {
                jsonrpc: Version::V2,
                id: None,
                error,
            };
        }
    };
    if matches!(request, Request::Cancel { .. }) && matches!(auth, AuthContext::Agent(_)) {
        return Response::Failure {
            jsonrpc: Version::V2,
            id: Some(request.request_id().into()),
            error: RpcError::new(-32002, "取消须通过Gateway会话"),
        };
    }
    handle_request(store, auth, request, now_ms)
}

pub(crate) fn validate(
    request: &Request,
    auth: AuthContext<'_>,
    now_ms: u64,
) -> Result<(), RpcError> {
    request.validate(now_ms)?;
    if !yonder_protocol::valid_id(auth.agent_id()) || request.agent_id() != auth.agent_id() {
        return Err(RpcError::new(-32003, "请求身份不匹配"));
    }
    Ok(())
}

pub(crate) fn handle_request(
    store: &mut impl TaskStore,
    auth: AuthContext<'_>,
    request: Request,
    now_ms: u64,
) -> Response {
    handle_request_versioned(
        store, auth, request, now_ms, false, false, false, false, false, false, false, false,
    )
}

pub(crate) fn handle_request_versioned(
    store: &mut impl TaskStore,
    auth: AuthContext<'_>,
    request: Request,
    now_ms: u64,
    include_steps: bool,
    include_attempt_results: bool,
    include_wait_reason: bool,
    include_audit: bool,
    include_observation_history: bool,
    include_control_history: bool,
    include_focus_history: bool,
    include_creation_history: bool,
) -> Response {
    let id = request.request_id().to_owned();
    let result = validate(&request, auth, now_ms).and_then(|()| {
        match request {
        Request::Cancel { params, .. } => {
            let task = crate::cancel_pending(store, auth, &params.task_id, yonder_protocol::sequence(&params.expected_sequence)?).map_err(error)?;
            Ok(QueryResult::Snapshot { task: summary(task) })
        },
        Request::Control { params, .. } => {
            let kind = match params.kind { ProtocolControlKind::Pause=>crate::ControlKind::Pause,ProtocolControlKind::Cancel=>crate::ControlKind::Cancel,ProtocolControlKind::Takeover=>crate::ControlKind::Takeover };
            let (task,control) = crate::request_control(store,auth,&params.task_id,yonder_protocol::sequence(&params.expected_sequence)?,kind).map_err(error)?;
            let control = ProtocolControl { attempt_id:control.attempt_id,control_id:control.control_id,kind:params.kind,phase:match control.phase { crate::ControlPhase::Pending=>ProtocolControlPhase::Pending,crate::ControlPhase::Stopped=>ProtocolControlPhase::Stopped },accepted_sequence:control.accepted_sequence.to_string(),stopped_sequence:control.stopped_sequence.map(|value| value.to_string()),focus_phase:control.focus_phase.map(|value|match value{crate::FocusPhase::Locating=>ProtocolFocusPhase::Locating,crate::FocusPhase::Focused=>ProtocolFocusPhase::Focused,crate::FocusPhase::Failed=>ProtocolFocusPhase::Failed}),focus_failure:control.focus_failure.map(|value|match value{crate::work_focus::FocusFailure::PermissionUnavailable=>ProtocolFocusFailure::PermissionUnavailable,crate::work_focus::FocusFailure::ProcessChanged=>ProtocolFocusFailure::ProcessChanged,crate::work_focus::FocusFailure::WindowMissing=>ProtocolFocusFailure::WindowMissing,crate::work_focus::FocusFailure::MappingNotUnique=>ProtocolFocusFailure::MappingNotUnique,crate::work_focus::FocusFailure::ActivationFailed=>ProtocolFocusFailure::ActivationFailed,crate::work_focus::FocusFailure::VerificationFailed=>ProtocolFocusFailure::VerificationFailed,crate::work_focus::FocusFailure::GeometryChanged=>ProtocolFocusFailure::GeometryChanged,crate::work_focus::FocusFailure::ReferenceUnavailable=>ProtocolFocusFailure::ReferenceUnavailable}) };
            Ok(QueryResult::Control { task: summary(task), control })
        },
        Request::Create { .. } => Err(RpcError::new(-32002, "创建须通过Gateway会话")),
        Request::StepDeclare { .. } => Err(RpcError::new(-32002, "步骤声明须通过Gateway会话")),
        Request::StepAdvance { .. } => Err(RpcError::new(-32002, "步骤推进须通过Gateway会话")),
        Request::BrowserExecute { .. } => Err(RpcError::new(-32002, "浏览器执行须通过Gateway会话")),
        Request::ComputerExecute { .. } => Err(RpcError::new(-32002, "桌面执行须通过Gateway会话")),
        Request::ComputerStep { .. } => Err(RpcError::new(-32002, "桌面步骤须通过Gateway会话")),
        Request::Complete { .. } => Err(RpcError::new(-32002, "任务完成须通过Gateway会话")),
        Request::Fail { .. } => Err(RpcError::new(-32002, "任务失败终结须通过Gateway会话")),
        Request::WaitForUser { .. } => Err(RpcError::new(-32002, "等待用户须通过Gateway会话")),
        Request::Hello { .. } => Err(RpcError::new(-32002, "握手须通过 Gateway 会话")),
        Request::List { params, .. } => {
            let page = if params.running_only {
                crate::list_running(store, auth, params.after_task_id.as_deref(), usize::from(params.limit))
            } else {
                crate::list(store, auth, params.after_task_id.as_deref(), params.include_finished, usize::from(params.limit))
            }.map_err(error)?;
            Ok(QueryResult::Tasks {
                tasks: page.tasks.into_iter().map(summary).collect(),
                next_after_task_id: page.next_after_task_id,
            })
        }
        Request::Get { params, .. } => {
            let (task, presentation) = store.get_presentation(&params.task_id).map_err(error)?;
            if !auth.can_read(&task) { return Err(error(Error::NotFound)); }
            let audit = if include_audit {
                store.get_audit(&params.task_id).map_err(error)?
            } else {
                crate::TaskAudit::default()
            };
            Ok(QueryResult::Snapshot { task: full_with_audit(task, presentation, audit) })
        }
        Request::Events { params, .. } => {
            let task = readable(store, auth, &params.task_id)?;
            let after = yonder_protocol::sequence(&params.after_sequence)?;
            let records = if include_steps { store.events_with_steps(&params.task_id, after, usize::from(params.limit)).map_err(error)? } else {
                events(store, &params.task_id, after, usize::from(params.limit)).map_err(error)?.into_iter().map(|transition| crate::TaskEventRecord { transition, creation_event: None, step_declaration: None, attempt_result: None, observation: None, control_event: None, focus_event: None, wait_reason: None, artifact_manifest: None, user_confirmation: None }).collect()
            };
            check_event_continuity(after, task.sequence, usize::from(params.limit), &records)?;
            let projected = records.into_iter().map(|e| {
                let attempt_result = if include_attempt_results { e.attempt_result.map(|result| {
                    let (phase,action_succeeded,observe_valid,unknown_reason) = match result.conclusion {
                        crate::AttemptConclusion::Observed { action_succeeded } => (ProtocolAttemptPhase::Observed,Some(action_succeeded),true,None),
                        crate::AttemptConclusion::Unknown { reason } => (ProtocolAttemptPhase::Unknown,None,false,Some(match reason {
                            crate::computer_use::UnknownReason::InvalidInput => ProtocolUnknownReason::InvalidInput,
                            crate::computer_use::UnknownReason::DependencyUnavailable => ProtocolUnknownReason::DependencyUnavailable,
                            crate::computer_use::UnknownReason::WorkerFailed => ProtocolUnknownReason::WorkerFailed,
                            crate::computer_use::UnknownReason::TimedOut => ProtocolUnknownReason::TimedOut,
                            crate::computer_use::UnknownReason::InvalidResponse => ProtocolUnknownReason::InvalidResponse,
                            crate::computer_use::UnknownReason::IdentityMismatch => ProtocolUnknownReason::IdentityMismatch,
                            crate::computer_use::UnknownReason::ObserveFailed => ProtocolUnknownReason::ObserveFailed,
                            crate::computer_use::UnknownReason::UserInput => ProtocolUnknownReason::UserInput,
                        })),
                    };
                    ProtocolAttemptResult { step_id:result.step_id, attempt_id:result.attempt_id, worker_instance_id:result.worker_instance_id, host_session_id:result.host_session_id, phase, action_succeeded, observe_valid, unknown_reason }
                }) } else { None };
                let control_event = include_control_history.then(|| e.control_event).flatten().map(|control| yonder_protocol::ControlEvent {
                    attempt_id: control.attempt_id,
                    control_id: control.control_id,
                    kind: match control.kind { crate::ControlKind::Pause => ProtocolControlKind::Pause, crate::ControlKind::Cancel => ProtocolControlKind::Cancel, crate::ControlKind::Takeover => ProtocolControlKind::Takeover },
                    phase: match control.phase { crate::ControlPhase::Pending => ProtocolControlPhase::Pending, crate::ControlPhase::Stopped => ProtocolControlPhase::Stopped },
                });
                let focus_event = include_focus_history.then(|| e.focus_event).flatten().map(|focus| yonder_protocol::FocusEvent {
                    control_id: focus.control_id,
                    phase: match focus.phase { crate::FocusPhase::Locating => ProtocolFocusPhase::Locating, crate::FocusPhase::Focused => ProtocolFocusPhase::Focused, crate::FocusPhase::Failed => ProtocolFocusPhase::Failed },
                    failure: focus.failure.map(|value| match value { crate::work_focus::FocusFailure::PermissionUnavailable=>ProtocolFocusFailure::PermissionUnavailable,crate::work_focus::FocusFailure::ProcessChanged=>ProtocolFocusFailure::ProcessChanged,crate::work_focus::FocusFailure::WindowMissing=>ProtocolFocusFailure::WindowMissing,crate::work_focus::FocusFailure::MappingNotUnique=>ProtocolFocusFailure::MappingNotUnique,crate::work_focus::FocusFailure::ActivationFailed=>ProtocolFocusFailure::ActivationFailed,crate::work_focus::FocusFailure::VerificationFailed=>ProtocolFocusFailure::VerificationFailed,crate::work_focus::FocusFailure::GeometryChanged=>ProtocolFocusFailure::GeometryChanged,crate::work_focus::FocusFailure::ReferenceUnavailable=>ProtocolFocusFailure::ReferenceUnavailable }),
                });
                let creation_event = include_creation_history.then(|| e.creation_event).flatten().map(|creation| yonder_protocol::TaskCreationEvent {
                    owner_agent_id: creation.owner_agent_id,
                    source: source(creation.source),
                });
                TaskEvent {
                    previous: status(e.transition.previous),
                    status: status(e.transition.next),
                    sequence: e.transition.sequence.to_string(),
                    creation_event,
                    step_declaration: e.step_declaration.map(|step| ProtocolStep { step_id: step.step_id, label: step.label, accepted_sequence: step.accepted_sequence.to_string() }),
                    attempt_result,
                    observation: include_observation_history.then(|| e.observation).flatten().map(observation),
                    control_event,
                    focus_event,
                    wait_reason: include_wait_reason.then_some(e.wait_reason).flatten(),
                    artifact_manifest: include_audit.then_some(e.artifact_manifest).flatten().map(|manifest| ProtocolManifest {
                        task_id: manifest.task_id,
                        version: manifest.version.to_string(),
                        item_count: manifest.item_count,
                    }),
                    user_confirmation: include_audit.then_some(e.user_confirmation).flatten().map(|confirmation| ProtocolConfirmation {
                        task_id: confirmation.task_id,
                        confirmation_id: confirmation.confirmation_id,
                        result_sequence: confirmation.result_sequence.to_string(),
                        manifest_version: confirmation.manifest_version.to_string(),
                        comment: confirmation.comment,
                        confirmed_by: confirmation.confirmed_by,
                    }),
                }
            }).collect();
            bounded_events_result(&id, params.task_id, projected)
        },
        Request::StepGet { params, .. } => {
            let (task, presentation) = store.get_presentation(&params.task_id).map_err(error)?;
            if !auth.can_read(&task) { return Err(error(Error::NotFound)); }
            let step = presentation.current_step.clone().map(|step| ProtocolStep { step_id: step.step_id, label: step.label, accepted_sequence: step.accepted_sequence.to_string() });
            let audit = if include_audit {
                store.get_audit(&params.task_id).map_err(error)?
            } else {
                crate::TaskAudit::default()
            };
            Ok(QueryResult::Step { task: full_with_audit(task, presentation, audit), step })
        },
        Request::BrowserGet { params, .. } => {
            let task = readable(store, auth, &params.task_id)?;
            let reference = store.get_browser_reference(&params.task_id).map_err(error)?.map(|value| BrowserReference { external_task_ref:value.external_task_ref, ownership:value.ownership, managed_pages:u16::try_from(value.managed_pages).unwrap_or(u16::MAX), finished:value.finished, updated_sequence:value.updated_sequence.to_string() });
            Ok(QueryResult::BrowserState { task: summary(task), reference })
        }
    }});
    match result {
        Ok(result) => Response::Success {
            jsonrpc: Version::V2,
            id,
            result,
        },
        Err(error) => Response::Failure {
            jsonrpc: Version::V2,
            id: Some(id),
            error,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(sequence: usize, label: String) -> TaskEvent {
        TaskEvent {
            previous: TaskStatus::Running,
            status: TaskStatus::Running,
            sequence: sequence.to_string(),
            creation_event: None,
            step_declaration: Some(ProtocolStep {
                step_id: format!("step-{sequence}"),
                label,
                accepted_sequence: sequence.to_string(),
            }),
            attempt_result: None,
            observation: None,
            control_event: None,
            focus_event: None,
            wait_reason: None,
            artifact_manifest: None,
            user_confirmation: None,
        }
    }

    #[test]
    fn event_continuity_allows_a_concurrent_newer_commit() {
        let record = crate::TaskEventRecord {
            transition: yonder_domain::Transition {
                previous: crate::Status::Running,
                next: crate::Status::Running,
                sequence: 3,
            },
            creation_event: None,
            step_declaration: None,
            attempt_result: None,
            observation: None,
            control_event: None,
            focus_event: None,
            wait_reason: None,
            artifact_manifest: None,
            user_confirmation: None,
        };
        assert!(check_event_continuity(2, 2, 10, &[record]).is_ok());
        assert!(check_event_continuity(2, 2, 10, &[]).is_ok());
    }

    #[test]
    fn events_budget_keeps_a_complete_prefix_and_continuation() {
        let page = bounded_events_result(
            "request-1",
            "task-1".into(),
            (1..=100)
                .map(|sequence| event(sequence, "\"".repeat(3_000)))
                .collect(),
        )
        .unwrap();
        let QueryResult::Events { events, .. } = page else {
            panic!("expected events")
        };
        let last = events.last().unwrap().sequence.parse::<usize>().unwrap();
        assert!(last < 100);
        assert!(last > 1);
        assert_eq!(events.len(), last);
        let encoded = yonder_protocol::encode(&Response::Success {
            jsonrpc: Version::V2,
            id: "request-1".into(),
            result: QueryResult::Events {
                task_id: "task-1".into(),
                events,
            },
        })
        .unwrap();
        assert!(encoded.len() <= yonder_protocol::MAX_TASK_EVENTS_RESPONSE_BYTES);

        let next = bounded_events_result(
            "request-2",
            "task-1".into(),
            ((last + 1)..=100)
                .map(|sequence| event(sequence, "\"".repeat(3_000)))
                .collect(),
        )
        .unwrap();
        let QueryResult::Events { events, .. } = next else {
            panic!("expected events")
        };
        assert_eq!(events[0].sequence, (last + 1).to_string());
    }

    #[test]
    fn events_budget_rejects_escaped_oversize_event_and_unfittable_first_item() {
        let oversized = bounded_events_result(
            "request-1",
            "task-1".into(),
            vec![event(1, "\"".repeat(4_100))],
        );
        assert!(oversized.is_err());

        let single = event(1, "a".into());
        let empty_bytes = yonder_protocol::encode(&Response::Success {
            jsonrpc: Version::V2,
            id: String::new(),
            result: QueryResult::Events {
                task_id: "task-1".into(),
                events: vec![],
            },
        })
        .unwrap()
        .len();
        let event_bytes = yonder_protocol::encoded_task_event_len(&single).unwrap();
        let exact_id =
            "a".repeat(yonder_protocol::MAX_TASK_EVENTS_RESPONSE_BYTES - empty_bytes - event_bytes);
        let exact = bounded_events_result(&exact_id, "task-1".into(), vec![single]);
        let encoded = yonder_protocol::encode(&Response::Success {
            jsonrpc: Version::V2,
            id: exact_id.clone(),
            result: exact.unwrap(),
        })
        .unwrap();
        assert_eq!(
            encoded.len(),
            yonder_protocol::MAX_TASK_EVENTS_RESPONSE_BYTES
        );
        let first_unfittable = bounded_events_result(
            &(exact_id + "a"),
            "task-1".into(),
            vec![event(1, "a".into())],
        );
        assert!(first_unfittable.is_err());
        assert!(
            matches!(bounded_events_result("request-1", "task-1".into(), vec![]), Ok(QueryResult::Events { events, .. }) if events.is_empty())
        );
    }
}
