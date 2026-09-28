use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use interprocess::local_socket::{
    GenericFilePath, ToFsName,
    tokio::{Stream, prelude::*},
};
use serde_json::{Value, json};
use std::{
    collections::{HashMap, VecDeque},
    env, io,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
    sync::mpsc,
    task::JoinHandle,
};
use tokio_tungstenite::{WebSocketStream, client_async, tungstenite::Message};
use yonder_protocol::{
    AgentInputParams, AgentInputResult, AgentRequest, AgentResponse, Capability, HelloParams,
    MAX_REQUEST_BYTES, OfferedCapability, ProtocolVersion, PROTOCOL_VERSION, Request, Response, Version,
};

const CODEX_RPC_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_CODEX_FRAME_BYTES: usize = 1024 * 1024;
const MAX_DEDUP_RESULTS: usize = 1024;
const MAX_PENDING_RESPONSES: usize = 8;

struct CodexAppServer {
    input: SplitSink<WebSocketStream<UnixStream>, Message>,
    responses: mpsc::Receiver<io::Result<Value>>,
    reader: JoinHandle<()>,
    next_id: u64,
}

impl CodexAppServer {
    async fn connect(socket: &Path) -> io::Result<Self> {
        let stream = UnixStream::connect(socket)
            .await
            .map_err(|error| io::Error::other(format!("Codex App Server连接失败：{error}")))?;
        let (socket, _) = client_async("ws://localhost/", stream)
            .await
            .map_err(|error| {
                io::Error::other(format!("Codex App Server WebSocket握手失败：{error}"))
            })?;
        let (input, output) = socket.split();
        let (responses, reader) = read_responses(output);
        let mut server = Self {
            input,
            responses,
            reader,
            next_id: 0,
        };
        server.call("initialize", json!({"clientInfo":{"name":"yonder","title":"Yonder","version":env!("CARGO_PKG_VERSION")}}), CODEX_RPC_TIMEOUT).await?;
        server.notify("initialized", json!({})).await?;
        Ok(server)
    }

    async fn notify(&mut self, method: &str, params: Value) -> io::Result<()> {
        self.write(&json!({"method":method,"params":params})).await
    }

    async fn call(&mut self, method: &str, params: Value, timeout: Duration) -> io::Result<Value> {
        self.next_id = self.next_id.saturating_add(1);
        let id = self.next_id;
        self.write(&json!({"method":method,"id":id,"params":params}))
            .await?;
        tokio::time::timeout(timeout, async {
            loop {
                let message = self.responses.recv().await.ok_or_else(|| {
                    io::Error::new(io::ErrorKind::UnexpectedEof, "Codex App Server已断开")
                })??;
                if message.get("id").and_then(Value::as_u64) != Some(id) {
                    continue;
                }
                if let Some(result) = message.get("result") {
                    return Ok(result.clone());
                }
                let reason = message
                    .get("error")
                    .and_then(|error| error.get("message"))
                    .and_then(Value::as_str)
                    .unwrap_or("Codex App Server拒绝请求");
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    reason.to_owned(),
                ));
            }
        })
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Codex App Server确认超时"))?
    }

    async fn write(&mut self, message: &Value) -> io::Result<()> {
        let bytes = serde_json::to_vec(message).map_err(io::Error::other)?;
        if bytes.len() > MAX_CODEX_FRAME_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Codex App Server请求过大",
            ));
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Codex App Server请求无效"))?;
        self.input
            .send(Message::Text(text.into()))
            .await
            .map_err(io::Error::other)
    }

    async fn ensure_loaded(&mut self, thread_id: &str) -> io::Result<()> {
        let result = self
            .call("thread/loaded/list", json!({}), CODEX_RPC_TIMEOUT)
            .await?;
        let loaded = result
            .get("data")
            .and_then(Value::as_array)
            .is_some_and(|threads| {
                threads
                    .iter()
                    .any(|value| value.as_str() == Some(thread_id))
            });
        if loaded {
            Ok(())
        } else {
            Err(io::Error::other("Codex thread未在指定App Server中加载"))
        }
    }

    async fn deliver(
        &mut self,
        thread_id: &str,
        content: &str,
        timeout: Duration,
    ) -> io::Result<()> {
        let deadline = Instant::now() + timeout;
        let result = self
            .call(
                "thread/read",
                json!({"threadId":thread_id,"includeTurns":true}),
                deadline.saturating_duration_since(Instant::now()),
            )
            .await?;
        let thread = result
            .get("thread")
            .ok_or_else(|| io::Error::other("Codex thread状态缺失"))?;
        let status = thread
            .get("status")
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str)
            .ok_or_else(|| io::Error::other("Codex thread状态缺失"))?;
        let input = json!([{"type":"text","text":content}]);
        match status {
            "active" => {
                let turn_id = active_turn_id(thread)
                    .ok_or_else(|| io::Error::other("Codex活动turn标识缺失"))?;
                let result = self
                    .call(
                        "turn/steer",
                        json!({"threadId":thread_id,"input":input,"expectedTurnId":turn_id}),
                        deadline.saturating_duration_since(Instant::now()),
                    )
                    .await?;
                if result.get("turnId").and_then(Value::as_str) == Some(turn_id) {
                    Ok(())
                } else {
                    Err(io::Error::other("Codex steer确认无效"))
                }
            }
            "idle" => {
                let result = self
                    .call(
                        "turn/start",
                        json!({"threadId":thread_id,"input":input}),
                        deadline.saturating_duration_since(Instant::now()),
                    )
                    .await?;
                if result
                    .get("turn")
                    .and_then(|turn| turn.get("id"))
                    .and_then(Value::as_str)
                    .is_some()
                {
                    Ok(())
                } else {
                    Err(io::Error::other("Codex start确认无效"))
                }
            }
            "notLoaded" => Err(io::Error::other("Codex thread已卸载")),
            _ => Err(io::Error::other("Codex thread当前不可接收输入")),
        }
    }
}

