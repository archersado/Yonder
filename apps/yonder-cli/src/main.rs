use interprocess::local_socket::{GenericFilePath, ToFsName, tokio::{Stream, prelude::*}};
use serde_json::{Value, json};
use std::{env, io::{self, BufRead, Write}, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use yonder_protocol::{BrowserExecuteParams, BrowserOperation, Capability, CancelParams, CompleteParams, ComputerExecuteParams, ComputerStepParams, ControlKind, ControlParams, CreateParams, EventsParams, GetParams, HelloParams, ListParams, ProtocolVersion, Request, Response, StepAdvanceParams, StepDeclareParams, Version, WaitForUserParams, MAX_REQUEST_BYTES};

const AGENT_ID: &str = "codex-cli";

fn socket_path() -> io::Result<PathBuf> {
    let home = env::var_os("HOME").ok_or_else(|| io::Error::other("无法定位Yonder数据目录"))?;
    Ok(PathBuf::from(home).join("Library/Application Support/com.yonder.desktop/agent.sock"))
}

fn now_ms() -> io::Result<u64> {
    u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?.as_millis()).map_err(io::Error::other)
}

async fn send(stream: &Stream, request: &Request) -> io::Result<Response> {
    let mut bytes = serde_json::to_vec(request).map_err(io::Error::other)?;
    if bytes.len() > MAX_REQUEST_BYTES { return Err(io::Error::other("请求过大")); }
    bytes.push(b'\n');
    (&*stream).write_all(&bytes).await?;
    let mut line = Vec::new();
    BufReader::new(stream).read_until(b'\n', &mut line).await?;
    if line.is_empty() || line.len() > MAX_REQUEST_BYTES + 1 || line.last() != Some(&b'\n') { return Err(io::Error::other("Yonder响应无效")); }
    line.pop();
    serde_json::from_slice(&line).map_err(io::Error::other)
}

async fn gateway(request: Request) -> io::Result<Response> {
    let stream = Stream::connect(socket_path()?.to_fs_name::<GenericFilePath>()?).await
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Yonder未运行"))?;
    let hello = Request::Hello { jsonrpc: Version::V2, request_id: "hello".into(), params: HelloParams {
        agent_id: AGENT_ID.into(), capability: Capability::TaskRead, deadline: now_ms()? + 60_000,
        protocol_version: ProtocolVersion { major: 1, minor: 18 },
        session_id: None, offered_capabilities: None,
    }};
    match send(&stream, &hello).await? {
        Response::Success { .. } => send(&stream, &request).await,
        Response::Failure { error, .. } => Err(io::Error::other(error.message)),
    }
}

fn text_result(value: Value, is_error: bool) -> Value {
    json!({"content":[{"type":"text","text":value.to_string()}],"isError":is_error})
}

fn field<'a>(args: &'a Value, name: &str) -> Result<&'a str, String> {
    args.get(name).and_then(Value::as_str).filter(|v| !v.is_empty()).ok_or_else(|| format!("缺少参数：{name}"))
}

