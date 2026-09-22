//! Gateway 协议唯一来源；此模块不提供传输或身份认证。
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const MAX_REQUEST_BYTES: usize = 64 * 1024;
pub const MAX_AGENT_ATTACHMENT_BYTES: u32 = 4 * 1024 * 1024;
pub const MAX_AGENT_ATTACHMENT_CHUNK_BASE64_BYTES: usize = 64_256;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
pub enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
pub enum Capability {
    #[serde(rename = "task.read")]
    TaskRead,
    #[serde(rename = "task.create")]
    TaskCreate,
    #[serde(rename = "task.cancel")]
    TaskCancel,
    #[serde(rename = "task.complete")]
    TaskComplete,
    #[serde(rename = "task.fail")]
    TaskFail,
    #[serde(rename = "task.control")]
    TaskControl,
    #[serde(rename = "task.wait-for-user")]
    TaskWaitForUser,
    #[serde(rename = "task.step.declare")]
    TaskStepDeclare,
    #[serde(rename = "task.step.advance")]
    TaskStepAdvance,
    #[serde(rename = "browser.execute")]
    BrowserExecute,
    #[serde(rename = "computer.execute")]
    ComputerExecute,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum Platform {
    Windows,
    Macos,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Available,
    PermissionRequired,
    DependencyMissing,
    TemporarilyUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
pub enum OfferedCapability {
    #[serde(rename = "user_input")]
    UserInput,
    #[serde(rename = "user_input_attachment")]
    UserInputAttachment,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct CapabilityInfo {
    pub name: Capability,
    pub version: ProtocolVersion,
    pub availability: Availability,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct HelloParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub protocol_version: ProtocolVersion,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub offered_capabilities: Option<Vec<OfferedCapability>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "snake_case")]
pub enum AgentInputSource {
    Voice,
    Selection,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
pub enum AgentAttachmentMime {
    #[serde(rename = "image/png")] ImagePng,
    #[serde(rename = "image/jpeg")] ImageJpeg,
    #[serde(rename = "image/webp")] ImageWebp,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct AgentAttachmentBeginParams {
    pub attachment_id: String,
    pub session_id: String,
    pub mime: AgentAttachmentMime,
    pub byte_length: u32,
    pub sha256: String,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
}

impl AgentAttachmentBeginParams {
    pub fn validate(&self, now_ms: u64) -> Result<(), RpcError> {
        if !valid_id(&self.attachment_id) || !valid_id(&self.session_id)
            || !(1..=MAX_AGENT_ATTACHMENT_BYTES).contains(&self.byte_length)
            || self.sha256.len() != 64 || !self.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            || self.deadline <= now_ms || self.deadline > 9_007_199_254_740_991 {
            return Err(RpcError::new(-32602, "非法Agent附件声明"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct AgentAttachmentChunkParams {
    pub attachment_id: String,
    pub session_id: String,
    pub sequence: u16,
    pub data_base64: String,
}

impl AgentAttachmentChunkParams {
    pub fn validate(&self) -> Result<(), RpcError> {
        let bytes = self.data_base64.as_bytes();
        if !valid_id(&self.attachment_id) || !valid_id(&self.session_id) || bytes.is_empty()
            || bytes.len() > MAX_AGENT_ATTACHMENT_CHUNK_BASE64_BYTES || bytes.len() % 4 != 0
            || !bytes.iter().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
            || bytes.iter().position(|byte| *byte == b'=').is_some_and(|start| start < bytes.len().saturating_sub(2) || !bytes[start..].iter().all(|byte| *byte == b'=')) {
            return Err(RpcError::new(-32602, "非法Agent附件分块"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct AgentAttachmentFinishParams { pub attachment_id: String, pub session_id: String }

impl AgentAttachmentFinishParams {
    pub fn validate(&self) -> Result<(), RpcError> {
        if !valid_id(&self.attachment_id) || !valid_id(&self.session_id) {
            return Err(RpcError::new(-32602, "非法Agent附件完成请求"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct AgentInputParams {
    pub input_id: String,
    pub session_id: String,
    pub source: AgentInputSource,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub attachment_id: Option<String>,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub created_at: u64,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
}

impl AgentInputParams {
    pub fn validate(&self, now_ms: u64) -> Result<(), RpcError> {
        if !valid_id(&self.input_id)
            || !valid_id(&self.session_id)
            || self.content.trim().is_empty()
            || self.content.as_bytes().len() > 16 * 1024
            || self.attachment_id.as_deref().is_some_and(|value| !valid_id(value))
            || self.created_at > self.deadline
            || self.deadline > 9_007_199_254_740_991
            || self.deadline <= now_ms
            || self.deadline.saturating_sub(self.created_at) > 60_000
        {
            return Err(RpcError::new(-32602, "非法Agent输入"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "method", deny_unknown_fields)]
pub enum AgentRequest {
    #[serde(rename = "agent.attachment.begin")]
    AttachmentBegin { jsonrpc: Version, params: AgentAttachmentBeginParams },
    #[serde(rename = "agent.attachment.chunk")]
    AttachmentChunk { jsonrpc: Version, params: AgentAttachmentChunkParams },
    #[serde(rename = "agent.attachment.finish")]
    AttachmentFinish { jsonrpc: Version, #[serde(rename = "id")] request_id: String, params: AgentAttachmentFinishParams },
    #[serde(rename = "agent.input")]
    Input {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: AgentInputParams,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct AgentInputResult {
    pub accepted: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged, deny_unknown_fields)]
pub enum AgentResponse {
    Success {
        jsonrpc: Version,
        id: String,
        result: AgentInputResult,
    },
    Failure {
        jsonrpc: Version,
        id: Option<String>,
        error: RpcError,
    },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct GetParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub task_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct EventsParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub task_id: String,
    #[schemars(regex(pattern = "^(0|[1-9][0-9]{0,18})$"))]
    pub after_sequence: String,
    #[schemars(range(min = 1, max = 100))]
    pub limit: u8,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct CreateParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub idempotency_key: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct CancelParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub task_id: String,
    pub expected_sequence: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct WaitForUserParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub task_id: String,
    pub expected_sequence: String,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum ControlKind {
    Pause,
    Cancel,
    Takeover,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct ControlParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub task_id: String,
    pub expected_sequence: String,
    pub kind: ControlKind,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct StepDeclareParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub task_id: String,
    pub expected_sequence: String,
    pub step_id: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct StepAdvanceParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub task_id: String,
    pub expected_sequence: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum BrowserOperation {
    Create,
    Observe,
    HandOff,
    TakeOver,
    Finish,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct BrowserExecuteParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub task_id: String,
    pub expected_sequence: String,
    pub operation: BrowserOperation,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct ComputerExecuteParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub task_id: String,
    pub expected_sequence: String,
    pub tool_name: String,
    #[ts(type = "Record<string, unknown>")]
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct ComputerStepParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub task_id: String,
    pub expected_sequence: String,
    pub step_id: String,
    pub label: String,
    pub tool_name: String,
    #[ts(type = "Record<string, unknown>")]
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct CompleteParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    pub task_id: String,
    pub expected_sequence: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "method", deny_unknown_fields)]
pub enum Request {
    #[serde(rename = "browser.execute")]
    BrowserExecute {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: BrowserExecuteParams,
    },
    #[serde(rename = "computer.execute")]
    ComputerExecute {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: ComputerExecuteParams,
    },
    #[serde(rename = "computer.step")]
    ComputerStep {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: ComputerStepParams,
    },
    #[serde(rename = "task.cancel")]
    Cancel {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: CancelParams,
    },
    #[serde(rename = "task.create")]
    Create {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: CreateParams,
    },
    #[serde(rename = "task.control")]
    Control {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: ControlParams,
    },
    #[serde(rename = "task.complete")]
    Complete {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: CompleteParams,
    },
    #[serde(rename = "task.fail")]
    Fail {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: CompleteParams,
    },
    #[serde(rename = "task.wait_for_user")]
    WaitForUser {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: WaitForUserParams,
    },
    #[serde(rename = "gateway.hello")]
    Hello {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: HelloParams,
    },
    #[serde(rename = "task.list")]
    List {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: ListParams,
    },
    #[serde(rename = "task.get")]
    Get {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: GetParams,
    },
    #[serde(rename = "task.events")]
    Events {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: EventsParams,
    },
    #[serde(rename = "task.step.declare")]
    StepDeclare {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: StepDeclareParams,
    },
    #[serde(rename = "task.step.get")]
    StepGet {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: GetParams,
    },
    #[serde(rename = "task.browser.get")]
    BrowserGet {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: GetParams,
    },
    #[serde(rename = "task.step.advance")]
    StepAdvance {
        jsonrpc: Version,
        #[serde(rename = "id")]
        request_id: String,
        params: StepAdvanceParams,
    },
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct ListParams {
    pub agent_id: String,
    pub capability: Capability,
    #[ts(type = "number")]
    #[schemars(range(min = 0, max = 9007199254740991_u64))]
    pub deadline: u64,
    #[ts(optional = nullable)]
    pub after_task_id: Option<String>,
    #[serde(default)]
    #[ts(as = "Option<bool>", optional)]
    pub include_finished: bool,
    #[serde(default)]
    #[ts(as = "Option<bool>", optional)]
    pub running_only: bool,
    #[schemars(range(min = 1, max = 100))]
    pub limit: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Created,
    Running,
    WaitingForUser,
    Paused,
    Interrupted,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum TaskSource {
    LocalAgent,
    CloudAgent,
    Legacy,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum TaskObservationResult {
    Matched,
    NotMatched,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct TaskObservation {
    pub step_id: String,
    pub result: TaskObservationResult,
    pub summary: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct TaskSnapshot {
    pub task_id: String,
    pub owner_agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub name: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub source: Option<TaskSource>,
    pub status: TaskStatus,
    pub sequence: String,
    #[serde(default)]
    #[ts(optional)]
    pub current_step: Option<StepDeclaration>,
    #[serde(default)]
    #[ts(optional)]
    pub observation: Option<TaskObservation>,
    #[serde(default)]
    #[ts(optional)]
    pub next_intent: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct TaskEvent {
    pub previous: TaskStatus,
    pub status: TaskStatus,
    pub sequence: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub step_declaration: Option<StepDeclaration>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub attempt_result: Option<AttemptResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub wait_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct StepDeclaration {
    pub step_id: String,
    pub label: String,
    pub accepted_sequence: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum AttemptResultPhase {
    Observed,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum AttemptUnknownReason {
    InvalidInput,
    DependencyUnavailable,
    WorkerFailed,
    TimedOut,
    InvalidResponse,
    IdentityMismatch,
    ObserveFailed,
    UserInput,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct AttemptResult {
    pub step_id: String,
    pub attempt_id: String,
    pub worker_instance_id: String,
    pub host_session_id: String,
    pub phase: AttemptResultPhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub action_succeeded: Option<bool>,
    pub observe_valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub unknown_reason: Option<AttemptUnknownReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum ControlPhase {
    Pending,
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum FocusPhase {
    Locating,
    Focused,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum FocusFailure {
    PermissionUnavailable,
    ProcessChanged,
    WindowMissing,
    MappingNotUnique,
    ActivationFailed,
    VerificationFailed,
    GeometryChanged,
    ReferenceUnavailable,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct ControlRecord {
    pub attempt_id: String,
    pub control_id: String,
    pub kind: ControlKind,
    pub phase: ControlPhase,
    pub accepted_sequence: String,
    pub stopped_sequence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub focus_phase: Option<FocusPhase>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub focus_failure: Option<FocusFailure>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct BrowserReference {
    pub external_task_ref: String,
    pub ownership: String,
    pub managed_pages: u16,
    pub finished: bool,
    pub updated_sequence: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct ComputerObservation {
    pub element_count: u16,
    pub screenshot_path: Option<String>,
    pub screenshot_mime: Option<String>,
    pub target_visible: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum QueryResult {
    Hello {
        protocol_version: ProtocolVersion,
        platform: Platform,
        capabilities: Vec<CapabilityInfo>,
    },
    Tasks {
        tasks: Vec<TaskSnapshot>,
        next_after_task_id: Option<String>,
    },
    Snapshot {
        task: TaskSnapshot,
    },
    Events {
        task_id: String,
        events: Vec<TaskEvent>,
    },
    Step {
        task: TaskSnapshot,
        step: Option<StepDeclaration>,
    },
    BrowserState {
        task: TaskSnapshot,
        reference: Option<BrowserReference>,
    },
    Control {
        task: TaskSnapshot,
        control: ControlRecord,
    },
    Browser {
        task: TaskSnapshot,
        reference: BrowserReference,
    },
    Computer {
        task: TaskSnapshot,
        attempt_result: AttemptResult,
    },
    ComputerStep {
        task_id: String,
        status: TaskStatus,
        sequence: String,
        action_succeeded: Option<bool>,
        unknown_reason: Option<AttemptUnknownReason>,
        observation: Option<ComputerObservation>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

impl RpcError {
    pub fn new(code: i32, message: &str) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged, deny_unknown_fields)]
pub enum Response {
    Success {
        jsonrpc: Version,
        id: String,
        result: QueryResult,
    },
    Failure {
        jsonrpc: Version,
        id: Option<String>,
        error: RpcError,
    },
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
}

pub fn sequence(value: &str) -> Result<u64, RpcError> {
    if value.is_empty()
        || value.len() > 19
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|c| c.is_ascii_digit())
    {
        return Err(RpcError::new(-32602, "非法任务序号"));
    }
    value
        .parse::<u64>()
        .ok()
        .filter(|n| *n <= i64::MAX as u64)
        .ok_or_else(|| RpcError::new(-32602, "非法任务序号"))
}

impl Request {
    pub fn agent_id(&self) -> &str {
        match self {
            Self::WaitForUser { params, .. } => &params.agent_id,
            Self::ComputerStep { params, .. } => &params.agent_id,
            Self::ComputerExecute { params, .. } => &params.agent_id,
            Self::Complete { params, .. } | Self::Fail { params, .. } => &params.agent_id,
            Self::BrowserExecute { params, .. } => &params.agent_id,
            Self::StepAdvance { params, .. } => &params.agent_id,
            Self::Cancel { params, .. } => &params.agent_id,
            Self::Control { params, .. } => &params.agent_id,
            Self::Create { params, .. } => &params.agent_id,
            Self::Get { params, .. }
            | Self::StepGet { params, .. }
            | Self::BrowserGet { params, .. } => &params.agent_id,
            Self::Events { params, .. } => &params.agent_id,
            Self::List { params, .. } => &params.agent_id,
            Self::Hello { params, .. } => &params.agent_id,
            Self::StepDeclare { params, .. } => &params.agent_id,
        }
    }
    pub fn request_id(&self) -> &str {
        match self {
            Self::WaitForUser { request_id, .. }
            | Self::ComputerStep { request_id, .. }
            | Self::ComputerExecute { request_id, .. }
            | Self::Complete { request_id, .. }
            | Self::Fail { request_id, .. }
            | Self::BrowserExecute { request_id, .. }
            | Self::StepAdvance { request_id, .. }
            | Self::Cancel { request_id, .. }
            | Self::Control { request_id, .. }
            | Self::Create { request_id, .. }
            | Self::Get { request_id, .. }
            | Self::Events { request_id, .. }
            | Self::List { request_id, .. }
            | Self::Hello { request_id, .. }
            | Self::StepDeclare { request_id, .. }
            | Self::StepGet { request_id, .. }
            | Self::BrowserGet { request_id, .. } => request_id,
        }
    }

    pub fn validate(&self, now_ms: u64) -> Result<(), RpcError> {
        let capability = match self {
            Self::WaitForUser { params, .. } => params.capability,
            Self::ComputerStep { params, .. } => params.capability,
            Self::ComputerExecute { params, .. } => params.capability,
            Self::Complete { params, .. } => params.capability,
            Self::Fail { params, .. } => params.capability,
            Self::BrowserExecute { params, .. } => params.capability,
            Self::StepAdvance { params, .. } => params.capability,
            Self::Cancel { params, .. } => params.capability,
            Self::Control { params, .. } => params.capability,
            Self::Create { params, .. } => params.capability,
            Self::Hello { params, .. } => params.capability,
            Self::List { params, .. } => params.capability,
            Self::Get { params, .. } => params.capability,
            Self::Events { params, .. } => params.capability,
            Self::StepDeclare { params, .. } => params.capability,
            Self::StepGet { params, .. } => params.capability,
            Self::BrowserGet { params, .. } => params.capability,
        };
        if capability
            != if matches!(self, Self::WaitForUser { .. }) {
                Capability::TaskWaitForUser
            } else if matches!(
                self,
                Self::ComputerExecute { .. } | Self::ComputerStep { .. }
            ) {
                Capability::ComputerExecute
            } else if matches!(self, Self::Complete { .. }) {
                Capability::TaskComplete
            } else if matches!(self, Self::Fail { .. }) {
                Capability::TaskFail
            } else if matches!(self, Self::BrowserExecute { .. }) {
                Capability::BrowserExecute
            } else if matches!(self, Self::StepAdvance { .. }) {
                Capability::TaskStepAdvance
            } else if matches!(self, Self::Create { .. }) {
                Capability::TaskCreate
            } else if matches!(self, Self::Cancel { .. }) {
                Capability::TaskCancel
            } else if matches!(self, Self::Control { .. }) {
                Capability::TaskControl
            } else if matches!(self, Self::StepDeclare { .. }) {
                Capability::TaskStepDeclare
            } else {
                Capability::TaskRead
            }
        {
            return Err(RpcError::new(-32602, "请求能力不匹配"));
        }
        let (agent_id, task_id, deadline) = match self {
            Self::WaitForUser { params, .. } => {
                if sequence(&params.expected_sequence)? == 0
                    || params.reason.trim().is_empty()
                    || params.reason.trim().as_bytes().len() > 512
                    || params.reason.chars().any(char::is_control)
                {
                    return Err(RpcError::new(-32602, "非法等待原因"));
                }
                (
                    &params.agent_id,
                    Some(params.task_id.as_str()),
                    params.deadline,
                )
            }
            Self::ComputerStep { params, .. } => {
                if sequence(&params.expected_sequence)? == 0
                    || !valid_id(&params.step_id)
                    || !valid_step_label(&params.label)
                    || !valid_sdk_tool_name(&params.tool_name)
                    || !params.arguments.is_object()
                    || !serde_json::to_vec(&params.arguments)
                        .is_ok_and(|value| value.len() <= 16 * 1024)
                    || !safe_sdk_arguments(&params.arguments)
                {
                    return Err(RpcError::new(-32602, "非法桌面步骤参数"));
                }
                (
                    &params.agent_id,
                    Some(params.task_id.as_str()),
                    params.deadline,
                )
            }
            Self::ComputerExecute { params, .. } => {
                if sequence(&params.expected_sequence)? == 0
                    || !valid_sdk_tool_name(&params.tool_name)
                    || !params.arguments.is_object()
                    || !serde_json::to_vec(&params.arguments)
                        .is_ok_and(|value| value.len() <= 16 * 1024)
                    || !safe_sdk_arguments(&params.arguments)
                {
                    return Err(RpcError::new(-32602, "非法桌面工具参数"));
                }
                (
                    &params.agent_id,
                    Some(params.task_id.as_str()),
                    params.deadline,
                )
            }
            Self::Complete { params, .. } => {
                if sequence(&params.expected_sequence)? == 0 {
                    return Err(RpcError::new(-32602, "非法期望序号"));
                }
                (
                    &params.agent_id,
                    Some(params.task_id.as_str()),
                    params.deadline,
                )
            }
            Self::Fail { params, .. } => {
                if sequence(&params.expected_sequence)? == 0 {
                    return Err(RpcError::new(-32602, "非法期望序号"));
                }
                (
                    &params.agent_id,
                    Some(params.task_id.as_str()),
                    params.deadline,
                )
            }
            Self::BrowserExecute { params, .. } => {
                if sequence(&params.expected_sequence)? == 0 {
                    return Err(RpcError::new(-32602, "非法期望序号"));
                }
                (
                    &params.agent_id,
                    Some(params.task_id.as_str()),
                    params.deadline,
                )
            }
            Self::StepAdvance { params, .. } => {
                if sequence(&params.expected_sequence)? == 0 {
                    return Err(RpcError::new(-32602, "非法期望序号"));
                }
                (
                    &params.agent_id,
                    Some(params.task_id.as_str()),
                    params.deadline,
                )
            }
            Self::Cancel { params, .. } => {
                if sequence(&params.expected_sequence)? == 0 {
                    return Err(RpcError::new(-32602, "非法期望序号"));
                }
                (
                    &params.agent_id,
                    Some(params.task_id.as_str()),
                    params.deadline,
                )
            }
            Self::Control { params, .. } => {
                if sequence(&params.expected_sequence)? == 0 {
                    return Err(RpcError::new(-32602, "非法期望序号"));
                }
                (
                    &params.agent_id,
                    Some(params.task_id.as_str()),
                    params.deadline,
                )
            }
            Self::Create { params, .. } => {
                if !valid_id(&params.idempotency_key)
                    || params.description.trim().is_empty()
                    || params.description.len() > 4096
                    || params
                        .name
                        .as_deref()
                        .is_some_and(|name| !valid_task_name(name))
                {
                    return Err(RpcError::new(-32602, "非法任务登记参数"));
                }
                (&params.agent_id, None, params.deadline)
            }
            Self::Hello { params, .. } => {
                let offers_input = params
                    .offered_capabilities
                    .as_deref()
                    .is_some_and(|value| value.contains(&OfferedCapability::UserInput));
                let offers_attachment = params
                    .offered_capabilities
                    .as_deref()
                    .is_some_and(|value| value.contains(&OfferedCapability::UserInputAttachment));
                if params.session_id.is_some() != offers_input
                    || params.session_id.as_deref().is_some_and(|id| !valid_id(id))
                    || offers_input && params.protocol_version.minor < 14
                    || offers_attachment
                        && (!offers_input || params.protocol_version.minor < 19)
                {
                    return Err(RpcError::new(-32602, "非法Agent会话声明"));
                }
                (&params.agent_id, None, params.deadline)
            }
            Self::List { params, .. } => {
                if !(1..=100).contains(&params.limit) {
                    return Err(RpcError::new(-32602, "非法分页上限"));
                }
                (
                    &params.agent_id,
                    params.after_task_id.as_deref(),
                    params.deadline,
                )
            }
            Self::Get { params, .. } => (
                &params.agent_id,
                Some(params.task_id.as_str()),
                params.deadline,
            ),
            Self::Events { params, .. } => {
                sequence(&params.after_sequence)?;
                if !(1..=100).contains(&params.limit) {
                    return Err(RpcError::new(-32602, "非法分页上限"));
                }
                (
                    &params.agent_id,
                    Some(params.task_id.as_str()),
                    params.deadline,
                )
            }
            Self::StepDeclare { params, .. } => {
                if sequence(&params.expected_sequence)? == 0
                    || !valid_id(&params.step_id)
                    || !valid_step_label(&params.label)
                {
                    return Err(RpcError::new(-32602, "非法步骤声明参数"));
                }
                (
                    &params.agent_id,
                    Some(params.task_id.as_str()),
                    params.deadline,
                )
            }
            Self::StepGet { params, .. } => (
                &params.agent_id,
                Some(params.task_id.as_str()),
                params.deadline,
            ),
            Self::BrowserGet { params, .. } => (
                &params.agent_id,
                Some(params.task_id.as_str()),
                params.deadline,
            ),
        };
        if !valid_id(self.request_id())
            || !valid_id(agent_id)
            || task_id.is_some_and(|id| !valid_id(id))
            || deadline > 9_007_199_254_740_991
        {
            return Err(RpcError::new(-32602, "非法请求参数"));
        }
        if deadline <= now_ms {
            return Err(RpcError::new(-32001, "请求已过期"));
        }
        Ok(())
    }
}

/// 传输层也必须有界读帧，不能先无限读取再依赖此处限长。
pub fn decode(bytes: &[u8]) -> Result<Request, RpcError> {
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(RpcError::new(-32600, "请求过大"));
    }
    serde_json::from_slice(bytes).map_err(|error| {
        let code = if error.is_syntax() || error.is_eof() {
            -32700
        } else {
            -32600
        };
        RpcError::new(code, "无效请求")
    })
}

pub fn encode(response: &Response) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(response)
}
pub fn encode_request(request: &Request) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(request)
}
pub fn decode_response(bytes: &[u8]) -> Result<Response, serde_json::Error> {
    serde_json::from_slice(bytes)
}
pub fn encode_agent_request(request: &AgentRequest) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(request)
}
pub fn decode_agent_request(bytes: &[u8]) -> Result<AgentRequest, serde_json::Error> {
    serde_json::from_slice(bytes)
}
pub fn encode_agent_response(response: &AgentResponse) -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(response)
}
pub fn decode_agent_response(bytes: &[u8]) -> Result<AgentResponse, serde_json::Error> {
    serde_json::from_slice(bytes)
}

pub fn input_registration(bytes: &[u8]) -> Option<(String, String, bool)> {
    match decode(bytes).ok()? {
        Request::Hello { params, .. }
            if params.protocol_version.minor >= 14
                && params
                    .offered_capabilities
                    .as_deref()
                    .is_some_and(|value| value.contains(&OfferedCapability::UserInput)) =>
        {
            let attachments = params.protocol_version.minor >= 19
                && params
                    .offered_capabilities
                    .as_deref()
                    .is_some_and(|value| value.contains(&OfferedCapability::UserInputAttachment));
            Some((params.agent_id, params.session_id?, attachments))
        }
        _ => None,
    }
}

fn valid_sdk_tool_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn safe_sdk_arguments(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(fields) => fields.iter().all(|(key, value)| {
            let normalized = key
                .bytes()
                .filter(|byte| *byte != b'_' && *byte != b'-')
                .map(|byte| byte.to_ascii_lowercase())
                .collect::<Vec<_>>();
            !matches!(
                normalized.as_slice(),
                b"pid"
                    | b"windowid"
                    | b"session"
                    | b"target"
                    | b"snapshotid"
                    | b"elementtoken"
                    | b"elementindex"
                    | b"workerinstanceid"
                    | b"hostsessionid"
                    | b"screenshotoutfile"
            ) && safe_sdk_arguments(value)
        }),
        serde_json::Value::Array(values) => values.iter().all(safe_sdk_arguments),
        serde_json::Value::String(value) => !value.contains('\0'),
        _ => true,
    }
}

pub fn sdk_arguments_json(value: &serde_json::Value) -> Result<String, RpcError> {
    serde_json::to_string(value).map_err(|_| RpcError::new(-32602, "非法桌面工具参数"))
}

pub fn generated_artifacts() -> Vec<(&'static str, String)> {
    let config = ts_rs::Config::new();
    let types = [
        Version::decl(&config),
        Capability::decl(&config),
        OfferedCapability::decl(&config),
        AgentInputSource::decl(&config),
        AgentAttachmentMime::decl(&config),
        AgentAttachmentBeginParams::decl(&config),
        AgentAttachmentChunkParams::decl(&config),
        AgentAttachmentFinishParams::decl(&config),
        AgentInputParams::decl(&config),
        AgentRequest::decl(&config),
        AgentInputResult::decl(&config),
        AgentResponse::decl(&config),
        ProtocolVersion::decl(&config),
        Platform::decl(&config),
        Availability::decl(&config),
        CapabilityInfo::decl(&config),
        HelloParams::decl(&config),
        CreateParams::decl(&config),
        CancelParams::decl(&config),
        WaitForUserParams::decl(&config),
        CompleteParams::decl(&config),
        ControlKind::decl(&config),
        ControlParams::decl(&config),
        StepDeclareParams::decl(&config),
        StepAdvanceParams::decl(&config),
        BrowserOperation::decl(&config),
        BrowserExecuteParams::decl(&config),
        ComputerExecuteParams::decl(&config),
        ComputerStepParams::decl(&config),
        GetParams::decl(&config),
        EventsParams::decl(&config),
        ListParams::decl(&config),
        Request::decl(&config),
        TaskStatus::decl(&config),
        TaskSource::decl(&config),
        TaskObservationResult::decl(&config),
        TaskObservation::decl(&config),
        TaskSnapshot::decl(&config),
        StepDeclaration::decl(&config),
        AttemptResultPhase::decl(&config),
        AttemptUnknownReason::decl(&config),
        AttemptResult::decl(&config),
        ControlPhase::decl(&config),
        FocusPhase::decl(&config),
        FocusFailure::decl(&config),
        ControlRecord::decl(&config),
        BrowserReference::decl(&config),
        ComputerObservation::decl(&config),
        TaskEvent::decl(&config),
        QueryResult::decl(&config),
        RpcError::decl(&config),
        Response::decl(&config),
    ];
    vec![
        (
            "request.schema.json",
            format!(
                "{}\n",
                serde_json::to_string_pretty(&schemars::schema_for!(Request)).unwrap()
            ),
        ),
        (
            "response.schema.json",
            format!(
                "{}\n",
                serde_json::to_string_pretty(
                    &schemars::generate::SchemaSettings::draft2020_12()
                        .for_serialize()
                        .into_generator()
                        .into_root_schema_for::<Response>()
                )
                .unwrap()
            ),
        ),
        (
            "agent-request.schema.json",
            format!(
                "{}\n",
                serde_json::to_string_pretty(&schemars::schema_for!(AgentRequest)).unwrap()
            ),
        ),
        (
            "agent-response.schema.json",
            format!(
                "{}\n",
                serde_json::to_string_pretty(&schemars::schema_for!(AgentResponse)).unwrap()
            ),
        ),
        (
            "protocol.ts",
            format!(
                "// 从 Rust 自动生成，请勿手改。\n{}\n",
                types
                    .into_iter()
                    .map(|t| format!("export {t}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_contract_rejects_invalid_inputs_and_keeps_integer_precision() {
        let raw = br#"{"jsonrpc":"2.0","id":"r1","method":"task.events","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"task_id":"t1","after_sequence":"9007199254740993","limit":100}}"#;
        let request = decode(raw).unwrap();
        assert!(request.validate(1999).is_ok());
        assert_eq!(request.validate(2000).unwrap_err().code, -32001);
        assert_eq!(sequence("9007199254740993"), Ok(9_007_199_254_740_993));
        assert_eq!(sequence("9223372036854775807"), Ok(i64::MAX as u64));
        for value in ["", "01", "-1", "+1", "1.0", " 1", "9223372036854775808"] {
            assert!(sequence(value).is_err());
        }
        let value: serde_json::Value = serde_json::from_slice(raw).unwrap();
        for field in [
            "agent_id",
            "capability",
            "deadline",
            "task_id",
            "after_sequence",
            "limit",
        ] {
            let mut missing = value.clone();
            missing["params"].as_object_mut().unwrap().remove(field);
            assert!(decode(&serde_json::to_vec(&missing).unwrap()).is_err());
        }
        for (field, invalid) in [
            ("limit", serde_json::json!(0)),
            ("limit", serde_json::json!(101)),
            ("agent_id", serde_json::json!("../a")),
            ("deadline", serde_json::json!(9_007_199_254_740_992_u64)),
        ] {
            let mut changed = value.clone();
            changed["params"][field] = invalid;
            assert!(
                decode(&serde_json::to_vec(&changed).unwrap())
                    .unwrap()
                    .validate(0)
                    .is_err()
            );
        }
        let mut extra = value.clone();
        extra["unexpected"] = true.into();
        assert!(decode(&serde_json::to_vec(&extra).unwrap()).is_err());
        let mut extra = value.clone();
        extra["params"]["unexpected"] = true.into();
        assert!(decode(&serde_json::to_vec(&extra).unwrap()).is_err());
        assert!(decode(&vec![b' '; MAX_REQUEST_BYTES + 1]).is_err());
        assert!(decode(b"[]").is_err());
        let response = Response::Success {
            jsonrpc: Version::V2,
            id: "r1".into(),
            result: QueryResult::Snapshot {
                task: TaskSnapshot {
                    task_id: "t1".into(),
                    owner_agent_id: "a1".into(),
                    name: None,
                    source: None,
                    status: TaskStatus::Interrupted,
                    sequence: "9007199254740993".into(),
                    current_step: None,
                    observation: None,
                    next_intent: None,
                },
            },
        };
        assert_eq!(
            serde_json::from_slice::<Response>(&encode(&response).unwrap()).unwrap(),
            response
        );
    }

    #[test]
    fn computer_bridge_accepts_sdk_arguments_and_rejects_host_identity() {
        let request = |arguments: serde_json::Value| serde_json::json!({"jsonrpc":"2.0","id":"c1","method":"computer.execute","params":{"agent_id":"a1","capability":"computer.execute","deadline":2000,"task_id":"t1","expected_sequence":"2","tool_name":"type_text","arguments":arguments}});
        assert!(
            decode(
                &serde_json::to_vec(&request(
                    serde_json::json!({"text":"hello","delivery_mode":"background"})
                ))
                .unwrap()
            )
            .unwrap()
            .validate(1)
            .is_ok()
        );
        assert!(
            decode(
                &serde_json::to_vec(&request(
                    serde_json::json!({"text":"hello","scope":"desktop"})
                ))
                .unwrap()
            )
            .unwrap()
            .validate(1)
            .is_ok()
        );
        for arguments in [
            serde_json::json!({"pid":1}),
            serde_json::json!({"target":{"window_id":2}}),
            serde_json::json!({"element_token":"opaque"}),
            serde_json::json!({"session":"forged"}),
        ] {
            assert!(
                decode(&serde_json::to_vec(&request(arguments)).unwrap())
                    .unwrap()
                    .validate(1)
                    .is_err()
            );
        }
        let step = serde_json::json!({"jsonrpc":"2.0","id":"c2","method":"computer.step","params":{"agent_id":"a1","capability":"computer.execute","deadline":2000,"task_id":"t1","expected_sequence":"2","step_id":"type","label":"输入文本","tool_name":"type_text","arguments":{"text":"hello"}}});
        assert!(matches!(
            decode(&serde_json::to_vec(&step).unwrap()).unwrap(),
            Request::ComputerStep { .. }
        ));
    }

    #[test]
    fn list_contract_defaults_and_rejects_invalid_pages() {
        let value = serde_json::json!({"jsonrpc":"2.0","id":"list-1","method":"task.list","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"limit":100}});
        let request = decode(&serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(request.validate(1999).is_ok());
        assert_eq!(request.validate(2000).unwrap_err().code, -32001);
        assert!(matches!(
            request,
            Request::List {
                params: ListParams {
                    after_task_id: None,
                    include_finished: false,
                    running_only: false,
                    ..
                },
                ..
            }
        ));
        for (field, invalid) in [
            ("limit", serde_json::json!(0)),
            ("limit", serde_json::json!(101)),
            ("after_task_id", serde_json::json!("")),
            ("after_task_id", serde_json::json!("../task")),
        ] {
            let mut changed = value.clone();
            changed["params"][field] = invalid;
            assert!(
                decode(&serde_json::to_vec(&changed).unwrap())
                    .unwrap()
                    .validate(0)
                    .is_err()
            );
        }
        for (field, invalid) in [
            ("limit", serde_json::json!(256)),
            ("include_finished", serde_json::json!("true")),
            ("unexpected", serde_json::json!(true)),
        ] {
            let mut changed = value.clone();
            changed["params"][field] = invalid;
            assert!(decode(&serde_json::to_vec(&changed).unwrap()).is_err());
        }
    }

    #[test]
    fn hello_contract_rejects_invalid_versions() {
        let value = serde_json::json!({"jsonrpc":"2.0","id":"h1","method":"gateway.hello","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":0}}});
        assert!(
            decode(&serde_json::to_vec(&value).unwrap())
                .unwrap()
                .validate(1999)
                .is_ok()
        );
        for version in [
            serde_json::json!({"major":-1,"minor":0}),
            serde_json::json!({"major":1,"minor":65536}),
            serde_json::json!({"major":1}),
            serde_json::json!({"major":1,"minor":0,"extra":true}),
            serde_json::Value::Null,
        ] {
            let mut changed = value.clone();
            changed["params"]["protocol_version"] = version;
            assert!(decode(&serde_json::to_vec(&changed).unwrap()).is_err());
        }
        let mut missing = value;
        missing["params"]
            .as_object_mut()
            .unwrap()
            .remove("protocol_version");
        assert!(decode(&serde_json::to_vec(&missing).unwrap()).is_err());
    }

    #[test]
    fn agent_attachment_contract_is_bounded_and_negotiated() {
        let hello = serde_json::json!({"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{"agent_id":"a1","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":19},"session_id":"s1","offered_capabilities":["user_input","user_input_attachment"]}});
        let hello = serde_json::to_vec(&hello).unwrap();
        assert!(decode(&hello).unwrap().validate(1000).is_ok());
        assert_eq!(input_registration(&hello), Some(("a1".into(), "s1".into(), true)));

        let begin = AgentAttachmentBeginParams { attachment_id: "image_1".into(), session_id: "s1".into(), mime: AgentAttachmentMime::ImagePng, byte_length: MAX_AGENT_ATTACHMENT_BYTES, sha256: "0".repeat(64), deadline: 2000 };
        assert!(begin.validate(1000).is_ok());
        let chunk = AgentAttachmentChunkParams { attachment_id: "image_1".into(), session_id: "s1".into(), sequence: 0, data_base64: "A".repeat(64_172) };
        assert!(chunk.validate().is_ok());
        assert!(encode_agent_request(&AgentRequest::AttachmentChunk { jsonrpc: Version::V2, params: chunk }).unwrap().len() <= MAX_REQUEST_BYTES);

        let mut invalid = begin.clone(); invalid.byte_length += 1;
        assert!(invalid.validate(1000).is_err());
        let invalid_chunk = AgentAttachmentChunkParams { attachment_id: "image_1".into(), session_id: "s1".into(), sequence: 0, data_base64: "====".into() };
        assert!(invalid_chunk.validate().is_err());

        let mut old = serde_json::from_slice::<serde_json::Value>(&hello).unwrap();
        old["params"]["protocol_version"]["minor"] = 18.into();
        assert!(decode(&serde_json::to_vec(&old).unwrap()).unwrap().validate(1000).is_err());
    }

    #[test]
    fn step_contract_is_strict_and_bounded() {
        let value = serde_json::json!({"jsonrpc":"2.0","id":"s1","method":"task.step.declare","params":{"agent_id":"a1","capability":"task.step.declare","deadline":2000,"task_id":"t1","expected_sequence":"1","step_id":"step-1","label":"打开文档"}});
        assert!(
            decode(&serde_json::to_vec(&value).unwrap())
                .unwrap()
                .validate(1999)
                .is_ok()
        );
        for (field, invalid) in [
            ("expected_sequence", serde_json::json!("0")),
            ("step_id", serde_json::json!("../bad")),
            ("label", serde_json::json!(" ")),
            ("label", serde_json::json!("a\n")),
            ("label", serde_json::json!("龙".repeat(86))),
        ] {
            let mut changed = value.clone();
            changed["params"][field] = invalid;
            assert!(
                decode(&serde_json::to_vec(&changed).unwrap())
                    .unwrap()
                    .validate(0)
                    .is_err()
            );
        }
        let mut extra = value;
        extra["params"]["action"] = serde_json::json!("click");
        assert!(decode(&serde_json::to_vec(&extra).unwrap()).is_err());
    }

    #[test]
    fn generated_contracts_are_current() {
        for (name, expected) in generated_artifacts() {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("generated")
                .join(name);
            assert_eq!(
                std::fs::read_to_string(path).unwrap(),
                expected,
                "生成产物过期：{name}"
            );
        }
    }
}

/// Agent名称限额；各调用边界复用，原样保存不做摘要或改名。
pub fn valid_task_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 256 && !name.chars().any(char::is_control)
}

pub fn valid_step_label(label: &str) -> bool {
    !label.trim().is_empty() && label.len() <= 256 && !label.chars().any(char::is_control)
}