impl Drop for CodexAppServer {
    fn drop(&mut self) {
        self.reader.abort();
    }
}

fn read_responses(
    mut output: SplitStream<WebSocketStream<UnixStream>>,
) -> (mpsc::Receiver<io::Result<Value>>, JoinHandle<()>) {
    let (sender, receiver) = mpsc::channel(MAX_PENDING_RESPONSES);
    let reader = tokio::spawn(async move {
        loop {
            let result: io::Result<Value> = match output.next().await {
                None | Some(Ok(Message::Close(_))) => Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Codex App Server已断开",
                )),
                Some(Ok(Message::Text(text))) if text.len() <= MAX_CODEX_FRAME_BYTES => {
                    serde_json::from_str(text.as_str()).map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidData, "Codex App Server响应无效")
                    })
                }
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => continue,
                Some(Ok(_)) => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Codex App Server响应无效",
                )),
                Some(Err(error)) => Err(io::Error::other(error)),
            };
            if result
                .as_ref()
                .is_ok_and(|message| message.get("method").is_some())
            {
                continue;
            }
            let terminal = result.is_err();
            if sender.send(result).await.is_err() || terminal {
                break;
            }
        }
    });
    (receiver, reader)
}

fn active_turn_id(thread: &Value) -> Option<&str> {
    thread
        .get("turns")?
        .as_array()?
        .iter()
        .rev()
        .find(|turn| turn.get("status").and_then(Value::as_str) == Some("inProgress"))?
        .get("id")?
        .as_str()
}

struct DedupResults {
    values: HashMap<String, bool>,
    order: VecDeque<String>,
}

impl DedupResults {
    fn new() -> Self {
        Self {
            values: HashMap::new(),
            order: VecDeque::new(),
        }
    }
    fn get(&self, input_id: &str) -> Option<bool> {
        self.values.get(input_id).copied()
    }
    fn insert(&mut self, input_id: String, accepted: bool) {
        if self.values.contains_key(&input_id) {
            return;
        }
        if self.order.len() == MAX_DEDUP_RESULTS {
            if let Some(expired) = self.order.pop_front() {
                self.values.remove(&expired);
            }
        }
        self.order.push_back(input_id.clone());
        self.values.insert(input_id, accepted);
    }
}

fn now_ms() -> io::Result<u64> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_millis(),
    )
    .map_err(io::Error::other)
}

fn app_server_socket() -> io::Result<PathBuf> {
    let value = env::var_os("YONDER_CODEX_APP_SERVER_SOCKET")
        .ok_or_else(|| io::Error::other("请设置YONDER_CODEX_APP_SERVER_SOCKET"))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(io::Error::other(
            "YONDER_CODEX_APP_SERVER_SOCKET必须是绝对路径",
        ));
    }
    Ok(path)
}

fn thread_id() -> io::Result<String> {
    let value = env::var("YONDER_CODEX_THREAD_ID")
        .map_err(|_| io::Error::other("请设置YONDER_CODEX_THREAD_ID"))?;
    if yonder_protocol::valid_id(&value) {
        Ok(value)
    } else {
        Err(io::Error::other("YONDER_CODEX_THREAD_ID无效"))
    }
}

