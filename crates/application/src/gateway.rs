//! 每连接握手门禁；身份只能由已认证的宿主传入。
use crate::{
    AuthContext, TaskStore,
    admission::Admission,
    browser_use::{BrowserReferenceRecord, BrowserUsePort},
    computer_use::{ComputerUsePort, WorkTargetPort},
    query,
};
use yonder_protocol::{
    AttemptResult as ProtocolAttemptResult, AttemptResultPhase, AttemptUnknownReason, Availability,
    BrowserOperation, BrowserReference, Capability, CapabilityInfo,
    ComputerObservation as ProtocolComputerObservation, ProtocolVersion, QueryResult, Request,
    Response, RpcError, TaskSnapshot, Version,
};

pub use yonder_protocol::Platform;

const PROTOCOL: ProtocolVersion = ProtocolVersion { major: 1, minor: 0 };

pub fn is_execution_request(bytes: &[u8]) -> bool {
    matches!(
        yonder_protocol::decode(bytes),
        Ok(Request::BrowserExecute { .. }
            | Request::ComputerExecute { .. }
            | Request::ComputerStep { .. })
    )
}

pub fn local_control_request(bytes: &[u8]) -> Option<(String, crate::ControlKind)> {
    match yonder_protocol::decode(bytes).ok()? {
        Request::Control { params, .. } => Some((
            params.task_id,
            match params.kind {
                yonder_protocol::ControlKind::Pause => crate::ControlKind::Pause,
                yonder_protocol::ControlKind::Cancel => crate::ControlKind::Cancel,
                yonder_protocol::ControlKind::Takeover => crate::ControlKind::Takeover,
            },
        )),
        _ => None,
    }
}
pub fn is_takeover_request(bytes: &[u8]) -> bool {
    matches!(yonder_protocol::decode(bytes),Ok(Request::Control{params,..}) if params.kind==yonder_protocol::ControlKind::Takeover)
}
pub fn local_takeover_request(
    task_id: &str,
    expected: u64,
    now_ms: u64,
) -> Result<Vec<u8>, crate::Error> {
    if !crate::valid_id(task_id) || expected == 0 || now_ms > 9_007_199_254_730_000 {
        return Err(crate::Error::InvalidInput);
    }
    yonder_protocol::encode_request(&Request::Control {
        jsonrpc: yonder_protocol::Version::V2,
        request_id: format!("user_takeover_{expected}"),
        params: yonder_protocol::ControlParams {
            agent_id: "desktop".into(),
            capability: Capability::TaskControl,
            deadline: now_ms + 10_000,
            task_id: task_id.into(),
            expected_sequence: expected.to_string(),
            kind: yonder_protocol::ControlKind::Takeover,
        },
    })
    .map_err(|_| crate::Error::StorageUnavailable)
}
pub fn computer_request_task(bytes: &[u8]) -> Option<String> {
    match yonder_protocol::decode(bytes).ok()? {
        Request::ComputerExecute { params, .. } => Some(params.task_id),
        Request::ComputerStep { params, .. } => Some(params.task_id),
        _ => None,
    }
}
pub fn response_is_stopped_control(bytes: &[u8]) -> bool {
    matches!(yonder_protocol::decode_response(bytes),Ok(Response::Success{result:QueryResult::Control{control,..},..}) if control.phase==yonder_protocol::ControlPhase::Stopped)
}
pub fn response_is_computer_success(bytes: &[u8]) -> bool {
    matches!(
        yonder_protocol::decode_response(bytes),
        Ok(Response::Success {
            result: QueryResult::Computer { .. } | QueryResult::ComputerStep { .. },
            ..
        })
    )
}
pub fn terminal_presentation(request: &[u8], response: &[u8]) -> Option<(&'static str, String)> {
    let expected = match yonder_protocol::decode(request).ok()? {
        Request::Complete { .. } => "success",
        Request::Fail { .. } => "failed",
        Request::BrowserExecute { params, .. } if params.operation == BrowserOperation::Finish => {
            "success"
        }
        _ => return None,
    };
    let task = match yonder_protocol::decode_response(response).ok()? {
        Response::Success {
            result: QueryResult::Snapshot { task } | QueryResult::Browser { task, .. },
            ..
        } => task,
        _ => return None,
    };
    let state = match task.status {
        yonder_protocol::TaskStatus::Completed => "success",
        yonder_protocol::TaskStatus::Failed => "failed",
        _ => return None,
    };
    if state != expected {
        return None;
    }
    Some((state, format!("{}:{}:{state}", task.task_id, task.sequence)))
}

