//! 每连接握手门禁；身份只能由已认证的宿主传入。
use crate::{
    AuthContext, TaskStore,
    admission::Admission,
    agent_registry::AgentRegistry,
    browser_use::{BrowserReferenceRecord, BrowserUsePort},
    computer_use::{ComputerUsePort, WorkTargetPort},
    file::FilePort,
    file_execution::{self, FileOperation},
    document::DocumentPort,
    document_execution,
    file_authorization::{FileAuthorizationRegistry, FileGrantPurpose},
    command_approval::CommandApprovalRegistry,
    command::{CommandOutcome, CommandPort, NeverCancel},
    command_execution,
    jev_config::JevConfig,
    jev_runtime::JevDecisionPort,
    query,
};
use yonder_protocol::{
    AttemptResult as ProtocolAttemptResult, AttemptResultPhase, AttemptUnknownReason, Availability,
    BrowserOperation, BrowserReference, Capability, CapabilityInfo,
    ComputerObservation as ProtocolComputerObservation, DocumentExecutionResult as ProtocolDocumentExecutionResult,
    DocumentFormat as ProtocolDocumentFormat, FileExecutionResult as ProtocolFileExecutionResult,
    FileGrantPurpose as ProtocolFileGrantPurpose, FileOperation as ProtocolFileOperation,
    FileGrantSummary as ProtocolFileGrantSummary, CommandExecutionOutcome as ProtocolCommandExecutionOutcome,
    CommandExecutionResult as ProtocolCommandExecutionResult, ProtocolVersion, QueryResult, Request, Response,
    RpcError, TaskSnapshot, Version,
};

pub use yonder_protocol::Platform;

/// 当前发布包公开的最高协议版本；握手仍按调用方能力向下协商。
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion { major: 1, minor: 31 };
const PROTOCOL: ProtocolVersion = ProtocolVersion { major: 1, minor: 0 };

pub fn is_execution_request(bytes: &[u8]) -> bool {
    matches!(
        yonder_protocol::decode(bytes),
        Ok(Request::BrowserExecute { .. }
            | Request::ComputerExecute { .. }
            | Request::ComputerStep { .. }
            | Request::PlanExecute { .. }
            | Request::FileExecute { .. }
            | Request::DocumentExecute { .. }
            | Request::CommandExecute { .. })
    )
}