async fn send_yonder(stream: &Stream, value: &impl serde::Serialize) -> io::Result<()> {
    let mut bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Yonder Agent帧过大",
        ));
    }
    bytes.push(b'\n');
    (&*stream).write_all(&bytes).await
}

async fn read_yonder(reader: &mut BufReader<&Stream>) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let read = reader.read_until(b'\n', &mut bytes).await?;
    if read == 0 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Yonder已断开"));
    }
    if read > MAX_REQUEST_BYTES || bytes.last() != Some(&b'\n') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Yonder Agent帧无效",
        ));
    }
    bytes.pop();
    Ok(bytes)
}

pub async fn run(agent_id: String, yonder_socket: PathBuf) -> io::Result<()> {
    let thread_id = thread_id()?;
    let mut codex = CodexAppServer::connect(&app_server_socket()?).await?;
    codex.ensure_loaded(&thread_id).await?;

    let stream = Stream::connect(yonder_socket.to_fs_name::<GenericFilePath>()?)
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Yonder未运行"))?;
    let hello = Request::Hello {
        jsonrpc: Version::V2,
        request_id: "hello".into(),
        params: HelloParams {
            agent_id,
            capability: Capability::TaskRead,
            deadline: now_ms()?.saturating_add(10_000),
            protocol_version: ProtocolVersion {
                major: PROTOCOL_VERSION.major,
                // 桥只使用 agent.input 文本通道；钉在 1.14+ 既有能力，不随新版本隐式扩权。
                minor: 19,
            },
            session_id: Some(thread_id.clone()),
            offered_capabilities: Some(vec![OfferedCapability::UserInput]),
        },
    };
    send_yonder(&stream, &hello).await?;
    let mut reader = BufReader::new(&stream);
    match serde_json::from_slice::<Response>(&read_yonder(&mut reader).await?) {
        Ok(Response::Success { .. }) => {}
        Ok(Response::Failure { error, .. }) => return Err(io::Error::other(error.message)),
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Yonder握手响应无效",
            ));
        }
    }

    let mut dedup = DedupResults::new();
    loop {
        let bytes = read_yonder(&mut reader).await?;
        let request = serde_json::from_slice::<AgentRequest>(&bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Yonder Agent请求无效"))?;
        let AgentRequest::Input {
            request_id, params, ..
        } = request
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Codex桥接不支持Agent附件",
            ));
        };
        let accepted = handle_input(&mut codex, &thread_id, &params, &mut dedup).await?;
        send_yonder(
            &stream,
            &AgentResponse::Success {
                jsonrpc: Version::V2,
                id: request_id,
                result: AgentInputResult { accepted },
            },
        )
        .await?;
    }
}

async fn handle_input(
    codex: &mut CodexAppServer,
    thread_id: &str,
    params: &AgentInputParams,
    dedup: &mut DedupResults,
) -> io::Result<bool> {
    if let Some(accepted) = dedup.get(&params.input_id) {
        return Ok(accepted);
    }
    let now = now_ms()?;
    if params.session_id != thread_id
        || params.attachment_id.is_some()
        || params.validate(now).is_err()
    {
        dedup.insert(params.input_id.clone(), false);
        return Ok(false);
    }
    let timeout = Duration::from_millis(params.deadline.saturating_sub(now)).min(CODEX_RPC_TIMEOUT);
    match codex.deliver(thread_id, &params.content, timeout).await {
        Ok(()) => {
            dedup.insert(params.input_id.clone(), true);
            Ok(true)
        }
        Err(error) if error.kind() == io::ErrorKind::InvalidInput => {
            dedup.insert(params.input_id.clone(), false);
            Ok(false)
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_only_the_in_progress_turn() {
        let thread = json!({"turns":[{"id":"old","status":"completed"},{"id":"active","status":"inProgress"}]});
        assert_eq!(active_turn_id(&thread), Some("active"));
        assert_eq!(
            active_turn_id(&json!({"turns":[{"id":"done","status":"interrupted"}]})),
            None
        );
    }

    #[test]
    fn dedup_is_bounded_and_keeps_the_first_result() {
        let mut results = DedupResults::new();
        results.insert("same".into(), true);
        results.insert("same".into(), false);
        assert_eq!(results.get("same"), Some(true));
        for index in 0..MAX_DEDUP_RESULTS {
            results.insert(format!("input-{index}"), false);
        }
        assert!(results.values.len() <= MAX_DEDUP_RESULTS);
        assert_eq!(results.get("same"), None);
    }
}