pub struct GatewaySession<'a> {
    auth: AuthContext<'a>,
    platform: Platform,
    negotiated: bool,
    can_create: bool,
    can_cancel: bool,
    can_name: bool,
    can_steps: bool,
    can_attempt_results: bool,
    can_controls: bool,
    can_focus: bool,
    can_advance: bool,
    can_browser: bool,
    can_browser_read: bool,
    can_running_filter: bool,
    can_wait_for_user: bool,
    can_presentation: bool,
    browser_available: bool,
    can_computer: bool,
    can_computer_step: bool,
    can_complete: bool,
    can_fail: bool,
    computer_available: bool,
    computer_permission_required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, Task, Transition};

    struct NoStore;
    impl TaskStore for NoStore {
        fn create(&mut self, _: &str, _: &str, _: crate::TaskSource) -> Result<Task, Error> {
            panic!("握手不得访问存储")
        }
        fn get(&mut self, _: &str) -> Result<Task, Error> {
            panic!("门禁不得访问存储")
        }
        fn list(
            &mut self,
            _: Option<&str>,
            _: Option<&str>,
            _: bool,
            _: usize,
        ) -> Result<Vec<Task>, Error> {
            panic!("门禁不得访问存储")
        }
        fn running(&mut self, _: usize) -> Result<Vec<Task>, Error> {
            panic!("握手不得访问存储")
        }
        fn events(&mut self, _: &str, _: u64, _: usize) -> Result<Vec<Transition>, Error> {
            panic!("门禁不得访问存储")
        }
        fn commit(&mut self, _: &str, _: u64, _: Transition) -> Result<(), Error> {
            panic!("握手不得访问存储")
        }
    }

    #[test]
    fn handshake_negotiates_and_isolates_sessions_without_storage() {
        fn request(method: &str, extra: &str, agent: &str) -> Vec<u8> {
            format!(r#"{{"jsonrpc":"2.0","id":"r1","method":"{method}","params":{{"agent_id":"{agent}","capability":"task.read","deadline":2000,{extra}}}}}"#).into_bytes()
        }
        fn failure(response: Response, code: i32) {
            assert!(
                matches!(&response, Response::Failure { id: Some(id), error, .. } if id == "r1" && error.code == code),
                "预期 {code}，实际 {response:?}"
            );
        }
        let mut session = GatewaySession::new(AuthContext::Agent("a1"), Platform::Macos);
        let mut other = GatewaySession::new(AuthContext::Agent("a1"), Platform::Windows);
        let queries = [
            request("task.list", r#""limit":1"#, "a1"),
            request("task.get", r#""task_id":"t1""#, "a1"),
            request(
                "task.events",
                r#""task_id":"t1","after_sequence":"0","limit":1"#,
                "a1",
            ),
        ];
        for query in &queries {
            failure(session.handle(&mut NoStore, query, 1000), -32002);
        }
        let hello = request(
            "gateway.hello",
            r#""protocol_version":{"major":1,"minor":99}"#,
            "a1",
        );
        failure(session.handle(&mut NoStore, &hello, 2000), -32001);
        for platform in [Platform::Macos, Platform::Windows] {
            let target = if platform == Platform::Macos {
                &mut session
            } else {
                &mut other
            };
            for _ in 0..2 {
                let response = target.handle(&mut NoStore, &hello, 1000);
                assert_eq!(
                    response,
                    Response::Success {
                        jsonrpc: Version::V2,
                        id: "r1".into(),
                        result: QueryResult::Hello {
                            protocol_version: PROTOCOL,
                            platform,
                            capabilities: vec![CapabilityInfo {
                                name: Capability::TaskRead,
                                version: PROTOCOL,
                                availability: Availability::Available,
                                reason: None
                            }]
                        }
                    }
                );
                assert!(!yonder_protocol::encode(&response).unwrap().is_empty());
            }
        }
        let mut fresh = GatewaySession::new(AuthContext::Agent("a1"), Platform::Macos);
        failure(fresh.handle(&mut NoStore, &queries[0], 1000), -32002);
        for method in ["gateway.hello", "task.list"] {
            let extra = if method == "gateway.hello" {
                r#""protocol_version":{"major":1,"minor":0}"#
            } else {
                r#""limit":1"#
            };
            failure(
                session.handle(&mut NoStore, &request(method, extra, "b2"), 1000),
                -32003,
            );
        }
        failure(
            session.handle(
                &mut NoStore,
                &request(
                    "gateway.hello",
                    r#""protocol_version":{"major":2,"minor":0}"#,
                    "a1",
                ),
                1000,
            ),
            -32010,
        );
        for query in &queries {
            failure(session.handle(&mut NoStore, query, 1000), -32002);
        }
        assert!(other.negotiated);
        assert!(
            matches!(session.handle(&mut NoStore, b"{", 1000), Response::Failure { id: None, error, .. } if error.code == -32700)
        );
        failure(
            query::handle(&mut NoStore, AuthContext::Agent("a1"), &hello, 1000),
            -32002,
        );
    }

    #[test]
    fn execution_notification_only_accepts_decoded_execution_requests() {
        let computer = br#"{"jsonrpc":"2.0","id":"r1","method":"computer.execute","params":{"agent_id":"a1","capability":"computer.execute","deadline":2000,"task_id":"t1","expected_sequence":"2","tool_name":"press_key","arguments":{"key":"tab"}}}"#;
        let step = br#"{"jsonrpc":"2.0","id":"r2","method":"computer.step","params":{"agent_id":"a1","capability":"computer.execute","deadline":2000,"task_id":"t1","expected_sequence":"2","step_id":"press","label":"press key","tool_name":"press_key","arguments":{"key":"tab"}}}"#;
        assert!(is_execution_request(computer));
        assert!(is_execution_request(step));
        assert!(!is_execution_request(br#"{"jsonrpc":"2.0","id":"r1","method":"task.get","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"t1"}}"#));
        assert!(!is_execution_request(br#"{"method":"computer.execute"}"#));
    }

    #[test]
    fn local_takeover_is_host_built_and_distinguishable() {
        let request = local_takeover_request("task_1", 7, 1000).unwrap();
        assert!(is_takeover_request(&request));
        assert!(
            matches!(yonder_protocol::decode(&request).unwrap(),Request::Control{params,..} if params.agent_id=="desktop"&&params.expected_sequence=="7"&&params.deadline==11_000&&params.kind==yonder_protocol::ControlKind::Takeover)
        );
        assert_eq!(
            local_takeover_request("../task", 7, 1000),
            Err(crate::Error::InvalidInput)
        );
    }

    #[test]
    fn terminal_presentation_requires_a_successful_terminal_write() {
        let complete = r#"{"jsonrpc":"2.0","id":"r1","method":"task.complete","params":{"agent_id":"agent","capability":"task.complete","deadline":1000,"task_id":"task_1","expected_sequence":"4"}}"#;
        let fail = r#"{"jsonrpc":"2.0","id":"r1","method":"task.fail","params":{"agent_id":"agent","capability":"task.fail","deadline":1000,"task_id":"task_1","expected_sequence":"4"}}"#;
        let get = r#"{"jsonrpc":"2.0","id":"r2","method":"task.get","params":{"agent_id":"agent","capability":"task.read","deadline":1000,"task_id":"task_1"}}"#;
        let success = r#"{"jsonrpc":"2.0","id":"r1","result":{"kind":"snapshot","task":{"task_id":"task_1","owner_agent_id":"agent","name":"测试","status":"completed","sequence":"5"}}}"#;
        let failed = r#"{"jsonrpc":"2.0","id":"r1","result":{"kind":"snapshot","task":{"task_id":"task_1","owner_agent_id":"agent","name":"测试","status":"failed","sequence":"5"}}}"#;
        let rejected =
            r#"{"jsonrpc":"2.0","id":"r1","error":{"code":-32011,"message":"任务序号冲突"}}"#;
        assert_eq!(
            terminal_presentation(complete.as_bytes(), success.as_bytes()),
            Some(("success", "task_1:5:success".into()))
        );
        assert_eq!(
            terminal_presentation(fail.as_bytes(), failed.as_bytes()),
            Some(("failed", "task_1:5:failed".into()))
        );
        assert_eq!(
            terminal_presentation(complete.as_bytes(), failed.as_bytes()),
            None
        );
        assert_eq!(
            terminal_presentation(get.as_bytes(), success.as_bytes()),
            None
        );
        assert_eq!(
            terminal_presentation(complete.as_bytes(), rejected.as_bytes()),
            None
        );
    }
}

impl<'a> GatewaySession<'a> {
    /// 正式组合根复用唯一协议编码，不需要直接依赖协议库。
    pub fn handle_encoded(
        &mut self,
        store: &mut impl TaskStore,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<Vec<u8>, crate::Error> {
        self.handle_encoded_with_create_signal(store, bytes, now_ms)
            .map(|result| result.0)
    }

    /// 组合根可用成功创建信号驱动短暂展示；响应字节与 wire 协议不变。
    pub fn handle_encoded_with_create_signal(
        &mut self,
        store: &mut impl TaskStore,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<(Vec<u8>, bool), crate::Error> {
        let create = matches!(yonder_protocol::decode(bytes), Ok(Request::Create { .. }));
        let response = self.handle(store, bytes, now_ms);
        let accepted = create && matches!(response, Response::Success { .. });
        Ok((
            yonder_protocol::encode(&response).map_err(|_| crate::Error::StorageUnavailable)?,
            accepted,
        ))
    }

    pub fn handle_encoded_with_runtimes(
        &mut self,
        store: &mut impl TaskStore,
        admission: &Admission,
        port: Option<&dyn BrowserUsePort>,
        computer: Option<&dyn ComputerUsePort>,
        targets: Option<&dyn WorkTargetPort>,
        computer_permission_required: bool,
        host_session_id: &str,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<(Vec<u8>, bool), crate::Error> {
        self.browser_available = port.is_some();
        self.computer_available = computer.is_some() && targets.is_some();
        self.computer_permission_required = computer_permission_required;
        let request = match yonder_protocol::decode(bytes) {
            Ok(request) => request,
            Err(_) => return self.handle_encoded_with_create_signal(store, bytes, now_ms),
        };
        if !matches!(
            request,
            Request::StepAdvance { .. }
                | Request::BrowserExecute { .. }
                | Request::ComputerExecute { .. }
                | Request::ComputerStep { .. }
                | Request::Complete { .. }
                | Request::Fail { .. }
                | Request::WaitForUser { .. }
        ) {
            return self.handle_encoded_with_create_signal(store, bytes, now_ms);
        }
        let id = request.request_id().to_owned();
        let result = query::validate(&request, self.auth, now_ms).and_then(|()| match request {
            Request::WaitForUser { params, .. } => {
                if !self.negotiated {
                    return Err(RpcError::new(-32002, "请先完成Gateway握手"));
                }
                if !self.can_wait_for_user {
                    return Err(RpcError::new(-32010, "等待用户需要协议1.17及执行能力"));
                }
                let task = crate::wait_for_user(
                    store,
                    self.auth,
                    &params.task_id,
                    yonder_protocol::sequence(&params.expected_sequence)?,
                    &params.reason,
                )
                .map_err(query::error)?;
                admission
                    .release_task_after_stop(&params.task_id)
                    .map_err(|_| RpcError::new(-32603, "任务资源释放失败"))?;
                Ok(QueryResult::Snapshot {
                    task: snapshot(task),
                })
            }
            Request::StepAdvance { params, .. } => {
                if !self.negotiated {
                    return Err(RpcError::new(-32002, "请先完成Gateway握手"));
                }
                if !self.can_advance {
                    return Err(RpcError::new(-32010, "步骤推进需要协议1.7及执行能力"));
                }
                let (task, _) = crate::get_with_step(store, self.auth, &params.task_id)
                    .map_err(query::error)?;
                if task.sequence != yonder_protocol::sequence(&params.expected_sequence)? {
                    return Err(RpcError::new(-32011, "任务序号冲突"));
                }
                let attempt = store
                    .get_attempt(&params.task_id)
                    .map_err(query::error)?
                    .ok_or_else(|| RpcError::new(-32012, "当前步骤没有可推进的执行结果"))?;
                let (task, _) =
                    crate::advance_after_observe(store, &params.task_id, &attempt.attempt_id)
                        .map_err(query::error)?;
                Ok(QueryResult::Snapshot {
                    task: snapshot(task),
                })
            }
            Request::BrowserExecute { params, .. } => {
                if !self.negotiated {
                    return Err(RpcError::new(-32002, "请先完成Gateway握手"));
                }
                if !self.can_browser {
                    return Err(RpcError::new(
                        -32010,
                        "浏览器执行需要协议1.8及ego-lite Bridge",
                    ));
                }
                let port = port.ok_or_else(|| RpcError::new(-32020, "ego-lite不可用"))?;
                let operation = match params.operation {
                    BrowserOperation::Create => "create",
                    BrowserOperation::Observe => "observe",
                    BrowserOperation::HandOff => "hand-off",
                    BrowserOperation::TakeOver => "take-over",
                    BrowserOperation::Finish => "finish",
                };
                let (task, reference) = crate::browser_use::execute_agent_action(
                    store,
                    admission,
                    port,
                    self.auth,
                    &params.task_id,
                    yonder_protocol::sequence(&params.expected_sequence)?,
                    operation,
                    host_session_id,
                )
                .map_err(query::error)?;
                Ok(QueryResult::Browser {
                    task: snapshot(task),
                    reference: browser_reference(reference),
                })
            }
            Request::ComputerExecute { params, .. } => {
                if !self.negotiated {
                    return Err(RpcError::new(-32002, "请先完成Gateway握手"));
                }
                if !self.can_computer {
                    return Err(RpcError::new(-32010, "桌面执行需要协议1.11及CUA Runtime"));
                }
                let (port, targets) = (
                    computer.ok_or_else(|| RpcError::new(-32020, "CUA Runtime不可用"))?,
                    targets.ok_or_else(|| RpcError::new(-32020, "桌面目标解析不可用"))?,
                );
                let arguments = yonder_protocol::sdk_arguments_json(&params.arguments)?;
                let (task, result, _) = crate::computer_use::execute_agent_action(
                    store,
                    admission,
                    port,
                    targets,
                    self.auth,
                    &params.task_id,
                    yonder_protocol::sequence(&params.expected_sequence)?,
                    &params.tool_name,
                    &arguments,
                    host_session_id,
                )
                .map_err(query::error)?;
                Ok(QueryResult::Computer {
                    task: snapshot(task),
                    attempt_result: attempt_result(result),
                })
            }
            Request::ComputerStep { params, .. } => {
                if !self.negotiated {
                    return Err(RpcError::new(-32002, "请先完成Gateway握手"));
                }
                if !self.can_computer_step {
                    return Err(RpcError::new(-32010, "桌面步骤需要协议1.12及CUA Runtime"));
                }
                let (port, targets) = (
                    computer.ok_or_else(|| RpcError::new(-32020, "CUA Runtime不可用"))?,
                    targets.ok_or_else(|| RpcError::new(-32020, "桌面目标解析不可用"))?,
                );
                let arguments = yonder_protocol::sdk_arguments_json(&params.arguments)?;
                let (task, result, observation) = crate::computer_use::execute_agent_step(
                    store,
                    admission,
                    port,
                    targets,
                    self.auth,
                    &params.task_id,
                    yonder_protocol::sequence(&params.expected_sequence)?,
                    &params.step_id,
                    &params.label,
                    &params.tool_name,
                    &arguments,
                    host_session_id,
                )
                .map_err(query::error)?;
                let result = attempt_result(result);
                Ok(QueryResult::ComputerStep {
                    task_id: task.id,
                    status: query::status(task.status),
                    sequence: task.sequence.to_string(),
                    action_succeeded: result.action_succeeded,
                    unknown_reason: result.unknown_reason,
                    observation: observation.map(|value| ProtocolComputerObservation {
                        element_count: value.element_count,
                        screenshot_path: value.screenshot_path,
                        screenshot_mime: value.screenshot_mime,
                        target_visible: value.target_visible,
                    }),
                })
            }
            Request::Complete { params, .. } => {
                if !self.negotiated {
                    return Err(RpcError::new(-32002, "请先完成Gateway握手"));
                }
                if !self.can_complete {
                    return Err(RpcError::new(-32010, "任务完成需要协议1.10及CUA Runtime"));
                }
                let task = crate::computer_use::complete_agent_task(
                    store,
                    admission,
                    self.auth,
                    &params.task_id,
                    yonder_protocol::sequence(&params.expected_sequence)?,
                )
                .map_err(query::error)?;
                if let Some(port) = computer {
                    port.end_session();
                }
                Ok(QueryResult::Snapshot {
                    task: snapshot(task),
                })
            }
            Request::Fail { params, .. } => {
                if !self.negotiated {
                    return Err(RpcError::new(-32002, "请先完成Gateway握手"));
                }
                if !self.can_fail {
                    return Err(RpcError::new(
                        -32010,
                        "任务失败终结需要协议1.18及CUA Runtime",
                    ));
                }
                let task = crate::computer_use::fail_agent_task(
                    store,
                    admission,
                    self.auth,
                    &params.task_id,
                    yonder_protocol::sequence(&params.expected_sequence)?,
                )
                .map_err(query::error)?;
                if let Some(port) = computer {
                    port.end_session();
                }
                Ok(QueryResult::Snapshot {
                    task: snapshot(task),
                })
            }
            _ => unreachable!(),
        });
        let response = match result {
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
        };
        Ok((
            yonder_protocol::encode(&response).map_err(|_| crate::Error::StorageUnavailable)?,
            false,
        ))
    }

    pub fn new(auth: AuthContext<'a>, platform: Platform) -> Self {
        Self {
            auth,
            platform,
            negotiated: false,
            can_create: false,
            can_cancel: false,
            can_name: false,
            can_steps: false,
            can_attempt_results: false,
            can_controls: false,
            can_focus: false,
            can_advance: false,
            can_browser: false,
            can_browser_read: false,
            can_running_filter: false,
            can_wait_for_user: false,
            can_presentation: false,
            browser_available: false,
            can_computer: false,
            can_computer_step: false,
            can_complete: false,
            can_fail: false,
            computer_available: false,
            computer_permission_required: false,
        }
    }

    pub fn handle(&mut self, store: &mut impl TaskStore, bytes: &[u8], now_ms: u64) -> Response {
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
        let id = request.request_id().to_owned();
        if let Err(error) = query::validate(&request, self.auth, now_ms) {
            return Response::Failure {
                jsonrpc: Version::V2,
                id: Some(id),
                error,
            };
        }
        if let Request::Create { params, .. } = &request {
            let result = if !self.negotiated {
                Err(RpcError::new(-32002, "请先完成Gateway握手"))
            } else if !matches!(self.auth, AuthContext::Agent(_)) {
                Err(RpcError::new(-32003, "创建任务需要Agent身份"))
            } else if !self.can_create {
                Err(RpcError::new(-32010, "任务登记需要协议1.1及登记能力"))
            } else if params.name.is_some() && !self.can_name {
                Err(RpcError::new(-32010, "任务名称需要协议1.3"))
            } else if self.can_name && params.name.is_none() {
                Err(RpcError::new(-32602, "Agent须提供任务名称"))
            } else {
                crate::register(
                    store,
                    self.auth,
                    &params.idempotency_key,
                    &params.description,
                    params.name.as_deref(),
                    crate::TaskSource::LocalAgent,
                )
                .map_err(query::error)
                .map(|task| {
                    let mut snapshot = query::summary(task);
                    if !self.can_name {
                        snapshot.name = None;
                    }
                    if !self.can_presentation {
                        snapshot.source = None;
                    }
                    QueryResult::Snapshot { task: snapshot }
                })
            };
            return match result {
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
            };
        }
        if matches!(request, Request::Cancel { .. }) && self.negotiated && !self.can_cancel {
            return Response::Failure {
                jsonrpc: Version::V2,
                id: Some(id),
                error: RpcError::new(-32010, "任务取消需要协议1.2及取消能力"),
            };
        }
        if matches!(request, Request::Control { .. }) && (!self.negotiated || !self.can_controls) {
            return Response::Failure {
                jsonrpc: Version::V2,
                id: Some(id),
                error: RpcError::new(-32010, "任务控制需要协议1.6及控制能力"),
            };
        }
        if let Request::StepDeclare { params, .. } = &request {
            let result = if !self.negotiated {
                Err(RpcError::new(-32002, "请先完成Gateway握手"))
            } else if !self.can_steps {
                Err(RpcError::new(-32010, "步骤声明需要协议1.4及存储能力"))
            } else if !matches!(self.auth, AuthContext::Agent(_)) {
                Err(RpcError::new(-32003, "步骤声明需要Agent身份"))
            } else {
                crate::declare_step(
                    &mut *store,
                    self.auth,
                    &params.task_id,
                    yonder_protocol::sequence(&params.expected_sequence).unwrap_or(0),
                    &params.step_id,
                    &params.label,
                )
                .map_err(query::error)
                .map(|(task, step)| {
                    let mut snapshot = query::summary(task);
                    if !self.can_presentation {
                        snapshot.source = None;
                    }
                    QueryResult::Step {
                        task: snapshot,
                        step: Some(yonder_protocol::StepDeclaration {
                            step_id: step.step_id,
                            label: step.label,
                            accepted_sequence: step.accepted_sequence.to_string(),
                        }),
                    }
                })
            };
            return match result {
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
            };
        }
        if matches!(request, Request::StepGet { .. }) && self.negotiated && !self.can_steps {
            return Response::Failure {
                jsonrpc: Version::V2,
                id: Some(id),
                error: RpcError::new(-32010, "步骤读取需要协议1.4及存储能力"),
            };
        }
        if matches!(request, Request::BrowserGet { .. })
            && self.negotiated
            && !self.can_browser_read
        {
            return Response::Failure {
                jsonrpc: Version::V2,
                id: Some(id),
                error: RpcError::new(-32010, "浏览器引用读取需要协议1.15及存储能力"),
            };
        }
        if matches!(&request, Request::List { params, .. } if params.running_only)
            && self.negotiated
            && !self.can_running_filter
        {
            return Response::Failure {
                jsonrpc: Version::V2,
                id: Some(id),
                error: RpcError::new(-32010, "运行中任务筛选需要协议1.16及存储能力"),
            };
        }
        if !matches!(request, Request::Hello { .. }) && self.negotiated {
            let mut response = query::handle_request_versioned(
                store,
                self.auth,
                request,
                now_ms,
                self.can_steps,
                self.can_attempt_results,
                self.can_wait_for_user,
            );
            // 旧客户端严格拒绝未知字段，只投影响应，不改存储的名称。
            if !self.can_name {
                if let Response::Success { result, .. } = &mut response {
                    match result {
                        QueryResult::Snapshot { task } => task.name = None,
                        QueryResult::Tasks { tasks, .. } => {
                            for task in tasks {
                                task.name = None;
                            }
                        }
                        QueryResult::Step { task, .. } => task.name = None,
                        QueryResult::BrowserState { task, .. } => task.name = None,
                        QueryResult::Control { task, .. } => task.name = None,
                        QueryResult::Browser { task, .. } => task.name = None,
                        QueryResult::Computer { task, .. } => task.name = None,
                        _ => {}
                    }
                }
            }
            if !self.can_presentation {
                if let Response::Success { result, .. } = &mut response {
                    match result {
                        QueryResult::Snapshot { task } => {
                            task.source = None;
                            task.current_step = None;
                            task.observation = None;
                            task.next_intent = None;
                        }
                        QueryResult::Tasks { tasks, .. } => {
                            for task in tasks {
                                task.source = None;
                                task.current_step = None;
                                task.observation = None;
                                task.next_intent = None;
                            }
                        }
                        QueryResult::Step { task, .. } => {
                            task.source = None;
                            task.current_step = None;
                            task.observation = None;
                            task.next_intent = None;
                        }
                        QueryResult::BrowserState { task, .. } => {
                            task.source = None;
                            task.current_step = None;
                            task.observation = None;
                            task.next_intent = None;
                        }
                        QueryResult::Control { task, .. } => {
                            task.source = None;
                            task.current_step = None;
                            task.observation = None;
                            task.next_intent = None;
                        }
                        QueryResult::Browser { task, .. } => {
                            task.source = None;
                            task.current_step = None;
                            task.observation = None;
                            task.next_intent = None;
                        }
                        QueryResult::Computer { task, .. } => {
                            task.source = None;
                            task.current_step = None;
                            task.observation = None;
                            task.next_intent = None;
                        }
                        _ => {}
                    }
                }
            }
            if !self.can_focus {
                if let Response::Success {
                    result: QueryResult::Control { control, .. },
                    ..
                } = &mut response
                {
                    control.focus_phase = None;
                    control.focus_failure = None;
                }
            }
            return response;
        }
        let result = (|| {
            if let Request::Hello { params, .. } = &request {
                self.negotiated = params.protocol_version.major == PROTOCOL.major;
                self.can_cancel = self.negotiated
                    && params.protocol_version.minor >= 2
                    && store.supports_pending_cancel();
                self.can_create = self.negotiated
                    && params.protocol_version.minor >= 1
                    && store.supports_registration()
                    && matches!(self.auth, AuthContext::Agent(_));
                self.can_name = self.negotiated
                    && params.protocol_version.minor >= 3
                    && store.supports_registration();
                self.can_steps = self.negotiated
                    && params.protocol_version.minor >= 4
                    && store.supports_step_declarations();
                self.can_attempt_results = self.can_steps
                    && params.protocol_version.minor >= 5
                    && store.supports_execution_attempts();
                self.can_controls = self.can_attempt_results
                    && params.protocol_version.minor >= 6
                    && store.supports_controls();
                self.can_focus = self.can_controls && params.protocol_version.minor >= 13;
                self.can_advance = self.can_attempt_results && params.protocol_version.minor >= 7;
                self.can_browser = self.can_advance
                    && params.protocol_version.minor >= 8
                    && self.browser_available
                    && store.supports_browser_references();
                self.can_browser_read = self.negotiated
                    && params.protocol_version.minor >= 15
                    && store.supports_browser_references();
                self.can_running_filter = self.negotiated
                    && params.protocol_version.minor >= 16
                    && store.supports_running_filter();
                self.can_wait_for_user = self.can_advance
                    && params.protocol_version.minor >= 17
                    && store.supports_wait_for_user();
                self.can_presentation = self.negotiated
                    && params.protocol_version.minor >= 19
                    && store.supports_presentation();
                self.can_computer = self.can_advance
                    && params.protocol_version.minor >= 11
                    && self.computer_available
                    && !self.computer_permission_required;
                self.can_computer_step = self.can_computer && params.protocol_version.minor >= 12;
                self.can_complete = self.can_computer && params.protocol_version.minor >= 10;
                self.can_fail = self.can_complete && params.protocol_version.minor >= 18;
                if !self.negotiated {
                    return Err(RpcError::new(-32010, "协议主版本不兼容"));
                }
                let mut capabilities = vec![CapabilityInfo {
                    name: Capability::TaskRead,
                    version: ProtocolVersion {
                        major: 1,
                        minor: if self.can_presentation {
                            19
                        } else if self.can_fail {
                            18
                        } else if self.can_wait_for_user {
                            17
                        } else if self.can_running_filter {
                            16
                        } else if self.can_browser_read {
                            15
                        } else if self.can_focus {
                            13
                        } else if self.can_computer_step {
                            12
                        } else if self.can_computer {
                            11
                        } else if self.can_browser {
                            8
                        } else if self.can_advance {
                            7
                        } else if self.can_controls {
                            6
                        } else if self.can_attempt_results {
                            5
                        } else if self.can_steps {
                            4
                        } else if self.can_name {
                            3
                        } else {
                            0
                        },
                    },
                    availability: Availability::Available,
                    reason: None,
                }];
                let version = ProtocolVersion {
                    major: 1,
                    minor: if self.can_presentation {
                        19
                    } else if self.can_fail {
                        18
                    } else if self.can_wait_for_user {
                        17
                    } else if self.can_running_filter {
                        16
                    } else if self.can_browser_read {
                        15
                    } else if params.offered_capabilities.as_deref().is_some_and(|value| {
                        value.contains(&yonder_protocol::OfferedCapability::UserInput)
                    }) {
                        14
                    } else if self.can_focus {
                        13
                    } else if self.can_computer_step {
                        12
                    } else if self.can_computer {
                        11
                    } else if self.can_browser {
                        8
                    } else if self.can_advance {
                        7
                    } else if self.can_controls {
                        6
                    } else if self.can_attempt_results {
                        5
                    } else if self.can_steps {
                        4
                    } else if self.can_name {
                        3
                    } else if self.can_cancel {
                        2
                    } else {
                        u16::from(self.can_create)
                    },
                };
                if self.can_create {
                    capabilities.push(CapabilityInfo {
                        name: Capability::TaskCreate,
                        version: ProtocolVersion {
                            major: 1,
                            minor: if self.can_name { 3 } else { 1 },
                        },
                        availability: Availability::Available,
                        reason: None,
                    });
                }
                if self.can_cancel {
                    capabilities.push(CapabilityInfo {
                        name: Capability::TaskCancel,
                        version: ProtocolVersion { major: 1, minor: 2 },
                        availability: Availability::Available,
                        reason: Some("仅支持未开始任务取消".into()),
                    });
                }
                if self.can_steps {
                    capabilities.push(CapabilityInfo {
                        name: Capability::TaskStepDeclare,
                        version: ProtocolVersion { major: 1, minor: 4 },
                        availability: Availability::Available,
                        reason: Some("仅支持created任务声明".into()),
                    });
                }
                if self.can_controls {
                    capabilities.push(CapabilityInfo {
                        name: Capability::TaskControl,
                        version: ProtocolVersion {
                            major: 1,
                            minor: if self.can_focus { 13 } else { 6 },
                        },
                        availability: Availability::Available,
                        reason: Some("登记停止请求，执行器在步骤边界确认".into()),
                    });
                }
                if self.can_advance {
                    capabilities.push(CapabilityInfo {
                        name: Capability::TaskStepAdvance,
                        version: ProtocolVersion { major: 1, minor: 7 },
                        availability: Availability::Available,
                        reason: Some("仅推进已完成Observe的步骤".into()),
                    });
                }
                if params.protocol_version.minor >= 8 && store.supports_browser_references() {
                    capabilities.push(CapabilityInfo {
                        name: Capability::BrowserExecute,
                        version: ProtocolVersion { major: 1, minor: 8 },
                        availability: if self.can_browser {
                            Availability::Available
                        } else {
                            Availability::DependencyMissing
                        },
                        reason: (!self.can_browser).then(|| "ego-lite Bridge不可用".into()),
                    });
                }
                if params.protocol_version.minor >= 11 && store.supports_execution_attempts() {
                    capabilities.push(CapabilityInfo {
                        name: Capability::ComputerExecute,
                        version: ProtocolVersion {
                            major: 1,
                            minor: if params.protocol_version.minor >= 12 {
                                12
                            } else {
                                11
                            },
                        },
                        availability: if self.can_computer {
                            Availability::Available
                        } else if self.computer_permission_required {
                            Availability::PermissionRequired
                        } else {
                            Availability::DependencyMissing
                        },
                        reason: (!self.can_computer).then(|| {
                            if self.computer_permission_required {
                                "Yonda需要macOS辅助功能权限".into()
                            } else {
                                "CUA Runtime不可用".into()
                            }
                        }),
                    });
                }
                if self.can_complete {
                    capabilities.push(CapabilityInfo {
                        name: Capability::TaskComplete,
                        version: ProtocolVersion {
                            major: 1,
                            minor: 10,
                        },
                        availability: Availability::Available,
                        reason: Some("仅完成已Observe并推进边界的CUA任务".into()),
                    });
                }
                if self.can_fail {
                    capabilities.push(CapabilityInfo {
                        name: Capability::TaskFail,
                        version: ProtocolVersion {
                            major: 1,
                            minor: 18,
                        },
                        availability: Availability::Available,
                        reason: Some("仅终结已Observe失败并推进边界的CUA任务".into()),
                    });
                }
                if self.can_wait_for_user {
                    capabilities.push(CapabilityInfo {
                        name: Capability::TaskWaitForUser,
                        version: ProtocolVersion {
                            major: 1,
                            minor: 17,
                        },
                        availability: Availability::Available,
                        reason: Some("仅在已Observe并推进的步骤边界等待".into()),
                    });
                }
                return Ok(QueryResult::Hello {
                    protocol_version: version,
                    platform: self.platform,
                    capabilities,
                });
            }
            Err(RpcError::new(-32002, "请先完成 Gateway 握手"))
        })();
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
}

fn snapshot(task: crate::Task) -> TaskSnapshot {
    query::summary(task)
}
fn browser_reference(value: BrowserReferenceRecord) -> BrowserReference {
    BrowserReference {
        external_task_ref: value.external_task_ref,
        ownership: value.ownership,
        managed_pages: u16::try_from(value.managed_pages).unwrap_or(u16::MAX),
        finished: value.finished,
        updated_sequence: value.updated_sequence.to_string(),
    }
}
fn attempt_result(value: crate::AttemptResultRecord) -> ProtocolAttemptResult {
    let (phase, action_succeeded, observe_valid, unknown_reason) = match value.conclusion {
        crate::AttemptConclusion::Observed { action_succeeded } => (
            AttemptResultPhase::Observed,
            Some(action_succeeded),
            true,
            None,
        ),
        crate::AttemptConclusion::Unknown { reason } => (
            AttemptResultPhase::Unknown,
            None,
            false,
            Some(match reason {
                crate::computer_use::UnknownReason::InvalidInput => {
                    AttemptUnknownReason::InvalidInput
                }
                crate::computer_use::UnknownReason::DependencyUnavailable => {
                    AttemptUnknownReason::DependencyUnavailable
                }
                crate::computer_use::UnknownReason::WorkerFailed => {
                    AttemptUnknownReason::WorkerFailed
                }
                crate::computer_use::UnknownReason::TimedOut => AttemptUnknownReason::TimedOut,
                crate::computer_use::UnknownReason::InvalidResponse => {
                    AttemptUnknownReason::InvalidResponse
                }
                crate::computer_use::UnknownReason::IdentityMismatch => {
                    AttemptUnknownReason::IdentityMismatch
                }
                crate::computer_use::UnknownReason::ObserveFailed => {
                    AttemptUnknownReason::ObserveFailed
                }
                crate::computer_use::UnknownReason::UserInput => AttemptUnknownReason::UserInput,
            }),
        ),
    };
    ProtocolAttemptResult {
        step_id: value.step_id,
        attempt_id: value.attempt_id,
        worker_instance_id: value.worker_instance_id,
        host_session_id: value.host_session_id,
        phase,
        action_succeeded,
        observe_valid,
        unknown_reason,
    }
}