fn request(name: &str, args: &Value, id: String, deadline: u64) -> Result<Request, String> {
    let base = || (Version::V2, id.clone());
    match name {
        "task_create" => { let (jsonrpc, request_id) = base(); Ok(Request::Create { jsonrpc, request_id, params: CreateParams { agent_id: AGENT_ID.into(), capability: Capability::TaskCreate, deadline, idempotency_key: field(args,"idempotency_key")?.into(), description: field(args,"description")?.into(), name: Some(field(args,"name")?.into()) } }) },
        "task_list" => { let (jsonrpc, request_id) = base(); Ok(Request::List { jsonrpc, request_id, params: ListParams { agent_id: AGENT_ID.into(), capability: Capability::TaskRead, deadline, after_task_id: args.get("after_task_id").and_then(Value::as_str).map(str::to_owned), include_finished: args.get("include_finished").and_then(Value::as_bool).unwrap_or(false), running_only: args.get("running_only").and_then(Value::as_bool).unwrap_or(false), limit: args.get("limit").and_then(Value::as_u64).unwrap_or(100).try_into().map_err(|_| "limit无效")? } }) },
        "task_get" | "task_step_get" => { let (jsonrpc, request_id) = base(); let params = GetParams { agent_id: AGENT_ID.into(), capability: Capability::TaskRead, deadline, task_id: field(args,"task_id")?.into() }; Ok(if name == "task_get" { Request::Get { jsonrpc, request_id, params } } else { Request::StepGet { jsonrpc, request_id, params } }) },
        "task_cancel" => { let (jsonrpc, request_id) = base(); Ok(Request::Cancel { jsonrpc, request_id, params: CancelParams { agent_id: AGENT_ID.into(), capability: Capability::TaskCancel, deadline, task_id: field(args,"task_id")?.into(), expected_sequence: field(args,"expected_sequence")?.into() } }) },
        "task_control" => { let (jsonrpc, request_id) = base(); let kind=match field(args,"kind")? { "pause"=>ControlKind::Pause,"cancel"=>ControlKind::Cancel,"takeover"=>ControlKind::Takeover,_=>return Err("kind须为pause、cancel或takeover".into()) }; Ok(Request::Control { jsonrpc,request_id,params:ControlParams { agent_id:AGENT_ID.into(),capability:Capability::TaskControl,deadline,task_id:field(args,"task_id")?.into(),expected_sequence:field(args,"expected_sequence")?.into(),kind } }) },
        "task_events" => { let (jsonrpc, request_id) = base(); Ok(Request::Events { jsonrpc, request_id, params: EventsParams { agent_id: AGENT_ID.into(), capability: Capability::TaskRead, deadline, task_id: field(args,"task_id")?.into(), after_sequence: args.get("after_sequence").and_then(Value::as_str).unwrap_or("0").into(), limit: args.get("limit").and_then(Value::as_u64).unwrap_or(100).try_into().map_err(|_| "limit无效")? } }) },
        "task_step_declare" => { let (jsonrpc, request_id) = base(); Ok(Request::StepDeclare { jsonrpc, request_id, params: StepDeclareParams { agent_id: AGENT_ID.into(), capability: Capability::TaskStepDeclare, deadline, task_id: field(args,"task_id")?.into(), expected_sequence: field(args,"expected_sequence")?.into(), step_id: field(args,"step_id")?.into(), label: field(args,"label")?.into() } }) },
        "task_step_advance" => { let (jsonrpc,request_id)=base(); Ok(Request::StepAdvance { jsonrpc,request_id,params:StepAdvanceParams { agent_id:AGENT_ID.into(),capability:Capability::TaskStepAdvance,deadline,task_id:field(args,"task_id")?.into(),expected_sequence:field(args,"expected_sequence")?.into() } }) },
        "browser_execute" => { let operation=match field(args,"operation")? { "create"=>BrowserOperation::Create,"observe"=>BrowserOperation::Observe,"hand-off"=>BrowserOperation::HandOff,"take-over"=>BrowserOperation::TakeOver,"finish"=>BrowserOperation::Finish,_=>return Err("operation须为create、observe、hand-off、take-over或finish".into()) }; let (jsonrpc,request_id)=base(); Ok(Request::BrowserExecute { jsonrpc,request_id,params:BrowserExecuteParams { agent_id:AGENT_ID.into(),capability:Capability::BrowserExecute,deadline,task_id:field(args,"task_id")?.into(),expected_sequence:field(args,"expected_sequence")?.into(),operation } }) },
        "computer_execute" => { let arguments=args.get("arguments").filter(|value|value.is_object()).cloned().ok_or("缺少参数：arguments")?;let (jsonrpc,request_id)=base(); Ok(Request::ComputerExecute { jsonrpc,request_id,params:ComputerExecuteParams { agent_id:AGENT_ID.into(),capability:Capability::ComputerExecute,deadline,task_id:field(args,"task_id")?.into(),expected_sequence:field(args,"expected_sequence")?.into(),tool_name:field(args,"tool_name")?.into(),arguments } }) },
        "computer_step" => { let arguments=args.get("arguments").filter(|value|value.is_object()).cloned().ok_or("缺少参数：arguments")?;let (jsonrpc,request_id)=base(); Ok(Request::ComputerStep { jsonrpc,request_id,params:ComputerStepParams { agent_id:AGENT_ID.into(),capability:Capability::ComputerExecute,deadline,task_id:field(args,"task_id")?.into(),expected_sequence:field(args,"expected_sequence")?.into(),step_id:field(args,"step_id")?.into(),label:field(args,"label")?.into(),tool_name:field(args,"tool_name")?.into(),arguments } }) },
        "task_complete" => { let (jsonrpc,request_id)=base(); Ok(Request::Complete { jsonrpc,request_id,params:CompleteParams { agent_id:AGENT_ID.into(),capability:Capability::TaskComplete,deadline,task_id:field(args,"task_id")?.into(),expected_sequence:field(args,"expected_sequence")?.into() } }) },
        "task_fail" => { let (jsonrpc,request_id)=base(); Ok(Request::Fail { jsonrpc,request_id,params:CompleteParams { agent_id:AGENT_ID.into(),capability:Capability::TaskFail,deadline,task_id:field(args,"task_id")?.into(),expected_sequence:field(args,"expected_sequence")?.into() } }) },
        "task_wait_for_user" => { let (jsonrpc,request_id)=base(); Ok(Request::WaitForUser { jsonrpc,request_id,params:WaitForUserParams { agent_id:AGENT_ID.into(),capability:Capability::TaskWaitForUser,deadline,task_id:field(args,"task_id")?.into(),expected_sequence:field(args,"expected_sequence")?.into(),reason:field(args,"reason")?.into() } }) },
        _ => Err("未知工具".into()),
    }
}

