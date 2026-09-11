//! 只读协议唯一来源；此模块不提供传输或身份认证。
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub const MAX_REQUEST_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
pub enum Version { #[serde(rename = "2.0")] V2 }

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
pub enum Capability { #[serde(rename = "task.read")] TaskRead }

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
#[serde(tag = "method", deny_unknown_fields)]
pub enum Request {
    #[serde(rename = "task.get")]
    Get { jsonrpc: Version, #[serde(rename = "id")] request_id: String, params: GetParams },
    #[serde(rename = "task.events")]
    Events { jsonrpc: Version, #[serde(rename = "id")] request_id: String, params: EventsParams },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus { Created, Running, WaitingForUser, Paused, Interrupted, Completed, Failed, Cancelled }

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct TaskSnapshot {
    pub task_id: String,
    pub status: TaskStatus,
    pub sequence: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct TaskEvent {
    pub previous: TaskStatus,
    pub status: TaskStatus,
    pub sequence: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum QueryResult {
    Snapshot { task: TaskSnapshot },
    Events { task_id: String, events: Vec<TaskEvent> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(deny_unknown_fields)]
pub struct RpcError { pub code: i32, pub message: String }

impl RpcError {
    pub fn new(code: i32, message: &str) -> Self { Self { code, message: message.into() } }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(untagged, deny_unknown_fields)]
pub enum Response {
    Success { jsonrpc: Version, id: String, result: QueryResult },
    Failure { jsonrpc: Version, id: Option<String>, error: RpcError },
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 128 && id.bytes().all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
}

pub fn sequence(value: &str) -> Result<u64, RpcError> {
    if value.is_empty() || value.len() > 19 || (value.len() > 1 && value.starts_with('0')) || !value.bytes().all(|c| c.is_ascii_digit()) {
        return Err(RpcError::new(-32602, "非法任务序号"));
    }
    value.parse::<u64>().ok().filter(|n| *n <= i64::MAX as u64).ok_or_else(|| RpcError::new(-32602, "非法任务序号"))
}

impl Request {
    pub fn request_id(&self) -> &str {
        match self { Self::Get { request_id, .. } | Self::Events { request_id, .. } => request_id }
    }

    pub fn validate(&self, now_ms: u64) -> Result<(), RpcError> {
        let (agent_id, task_id, deadline) = match self {
            Self::Get { params, .. } => (&params.agent_id, &params.task_id, params.deadline),
            Self::Events { params, .. } => {
                sequence(&params.after_sequence)?;
                if !(1..=100).contains(&params.limit) { return Err(RpcError::new(-32602, "非法分页上限")); }
                (&params.agent_id, &params.task_id, params.deadline)
            }
        };
        if !valid_id(self.request_id()) || !valid_id(agent_id) || !valid_id(task_id) || deadline > 9_007_199_254_740_991 {
            return Err(RpcError::new(-32602, "非法请求参数"));
        }
        if deadline <= now_ms { return Err(RpcError::new(-32001, "请求已过期")); }
        Ok(())
    }
}

/// 传输层也必须有界读帧，不能先无限读取再依赖此处限长。
pub fn decode(bytes: &[u8]) -> Result<Request, RpcError> {
    if bytes.len() > MAX_REQUEST_BYTES { return Err(RpcError::new(-32600, "请求过大")); }
    serde_json::from_slice(bytes).map_err(|error| {
        let code = if error.is_syntax() || error.is_eof() { -32700 } else { -32600 };
        RpcError::new(code, "无效只读请求")
    })
}

pub fn encode(response: &Response) -> Result<Vec<u8>, serde_json::Error> { serde_json::to_vec(response) }

pub fn generated_artifacts() -> Vec<(&'static str, String)> {
    let config = ts_rs::Config::new();
    let types = [Version::decl(&config), Capability::decl(&config), GetParams::decl(&config), EventsParams::decl(&config), Request::decl(&config), TaskStatus::decl(&config), TaskSnapshot::decl(&config), TaskEvent::decl(&config), QueryResult::decl(&config), RpcError::decl(&config), Response::decl(&config)];
    vec![
        ("request.schema.json", format!("{}\n", serde_json::to_string_pretty(&schemars::schema_for!(Request)).unwrap())),
        ("response.schema.json", format!("{}\n", serde_json::to_string_pretty(&schemars::generate::SchemaSettings::draft2020_12().for_serialize().into_generator().into_root_schema_for::<Response>()).unwrap())),
        ("protocol.ts", format!("// 从 Rust 自动生成，请勿手改。\n{}\n", types.into_iter().map(|t| format!("export {t}")).collect::<Vec<_>>().join("\n"))),
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
        for value in ["", "01", "-1", "+1", "1.0", " 1", "9223372036854775808"] { assert!(sequence(value).is_err()); }
        let value: serde_json::Value = serde_json::from_slice(raw).unwrap();
        for field in ["agent_id", "capability", "deadline", "task_id", "after_sequence", "limit"] {
            let mut missing = value.clone();
            missing["params"].as_object_mut().unwrap().remove(field);
            assert!(decode(&serde_json::to_vec(&missing).unwrap()).is_err());
        }
        for (field, invalid) in [("limit", serde_json::json!(0)), ("limit", serde_json::json!(101)), ("agent_id", serde_json::json!("../a")), ("deadline", serde_json::json!(9_007_199_254_740_992_u64))] {
            let mut changed = value.clone(); changed["params"][field] = invalid;
            assert!(decode(&serde_json::to_vec(&changed).unwrap()).unwrap().validate(0).is_err());
        }
        let mut extra = value.clone(); extra["unexpected"] = true.into();
        assert!(decode(&serde_json::to_vec(&extra).unwrap()).is_err());
        let mut extra = value.clone(); extra["params"]["unexpected"] = true.into();
        assert!(decode(&serde_json::to_vec(&extra).unwrap()).is_err());
        assert!(decode(&vec![b' '; MAX_REQUEST_BYTES + 1]).is_err());
        assert!(decode(b"[]").is_err());
        let response = Response::Success { jsonrpc: Version::V2, id: "r1".into(), result: QueryResult::Snapshot { task: TaskSnapshot { task_id: "t1".into(), status: TaskStatus::Interrupted, sequence: "9007199254740993".into() } } };
        assert_eq!(serde_json::from_slice::<Response>(&encode(&response).unwrap()).unwrap(), response);
    }

    #[test]
    fn generated_contracts_are_current() {
        for (name, expected) in generated_artifacts() {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("generated").join(name);
            assert_eq!(std::fs::read_to_string(path).unwrap(), expected, "生成产物过期：{name}");
        }
    }
}