/// 桌面表现层只读取已解码执行请求的任务身份，不解析动作内容。
pub fn execution_request_task(bytes: &[u8]) -> Option<String> {
    match yonder_protocol::decode(bytes).ok()? {
        Request::BrowserExecute { params, .. } => Some(params.task_id),
        Request::ComputerExecute { params, .. } => Some(params.task_id),
        Request::ComputerStep { params, .. } => Some(params.task_id),
        Request::PlanExecute { params, .. } => Some(params.task_id),
        Request::FileExecute { params, .. } => Some(params.task_id),
        Request::DocumentExecute { params, .. } => Some(params.task_id),
        Request::CommandExecute { params, .. } => Some(params.task_id),
        _ => None,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionPresentationHint {
    StoredStep { task_id: String },
    DeclaredStep { task_id: String, label: String },
    PlanSlot { task_id: String, plan_id: String, plan_version: u64 },
}

/// 只在协议校验与连接身份一致后提供当前执行请求的步骤定位信息。
pub fn execution_presentation_hint(
    bytes: &[u8],
    connected_agent_id: &str,
    now_ms: u64,
) -> Option<ExecutionPresentationHint> {
    let request = yonder_protocol::decode(bytes).ok()?;
    request.validate(now_ms).ok()?;
    if request.agent_id() != connected_agent_id {
        return None;
    }
    match request {
        Request::ComputerStep { params, .. } => Some(ExecutionPresentationHint::DeclaredStep {
            task_id: params.task_id,
            label: params.label,
        }),
        Request::PlanExecute { params, .. } => Some(ExecutionPresentationHint::PlanSlot {
            task_id: params.task_id,
            plan_id: params.plan_id,
            plan_version: params.plan_version,
        }),
        Request::BrowserExecute { params, .. } => Some(ExecutionPresentationHint::StoredStep { task_id: params.task_id }),
        Request::ComputerExecute { params, .. } => Some(ExecutionPresentationHint::StoredStep { task_id: params.task_id }),
        Request::FileExecute { params, .. } => Some(ExecutionPresentationHint::StoredStep { task_id: params.task_id }),
        Request::DocumentExecute { params, .. } => Some(ExecutionPresentationHint::StoredStep { task_id: params.task_id }),
        Request::CommandExecute { params, .. } => Some(ExecutionPresentationHint::StoredStep { task_id: params.task_id }),
        _ => None,
    }
}

fn is_agent_write_request(request: &Request) -> bool {
    matches!(
        request,
        Request::BrowserExecute { .. }
            | Request::ComputerExecute { .. }
            | Request::ComputerStep { .. }
            | Request::PlanSubmit { .. }
            | Request::PlanExecute { .. }
            | Request::FileExecute { .. }
            | Request::DocumentExecute { .. }
            | Request::CommandPropose { .. }
            | Request::CommandExecute { .. }
            | Request::Cancel { .. }
            | Request::Create { .. }
            | Request::Control { .. }
            | Request::Complete { .. }
            | Request::Fail { .. }
            | Request::WaitForUser { .. }
            | Request::StepDeclare { .. }
            | Request::StepAdvance { .. }
            | Request::Hello { .. }
    )
}

/// 本地传输在创建会话前只可读取首个握手的逻辑身份；桌面层不得直接解析协议。
pub fn local_hello_agent_id(bytes: &[u8], now_ms: u64) -> Result<String, RpcError> {
    let request = yonder_protocol::decode(bytes)?;
    request.validate(now_ms)?;
    match request {
        Request::Hello { params, .. } => Ok(params.agent_id),
        _ => Err(RpcError::new(-32002, "请先完成Gateway握手")),
    }
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
        Request::PlanExecute { params, .. } => Some(params.task_id),
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
    can_audit: bool,
    can_observation_history: bool,
    can_control_history: bool,
    can_focus_history: bool,
    can_creation_history: bool,
    can_attempt_start_history: bool,
    can_artifact_items: bool,
    can_file_grants: bool,
    can_file_execute: bool,
    can_document_execute: bool,
    can_command_propose: bool,
    can_command_execute: bool,
    /// 计划片段是 1.31 的能力；必须在握手中显式协商，不能靠请求解码绕过。
    can_plan_submit: bool,
    can_plan_execute: bool,
    browser_available: bool,
    can_computer: bool,
    can_computer_step: bool,
    can_complete: bool,
    can_fail: bool,
    computer_available: bool,
    computer_permission_required: bool,
    file_grants_available: bool,
    file_execution_available: bool,
    document_execution_available: bool,
    command_approval_available: bool,
    command_execution_available: bool,
    /// 仅在一次同步 `handle` 调用期间传递给桌面组合根，不是协议或核心状态。
    last_create_was_new: bool,
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

    impl AgentRegistry for NoStore {
        fn register_agent(
            &mut self,
            _: &str,
            _: u64,
        ) -> Result<crate::agent_registry::AgentRegistration, Error> {
            panic!("计划片段能力门禁不得登记Agent")
        }
        fn list_agents(&mut self) -> Result<Vec<crate::agent_registry::AgentRegistration>, Error> {
            panic!("计划片段能力门禁不得读取Agent登记")
        }
        fn set_agent_status(
            &mut self,
            _: &str,
            _: crate::agent_registry::AgentRegistrationStatus,
            _: u64,
        ) -> Result<crate::agent_registry::AgentRegistration, Error> {
            panic!("计划片段能力门禁不得变更Agent登记")
        }
        fn authorize_agent_write(&mut self, _: &str, _: u64) -> Result<(), Error> {
            Ok(())
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
        assert_eq!(execution_request_task(computer).as_deref(), Some("t1"));
        assert_eq!(execution_request_task(step).as_deref(), Some("t1"));
        assert_eq!(
            execution_presentation_hint(step, "a1", 1000),
            Some(ExecutionPresentationHint::DeclaredStep {
                task_id: "t1".into(),
                label: "press key".into(),
            })
        );
        assert!(execution_presentation_hint(step, "other", 1000).is_none());
        assert!(execution_presentation_hint(step, "a1", 3000).is_none());
        assert!(!is_execution_request(br#"{"jsonrpc":"2.0","id":"r1","method":"task.get","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"t1"}}"#));
        assert!(!is_execution_request(br#"{"method":"computer.execute"}"#));
    }

    #[test]
    fn plan_fragment_requests_require_a_negotiated_1_31_capability() {
        let hello_30 = br#"{"jsonrpc":"2.0","id":"hello","method":"gateway.hello","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":30}}}"#;
        let submit = br#"{"jsonrpc":"2.0","id":"submit","method":"task.plan.submit","params":{"agent_id":"a1","capability":"task.plan.submit","deadline":2000,"task_id":"task_1","expected_sequence":"1","plan_id":"plan_1","plan_version":1,"token_budget":100,"slots":[{"step_id":"step_1","label":"\u8f93\u5165","candidates":[{"candidate_id":"candidate_1","tool_name":"type_text","arguments":{"text":"\u6d4b\u8bd5"},"action_kind":"draft-message","target_ref":"test-composer","preconditions":[{"fact":"composer-ready","expected":true}],"expected_observe":[{"fact":"composer-ready","expected":true}]}]}]}}"#;
        let execute = br#"{"jsonrpc":"2.0","id":"execute","method":"task.plan.execute","params":{"agent_id":"a1","capability":"task.plan.execute","deadline":2000,"task_id":"task_1","expected_sequence":"1","plan_id":"plan_1","plan_version":1}}"#;
        let mut store = NoStore;
        let mut session = GatewaySession::new(AuthContext::Agent("a1"), Platform::Macos);
        let admission = Admission::new(1).unwrap();
        let dispatch = |session: &mut GatewaySession<'_>, store: &mut NoStore, request: &[u8]| {
            let (encoded, _) = session
                .handle_encoded_with_runtimes(
                    store, &admission, None, None, None, false, "test-host", request, 1000,
                )
                .unwrap();
            yonder_protocol::decode_response(&encoded).unwrap()
        };
        assert!(matches!(
            dispatch(&mut session, &mut store, hello_30),
            Response::Success { .. }
        ));
        for request in [submit.as_slice(), execute.as_slice()] {
            let response = dispatch(&mut session, &mut store, request);
            assert!(matches!(
                response,
                Response::Failure { ref error, .. } if error.code == -32010
            ), "{response:?}");
        }

        let hello_31 = br#"{"jsonrpc":"2.0","id":"hello","method":"gateway.hello","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":31}}}"#;
        let mut session = GatewaySession::new(AuthContext::Agent("a1"), Platform::Macos);
        assert!(matches!(
            dispatch(&mut session, &mut store, hello_31),
            Response::Success { .. }
        ));
        let response = dispatch(&mut session, &mut store, submit);
        assert!(matches!(
            response,
            Response::Failure { ref error, .. } if error.code == -32010
        ), "{response:?}");
    }

    #[test]
    fn local_first_hello_binds_the_declared_agent_id() {
        let hello = |agent| format!(r#"{{"jsonrpc":"2.0","id":"hello","method":"gateway.hello","params":{{"agent_id":"{agent}","capability":"task.read","deadline":2000,"protocol_version":{{"major":1,"minor":1}}}}}}"#);
        assert_eq!(local_hello_agent_id(hello("agent-a").as_bytes(), 1000), Ok("agent-a".into()));
        assert_eq!(local_hello_agent_id(hello("agent-b").as_bytes(), 1000), Ok("agent-b".into()));
        assert!(local_hello_agent_id(br#"{"jsonrpc":"2.0","id":"list","method":"task.list","params":{"agent_id":"agent-a","capability":"task.read","deadline":2000,"limit":1}}"#, 1000).is_err());
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
        let response = self.handle(store, bytes, now_ms);
        let accepted = self.last_create_was_new && matches!(response, Response::Success { .. });
        Ok((
            yonder_protocol::encode(&response).map_err(|_| crate::Error::StorageUnavailable)?,
            accepted,
        ))
    }

    pub fn handle_encoded_with_runtimes<T>(
        &mut self,
        store: &mut T,
        admission: &Admission,
        port: Option<&dyn BrowserUsePort>,
        computer: Option<&dyn ComputerUsePort>,
        targets: Option<&dyn WorkTargetPort>,
        computer_permission_required: bool,
        host_session_id: &str,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<(Vec<u8>, bool), crate::Error>
    where
        T: TaskStore + AgentRegistry,
    {
        self.handle_encoded_with_runtimes_and_file_grants(
            store,
            admission,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            port,
            computer,
            targets,
            computer_permission_required,
            host_session_id,
            bytes,
            now_ms,
        )
    }

    /// 只有持有可信本机文件授权 Registry 的组合根才可开放协议 1.27 查询。
    pub fn handle_encoded_with_runtimes_and_file_grants<T>(
        &mut self,
        store: &mut T,
        admission: &Admission,
        file_grants: Option<&FileAuthorizationRegistry>,
        command_approvals: Option<&CommandApprovalRegistry>,
        file_port: Option<&dyn FilePort>,
        documents: Option<&dyn DocumentPort>,
        command_port: Option<&dyn CommandPort>,
        jev_config: Option<&JevConfig>,
        jev: Option<&dyn JevDecisionPort>,
        port: Option<&dyn BrowserUsePort>,
        computer: Option<&dyn ComputerUsePort>,
        targets: Option<&dyn WorkTargetPort>,
        computer_permission_required: bool,
        host_session_id: &str,
        bytes: &[u8],
        now_ms: u64,
    ) -> Result<(Vec<u8>, bool), crate::Error>
    where
        T: TaskStore + AgentRegistry,
    {
        self.browser_available = port.is_some();
        self.computer_available = computer.is_some() && targets.is_some();
        self.computer_permission_required = computer_permission_required;
        self.file_grants_available = file_grants.is_some();
        self.file_execution_available = file_grants.is_some() && file_port.is_some();
        self.document_execution_available = file_grants.is_some() && file_port.is_some() && documents.is_some();
        self.command_approval_available = command_approvals.is_some();
        self.command_execution_available = command_approvals.is_some() && command_port.is_some();
        let request = match yonder_protocol::decode(bytes) {
            Ok(request) => request,
            Err(_) => return self.handle_encoded_with_create_signal(store, bytes, now_ms),
        };
        if is_agent_write_request(&request) || matches!(request, Request::FileGrants { .. } | Request::FileExecute { .. } | Request::DocumentExecute { .. } | Request::CommandPropose { .. } | Request::CommandExecute { .. }) {
            let id = request.request_id().to_owned();
            if store
                .authorize_agent_write(self.auth.agent_id(), now_ms)
                .is_err()
            {
                let response = Response::Failure {
                    jsonrpc: Version::V2,
                    id: Some(id),
                    error: RpcError::new(-32003, "Agent已禁用、撤权或未登记"),
                };
                return Ok((
                    yonder_protocol::encode(&response).map_err(|_| crate::Error::StorageUnavailable)?,
                    false,
                ));
            }
        }
        if !matches!(
            request,
            Request::StepAdvance { .. }
                | Request::BrowserExecute { .. }
                | Request::ComputerExecute { .. }
                | Request::ComputerStep { .. }
                | Request::PlanSubmit { .. }
                | Request::PlanExecute { .. }
                | Request::FileExecute { .. }
                | Request::DocumentExecute { .. }
                | Request::CommandPropose { .. }
                | Request::CommandExecute { .. }
                | Request::Complete { .. }
                | Request::Fail { .. }
                | Request::WaitForUser { .. }
                | Request::FileGrants { .. }
        ) {
            return self.handle_encoded_with_create_signal(store, bytes, now_ms);
        }
        let id = request.request_id().to_owned();
        let result = query::validate(&request, self.auth, now_ms).and_then(|()| match request {
            Request::PlanSubmit { params, .. } => {
                if !self.negotiated { return Err(RpcError::new(-32002, "请先完成Gateway握手")); }
                if !self.can_plan_submit { return Err(RpcError::new(-32010, "计划片段提交需要协议1.31及本机片段存储")); }
                let task = crate::plan_fragment::submit(store, self.auth, &params).map_err(query::error)?;
                Ok(QueryResult::Plan { task_id: task.id, plan_id: params.plan_id, plan_version: params.plan_version, sequence: task.sequence.to_string(), disposition: "accepted".into(), handoff_reason: None })
            }
            Request::PlanExecute { params, .. } => {
                if !self.negotiated { return Err(RpcError::new(-32002, "请先完成Gateway握手")); }
                if !self.can_plan_execute { return Err(RpcError::new(-32010, "计划片段执行需要协议1.31及可用的macOS CUA运行时")); }
                let config=jev_config.ok_or_else(||RpcError::new(-32020,"Jev计划执行组合根不可用"))?;
                let jev=jev.ok_or_else(||RpcError::new(-32020,"Jev计划执行组合根不可用"))?;
                let (computer,targets)=(computer.ok_or_else(||RpcError::new(-32020,"CUA Runtime不可用"))?,targets.ok_or_else(||RpcError::new(-32020,"桌面目标解析不可用"))?);
                let (task,disposition)=crate::plan_fragment::execute_available(store,admission,computer,targets,config,jev,self.auth,&params.task_id,&params.plan_id,params.plan_version,yonder_protocol::sequence(&params.expected_sequence)?,now_ms,host_session_id).map_err(query::error)?;
                let handoff_reason = plan_handoff_reason(disposition).map(str::to_owned);
                Ok(QueryResult::Plan { task_id:task.id,plan_id:params.plan_id,plan_version:params.plan_version,sequence:task.sequence.to_string(),disposition:disposition.into(), handoff_reason })
            },
            Request::FileGrants { params, .. } => {
                if !self.negotiated {
                    return Err(RpcError::new(-32002, "请先完成Gateway握手"));
                }
                if !self.can_file_grants {
                    return Err(RpcError::new(-32010, "文件授权读取需要协议1.27及本机文件授权入口"));
                }
                let task = crate::get(store, &params.task_id).map_err(query::error)?;
                if !self.auth.can_read(&task) {
                    return Err(query::error(crate::Error::NotFound));
                }
                let grants = file_grants
                    .ok_or_else(|| RpcError::new(-32020, "本机文件授权入口不可用"))?
                    .list_for_task(self.auth, &task, now_ms)
                    .map_err(file_grant_error)?
                    .into_iter()
                    .map(protocol_file_grant_summary)
                    .collect();
                Ok(QueryResult::FileGrants {
                    task_id: task.id,
                    grants,
                })
            }
            Request::FileExecute { params, .. } => {
                if !self.negotiated { return Err(RpcError::new(-32002, "请先完成Gateway握手")); }
                if !self.can_file_execute { return Err(RpcError::new(-32010, "文件执行需要协议1.28及macOS文件运行时")); }
                let operation = match params.operation {
                    ProtocolFileOperation::Read => FileOperation::Read,
                    ProtocolFileOperation::CreateNew => FileOperation::CreateNew,
                    ProtocolFileOperation::Replace => FileOperation::Replace,
                    ProtocolFileOperation::Trash => FileOperation::Trash,
                };
                use base64::{Engine as _, engine::general_purpose::STANDARD};
                let body = params.data_base64.as_deref().map(|value| STANDARD.decode(value).map_err(|_| RpcError::new(-32602, "非法文件正文编码"))).transpose()?;
                let result = file_execution::execute_agent_file(
                    store, admission,
                    file_grants.ok_or_else(|| RpcError::new(-32020, "本机文件授权入口不可用"))?,
                    file_port.ok_or_else(|| RpcError::new(-32020, "macOS文件运行时不可用"))?,
                    self.auth, &params.task_id, yonder_protocol::sequence(&params.expected_sequence)?,
                    &params.grant_id, operation, body.as_deref(), host_session_id, now_ms,
                ).map_err(query::error)?;
                let data_base64 = result.read_bytes.as_deref().map(|value| STANDARD.encode(value));
                Ok(QueryResult::FileExecution { execution: ProtocolFileExecutionResult {
                    task: snapshot(result.task), attempt_result: attempt_result(result.attempt_result),
                    data_base64, sha256: result.sha256, bytes_written: result.bytes_written,
                }})
            }
            Request::DocumentExecute { params, .. } => {
                if !self.negotiated { return Err(RpcError::new(-32002, "请先完成Gateway握手")); }
                if !self.can_document_execute { return Err(RpcError::new(-32010, "文档执行需要协议1.29及macOS文档运行时")); }
                let result = document_execution::execute_agent_save_as(
                    store, admission,
                    file_grants.ok_or_else(|| RpcError::new(-32020, "本机文件授权入口不可用"))?,
                    file_port.ok_or_else(|| RpcError::new(-32020, "macOS文件运行时不可用"))?,
                    documents.ok_or_else(|| RpcError::new(-32020, "macOS文档运行时不可用"))?,
                    self.auth, &params.task_id, yonder_protocol::sequence(&params.expected_sequence)?,
                    &params.source_grant_id, &params.output_grant_id, &params.expected_hash,
                    &params.before, &params.after, host_session_id, now_ms,
                ).map_err(query::error)?;
                let (format, sha256, bytes_written) = match result.receipt {
                    Some(receipt) => (Some(match receipt.format { crate::document::DocumentFormat::Docx=>ProtocolDocumentFormat::Docx,crate::document::DocumentFormat::Xlsx=>ProtocolDocumentFormat::Xlsx,crate::document::DocumentFormat::Pptx=>ProtocolDocumentFormat::Pptx }),Some(receipt.sha256),Some(receipt.bytes_written)),
                    None => (None,None,None),
                };
                Ok(QueryResult::DocumentExecution { execution: ProtocolDocumentExecutionResult {
                    task:snapshot(result.task), attempt_result:attempt_result(result.attempt_result), format, sha256, bytes_written,
                }})
            }
            Request::CommandPropose { params, .. } => {
                if !self.negotiated || !self.can_command_propose { return Err(RpcError::new(-32010, "命令提议需要协议1.30及本机批准入口")); }
                let task=crate::get(store,&params.task_id).map_err(query::error)?;
                let approval=command_approvals.ok_or_else(||RpcError::new(-32020,"本机命令批准入口不可用"))?.propose(self.auth,&task,crate::command::CommandRequest{program:params.program,args:params.args,cwd:params.cwd,env:params.env,timeout_ms:params.timeout_ms},now_ms).map_err(|_|RpcError::new(-32003,"命令提议被拒绝"))?;
                Ok(QueryResult::CommandApproval{approval:yonder_protocol::CommandApprovalSummary{command_id:approval.command_id,state:"awaiting_user".into(),expires_at_ms:approval.expires_at_ms}})
            }
            Request::CommandExecute { params, .. } => {
                if !self.negotiated || !self.can_command_execute { return Err(RpcError::new(-32010, "命令执行需要协议1.30、本机批准及macOS命令运行时")); }
                let result=command_execution::execute_agent_command(
                    store,admission,
                    command_approvals.ok_or_else(||RpcError::new(-32020,"本机命令批准入口不可用"))?,
                    command_port.ok_or_else(||RpcError::new(-32020,"macOS命令运行时不可用"))?,
                    &NeverCancel,self.auth,&params.task_id,yonder_protocol::sequence(&params.expected_sequence)?,
                    &params.command_id,host_session_id,now_ms,
                ).map_err(query::error)?;
                use base64::{Engine as _,engine::general_purpose::STANDARD};
                let (outcome,exit_code)=match result.execution.outcome {
                    CommandOutcome::Exited{exit_code}=>(ProtocolCommandExecutionOutcome::Exited,Some(exit_code)),
                    CommandOutcome::TimedOut=>(ProtocolCommandExecutionOutcome::TimedOut,None),
                    CommandOutcome::Cancelled=>(ProtocolCommandExecutionOutcome::Cancelled,None),
                    CommandOutcome::OutputLimitExceeded=>(ProtocolCommandExecutionOutcome::OutputLimitExceeded,None),
                    CommandOutcome::Unknown=>(ProtocolCommandExecutionOutcome::Unknown,None),
                };
                Ok(QueryResult::CommandExecution{execution:ProtocolCommandExecutionResult{
                    task:snapshot(result.task),attempt_result:attempt_result(result.attempt_result),outcome,exit_code,
                    stdout_base64:STANDARD.encode(result.execution.stdout),stderr_base64:STANDARD.encode(result.execution.stderr),
                    stdout_truncated:result.execution.stdout_truncated,stderr_truncated:result.execution.stderr_truncated,
                }})
            }
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
            can_audit: false,
            can_observation_history: false,
            can_control_history: false,
            can_focus_history: false,
            can_creation_history: false,
            can_attempt_start_history: false,
            can_artifact_items: false,
            can_file_grants: false,
            can_file_execute: false,
            can_document_execute: false,
            can_command_propose: false,
            can_command_execute: false,
            can_plan_submit: false,
            can_plan_execute: false,
            browser_available: false,
            can_computer: false,
            can_computer_step: false,
            can_complete: false,
            can_fail: false,
            computer_available: false,
            computer_permission_required: false,
            file_grants_available: false,
            file_execution_available: false,
            document_execution_available: false,
            command_approval_available: false,
            command_execution_available: false,
            last_create_was_new: false,
        }
    }

    pub fn handle(&mut self, store: &mut impl TaskStore, bytes: &[u8], now_ms: u64) -> Response {
        self.last_create_was_new = false;
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
                crate::register_with_outcome(
                    store,
                    self.auth,
                    &params.idempotency_key,
                    &params.description,
                    params.name.as_deref(),
                    crate::TaskSource::LocalAgent,
                )
                .map_err(query::error)
                .map(|outcome| {
                    self.last_create_was_new = outcome.created;
                    let mut snapshot = query::summary(outcome.task);
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
        if matches!(request, Request::Artifacts { .. })
            && self.negotiated
            && !self.can_artifact_items
        {
            return Response::Failure {
                jsonrpc: Version::V2,
                id: Some(id),
                error: RpcError::new(-32010, "产物清单读取需要协议1.26及审计存储能力"),
            };
        }
        if matches!(request, Request::FileGrants { .. }) {
            return Response::Failure {
                jsonrpc: Version::V2,
                id: Some(id),
                error: RpcError::new(-32020, "本机文件授权入口不可用"),
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
                self.can_audit,
                self.can_observation_history,
                self.can_control_history,
                self.can_focus_history,
                self.can_creation_history,
                self.can_attempt_start_history,
                self.can_artifact_items,
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
            if !self.can_audit {
                if let Response::Success { result, .. } = &mut response {
                    match result {
                        QueryResult::Snapshot { task } | QueryResult::Step { task, .. } => {
                            task.artifact_manifest = None;
                            task.user_confirmation = None;
                        }
                        QueryResult::Events { events, .. } => {
                            for event in events {
                                event.artifact_manifest = None;
                                event.user_confirmation = None;
                            }
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
                self.can_browser = self.can_advance && params.protocol_version.minor >= 8 && self.browser_available && store.supports_browser_references();
                self.can_browser_read = self.negotiated && params.protocol_version.minor >= 15 && store.supports_browser_references();
                self.can_running_filter = self.negotiated && params.protocol_version.minor >= 16 && store.supports_running_filter();
                self.can_wait_for_user = self.can_advance && params.protocol_version.minor >= 17 && store.supports_wait_for_user();
                self.can_presentation = self.negotiated && params.protocol_version.minor >= 19 && store.supports_presentation();
                self.can_audit = self.negotiated && params.protocol_version.minor >= 20 && store.supports_audit();
                self.can_audit = self.negotiated && params.protocol_version.minor >= 20 && store.supports_audit();
                self.can_observation_history = self.can_presentation && params.protocol_version.minor >= 21;
                self.can_control_history = self.can_observation_history && params.protocol_version.minor >= 22;
                self.can_focus_history = self.can_control_history && params.protocol_version.minor >= 23;
                self.can_creation_history = self.can_focus_history && params.protocol_version.minor >= 24;
                self.can_attempt_start_history = self.can_creation_history && params.protocol_version.minor >= 25;
                self.can_artifact_items = self.can_audit && params.protocol_version.minor >= 26;
                self.can_file_grants = self.negotiated
                    && params.protocol_version.minor >= 27
                    && self.file_grants_available;
                self.can_file_execute = self.negotiated
                    && params.protocol_version.minor >= 28
                    && self.file_execution_available;
                self.can_document_execute = self.negotiated
                    && params.protocol_version.minor >= 29
                    && self.document_execution_available;
                self.can_command_propose=self.negotiated&&params.protocol_version.minor>=30&&self.command_approval_available&&matches!(self.platform,Platform::Macos);
                self.can_command_execute=self.negotiated&&params.protocol_version.minor>=30&&self.command_execution_available&&matches!(self.platform,Platform::Macos);
                self.can_computer=self.can_advance&&params.protocol_version.minor>=11&&self.computer_available&&!self.computer_permission_required;
                self.can_computer_step=self.can_computer&&params.protocol_version.minor>=12;
                self.can_plan_submit=self.negotiated&&params.protocol_version.minor>=31&&store.supports_plan_fragments();
                self.can_plan_execute=self.can_plan_submit&&self.can_computer&&matches!(self.platform,Platform::Macos);
                self.can_complete=self.can_computer&&params.protocol_version.minor>=10;
                self.can_fail=self.can_complete&&params.protocol_version.minor>=18;
                if !self.negotiated { return Err(RpcError::new(-32010, "协议主版本不兼容")); }
                let mut capabilities = vec![CapabilityInfo { name: Capability::TaskRead, version: ProtocolVersion { major: 1, minor: if self.can_plan_submit {31} else if self.can_document_execute {29} else if self.can_file_execute {28} else if self.can_file_grants {27} else if self.can_artifact_items {26} else if self.can_attempt_start_history {25} else if self.can_creation_history {24} else if self.can_focus_history {23} else if self.can_control_history {22} else if self.can_observation_history {21} else if self.can_audit {20} else if self.can_presentation {19} else if self.can_fail {18} else if self.can_wait_for_user {17} else if self.can_running_filter {16} else if self.can_browser_read {15} else if self.can_focus {13} else if self.can_computer_step {12} else if self.can_computer {11} else if self.can_browser { 8 } else if self.can_advance { 7 } else if self.can_controls { 6 } else if self.can_attempt_results { 5 } else if self.can_steps { 4 } else if self.can_name { 3 } else { 0 } }, availability: Availability::Available, reason: None }];
                let version = ProtocolVersion { major: 1, minor: if self.can_plan_submit {31} else if self.can_command_propose||self.can_command_execute {30} else if self.can_document_execute {29} else if self.can_file_execute {28} else if self.can_file_grants {27} else if self.can_artifact_items {26} else if self.can_attempt_start_history {25} else if self.can_creation_history {24} else if self.can_focus_history {23} else if self.can_control_history {22} else if self.can_observation_history {21} else if self.can_audit {20} else if params.offered_capabilities.as_deref().is_some_and(|value|value.contains(&yonder_protocol::OfferedCapability::UserInputAttachment)) || self.can_presentation {19} else if self.can_fail {18} else if self.can_wait_for_user {17} else if self.can_running_filter {16} else if self.can_browser_read {15} else if params.offered_capabilities.as_deref().is_some_and(|value|value.contains(&yonder_protocol::OfferedCapability::UserInput)) {14} else if self.can_focus {13} else if self.can_computer_step {12} else if self.can_computer {11} else if self.can_browser { 8 } else if self.can_advance { 7 } else if self.can_controls { 6 } else if self.can_attempt_results { 5 } else if self.can_steps { 4 } else if self.can_name { 3 } else if self.can_cancel { 2 } else { u16::from(self.can_create) } };
                if self.can_create { capabilities.push(CapabilityInfo { name: Capability::TaskCreate, version: ProtocolVersion { major: 1, minor: if self.can_name { 3 } else { 1 } }, availability: Availability::Available, reason: None }); }
                if self.can_cancel { capabilities.push(CapabilityInfo { name: Capability::TaskCancel, version: ProtocolVersion { major: 1, minor: 2 }, availability: Availability::Available, reason: Some("仅支持未开始任务取消".into()) }); }
                if self.can_steps { capabilities.push(CapabilityInfo { name: Capability::TaskStepDeclare, version: ProtocolVersion { major: 1, minor: 4 }, availability: Availability::Available, reason: Some("仅支持created任务声明".into()) }); }
                if self.can_controls { capabilities.push(CapabilityInfo { name: Capability::TaskControl, version: ProtocolVersion { major: 1, minor: if self.can_focus{13}else{6} }, availability: Availability::Available, reason: Some("登记停止请求，执行器在步骤边界确认".into()) }); }
                if self.can_advance { capabilities.push(CapabilityInfo { name: Capability::TaskStepAdvance, version: ProtocolVersion { major: 1, minor: 7 }, availability: Availability::Available, reason: Some("仅推进已完成Observe的步骤".into()) }); }
                if params.protocol_version.minor >= 8 && store.supports_browser_references() { capabilities.push(CapabilityInfo { name: Capability::BrowserExecute, version: ProtocolVersion { major: 1, minor: 8 }, availability: if self.can_browser { Availability::Available } else { Availability::DependencyMissing }, reason: (!self.can_browser).then(|| "ego-lite Bridge不可用".into()) }); }
                if params.protocol_version.minor>=11&&store.supports_execution_attempts() {capabilities.push(CapabilityInfo{name:Capability::ComputerExecute,version:ProtocolVersion{major:1,minor:if params.protocol_version.minor>=12{12}else{11}},availability:if self.can_computer{Availability::Available}else if self.computer_permission_required{Availability::PermissionRequired}else{Availability::DependencyMissing},reason:(!self.can_computer).then(||if self.computer_permission_required{"Yonda需要macOS辅助功能权限".into()}else{"CUA Runtime不可用".into()})});}
                if self.can_complete{capabilities.push(CapabilityInfo{name:Capability::TaskComplete,version:ProtocolVersion{major:1,minor:10},availability:Availability::Available,reason:Some("仅完成已Observe并推进边界的CUA任务".into())});}
                if self.can_fail{capabilities.push(CapabilityInfo{name:Capability::TaskFail,version:ProtocolVersion{major:1,minor:18},availability:Availability::Available,reason:Some("仅终结已Observe失败并推进边界的CUA任务".into())});}
                if self.can_wait_for_user{capabilities.push(CapabilityInfo{name:Capability::TaskWaitForUser,version:ProtocolVersion{major:1,minor:17},availability:Availability::Available,reason:Some("仅在已Observe并推进的步骤边界等待".into())});}
                if params.protocol_version.minor >= 27 && self.file_grants_available { capabilities.push(CapabilityInfo{name:Capability::FileGrantRead,version:ProtocolVersion{major:1,minor:27},availability:Availability::Available,reason:None}); }
                if params.protocol_version.minor >= 28 && self.file_execution_available { capabilities.push(CapabilityInfo{name:Capability::FileExecute,version:ProtocolVersion{major:1,minor:28},availability:Availability::Available,reason:None}); }
                if params.protocol_version.minor >= 29 && self.document_execution_available { capabilities.push(CapabilityInfo{name:Capability::DocumentExecute,version:ProtocolVersion{major:1,minor:29},availability:Availability::Available,reason:None}); }
                if params.protocol_version.minor>=30 && self.command_approval_available && matches!(self.platform,Platform::Macos) { capabilities.push(CapabilityInfo{name:Capability::CommandPropose,version:ProtocolVersion{major:1,minor:30},availability:if self.can_command_propose{Availability::Available}else{Availability::TemporarilyUnavailable},reason:(!self.can_command_propose).then(||"本机命令批准入口不可用".into())}); }
                if params.protocol_version.minor>=30 && self.command_approval_available && matches!(self.platform,Platform::Macos) { capabilities.push(CapabilityInfo{name:Capability::CommandExecute,version:ProtocolVersion{major:1,minor:30},availability:if self.can_command_execute{Availability::Available}else{Availability::DependencyMissing},reason:(!self.can_command_execute).then(||"macOS命令运行时不可用".into())}); }
                if params.protocol_version.minor>=31 && store.supports_plan_fragments() { capabilities.push(CapabilityInfo{name:Capability::TaskPlanSubmit,version:ProtocolVersion{major:1,minor:31},availability:if self.can_plan_submit{Availability::Available}else{Availability::DependencyMissing},reason:(!self.can_plan_submit).then(||"计划片段存储不可用".into())}); }
                if params.protocol_version.minor>=31 && store.supports_plan_fragments() { capabilities.push(CapabilityInfo{name:Capability::TaskPlanExecute,version:ProtocolVersion{major:1,minor:31},availability:if self.can_plan_execute{Availability::Available}else{Availability::DependencyMissing},reason:(!self.can_plan_execute).then(||"macOS CUA运行时或辅助功能权限不可用".into())}); }
                return Ok(QueryResult::Hello { protocol_version: version, platform: self.platform, capabilities });
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

fn plan_handoff_reason(disposition: &str) -> Option<&'static str> {
    match disposition {
        "handback" => Some("需要慢脑重新 Observe 或规划"),
        "slow-brain-required" => Some("需要慢脑安排本机发送确认"),
        _ => None,
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

fn protocol_file_grant_summary(
    grant: crate::file_authorization::FileGrantSummary,
) -> ProtocolFileGrantSummary {
    ProtocolFileGrantSummary {
        grant_id: grant.grant_id,
        purpose: match grant.purpose {
            FileGrantPurpose::Read => ProtocolFileGrantPurpose::Read,
            FileGrantPurpose::CreateNew => ProtocolFileGrantPurpose::CreateNew,
            FileGrantPurpose::Replace => ProtocolFileGrantPurpose::Replace,
            FileGrantPurpose::Trash => ProtocolFileGrantPurpose::Trash,
        },
        expires_at_ms: grant.expires_at_ms,
    }
}

fn file_grant_error(error: crate::file_authorization::FileGrantError) -> RpcError {
    match error {
        crate::file_authorization::FileGrantError::PermissionDenied => {
            RpcError::new(-32003, "文件授权不属于当前任务或Agent")
        }
        crate::file_authorization::FileGrantError::NotFound => {
            RpcError::new(-32004, "文件授权不存在")
        }
        crate::file_authorization::FileGrantError::Expired => {
            RpcError::new(-32005, "文件授权已过期")
        }
        crate::file_authorization::FileGrantError::Unavailable => {
            RpcError::new(-32020, "本机文件授权入口不可用")
        }
        _ => RpcError::new(-32603, "文件授权查询失败"),
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