fn tools() -> Value {
    let object = |required: Vec<&str>, properties: Value| json!({"type":"object","properties":properties,"required":required,"additionalProperties":false});
    json!([
        {"name":"task_create","description":"登记一个由Agent创建的Yonder任务","inputSchema":object(vec!["idempotency_key","name","description"],json!({"idempotency_key":{"type":"string"},"name":{"type":"string"},"description":{"type":"string"}}))},
        {"name":"task_list","description":"列出当前Agent的Yonder任务","inputSchema":object(vec![],json!({"after_task_id":{"type":"string"},"include_finished":{"type":"boolean"},"running_only":{"type":"boolean"},"limit":{"type":"integer","minimum":1,"maximum":100}}))},
        {"name":"task_get","description":"读取Yonder任务快照","inputSchema":object(vec!["task_id"],json!({"task_id":{"type":"string"}}))},
        {"name":"task_cancel","description":"取消尚未开始的Yonder任务并保留数据","inputSchema":object(vec!["task_id","expected_sequence"],json!({"task_id":{"type":"string"},"expected_sequence":{"type":"string"}}))},
        {"name":"task_control","description":"请求在步骤边界暂停、取消或接管执行中的Yonder任务","inputSchema":object(vec!["task_id","expected_sequence","kind"],json!({"task_id":{"type":"string"},"expected_sequence":{"type":"string"},"kind":{"type":"string","enum":["pause","cancel","takeover"]}}))},
        {"name":"task_events","description":"读取Yonder任务增量事件","inputSchema":object(vec!["task_id"],json!({"task_id":{"type":"string"},"after_sequence":{"type":"string"},"limit":{"type":"integer","minimum":1,"maximum":100}}))},
        {"name":"task_step_declare","description":"声明Agent当前任务步骤","inputSchema":object(vec!["task_id","expected_sequence","step_id","label"],json!({"task_id":{"type":"string"},"expected_sequence":{"type":"string"},"step_id":{"type":"string"},"label":{"type":"string"}}))},
        {"name":"task_step_get","description":"读取Agent当前任务步骤","inputSchema":object(vec!["task_id"],json!({"task_id":{"type":"string"}}))}
        ,{"name":"task_step_advance","description":"在动作已Observe后推进至下一步骤边界","inputSchema":object(vec!["task_id","expected_sequence"],json!({"task_id":{"type":"string"},"expected_sequence":{"type":"string"}}))}
        ,{"name":"browser_execute","description":"通过Yonder托管的ego-lite Browser Task Space执行并Observe动作","inputSchema":object(vec!["task_id","expected_sequence","operation"],json!({"task_id":{"type":"string"},"expected_sequence":{"type":"string"},"operation":{"type":"string","enum":["create","observe","hand-off","take-over","finish"]}}))}
        ,{"name":"computer_step","description":"由Yonder一次完成步骤声明、CUA动作、Observe和步骤推进","inputSchema":object(vec!["task_id","expected_sequence","step_id","label","tool_name","arguments"],json!({"task_id":{"type":"string"},"expected_sequence":{"type":"string"},"step_id":{"type":"string"},"label":{"type":"string"},"tool_name":{"type":"string"},"arguments":{"type":"object","additionalProperties":true}}))}
        ,{"name":"task_complete","description":"完成已Observe并推进边界的CUA任务","inputSchema":object(vec!["task_id","expected_sequence"],json!({"task_id":{"type":"string"},"expected_sequence":{"type":"string"}}))}
        ,{"name":"task_fail","description":"终结最新已Observe失败并推进边界的CUA任务","inputSchema":object(vec!["task_id","expected_sequence"],json!({"task_id":{"type":"string"},"expected_sequence":{"type":"string"}}))}
        ,{"name":"task_wait_for_user","description":"在已Observe并推进的步骤边界等待用户处理","inputSchema":object(vec!["task_id","expected_sequence","reason"],json!({"task_id":{"type":"string"},"expected_sequence":{"type":"string"},"reason":{"type":"string","minLength":1,"maxLength":512}}))}
    ])
}

async fn handle(message: Value, counter: &mut u64) -> Option<Value> {
    let id = message.get("id")?.clone();
    let method = message.get("method").and_then(Value::as_str).unwrap_or("");
    let result = match method {
        "initialize" => json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"yonder","version":"0.1.0"},"instructions":"任务只能由Agent创建。创建时提供简短名称、说明和稳定幂等键；执行动作前声明当前步骤。"}),
        "ping" => json!({}),
        "tools/list" => json!({"tools":tools()}),
        "tools/call" => {
            *counter += 1;
            let params = message.get("params").unwrap_or(&Value::Null);
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").unwrap_or(&Value::Null);
            match now_ms().map_err(|e| e.to_string()).and_then(|now| request(name, args, format!("mcp-{counter}"), now + 60_000)) {
                Ok(request) => match gateway(request).await {
                    Ok(Response::Success { result, .. }) => text_result(serde_json::to_value(result).unwrap_or(Value::Null), false),
                    Ok(Response::Failure { error, .. }) => text_result(json!({"code":error.code,"message":error.message}), true),
                    Err(error) => text_result(json!({"message":error.to_string()}), true),
                },
                Err(error) => text_result(json!({"message":error}), true),
            }
        },
        _ => return Some(json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"方法不存在"}})),
    };
    Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
}

async fn agent_bridge() -> io::Result<()> {
    Err(io::Error::other("Codex CLI当前只提供持久队列，未暴露可确认提交的当前会话输入通道"))
}

async fn mcp() {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let mut counter = 0;
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.len() > MAX_REQUEST_BYTES { break; }
        let Ok(message) = serde_json::from_str::<Value>(&line) else { continue };
        if let Some(response) = handle(message, &mut counter).await {
            if serde_json::to_writer(&mut stdout, &response).is_err() || writeln!(stdout).is_err() || stdout.flush().is_err() { break; }
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    match env::args().nth(1).as_deref() {
        Some("mcp")=>mcp().await,
        Some("agent-bridge")=>if let Err(error)=agent_bridge().await{eprintln!("Agent输入桥接关闭：{error}");std::process::exit(1)},
        _=>{eprintln!("用法：yonder <mcp|agent-bridge>");std::process::exit(2)}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mcp_tools_build_typed_gateway_requests() {
        let create_request = request("task_create", &json!({"idempotency_key":"same","name":"整理文档","description":"整理指定文档"}), "r1".into(), 2000).unwrap();
        let encoded = serde_json::to_vec(&create_request).unwrap();
        assert!(matches!(yonder_protocol::decode(&encoded).unwrap(), Request::Create { .. }));
        assert!(matches!(request("task_control",&json!({"task_id":"task-1","expected_sequence":"4","kind":"takeover"}),"r2".into(),2000).unwrap(),Request::Control { params:ControlParams { kind:ControlKind::Takeover,.. },.. }));
        assert!(matches!(request("browser_execute",&json!({"task_id":"task-1","expected_sequence":"4","operation":"observe"}),"r3".into(),2000).unwrap(),Request::BrowserExecute { params:BrowserExecuteParams { operation:BrowserOperation::Observe,.. },.. }));
        assert!(matches!(request("computer_step",&json!({"task_id":"task-1","expected_sequence":"4","step_id":"type","label":"输入文本","tool_name":"type_text","arguments":{"text":"hello"}}),"r4".into(),2000).unwrap(),Request::ComputerStep { .. }));
        let tools=tools();let names=tools.as_array().unwrap().iter().filter_map(|tool|tool["name"].as_str()).collect::<Vec<_>>();
        assert_eq!(names.len(),14);assert!(names.contains(&"computer_step"));assert!(names.contains(&"task_fail"));assert!(names.contains(&"task_wait_for_user"));assert!(!names.contains(&"computer_execute"));
    }
}
