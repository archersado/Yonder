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
    pin::Pin,
    path::{Path, PathBuf},
    process::Stdio,
    task::{Context, Poll},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf},
    net::UnixStream,
    process::{Child, ChildStdin, ChildStdout, Command},
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
const YONDER_RECONNECT_WINDOW: Duration = Duration::from_secs(300);
const YONDER_RECONNECT_INITIAL: Duration = Duration::from_millis(100);
const YONDER_RECONNECT_MAX: Duration = Duration::from_secs(2);

trait CodexTransport: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> CodexTransport for T {}
type BoxedCodexTransport = Box<dyn CodexTransport>;
type CodexSocket = WebSocketStream<BoxedCodexTransport>;

struct ChildDuplex {
    input: ChildStdin,
    output: ChildStdout,
}

impl AsyncRead for ChildDuplex {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.output).poll_read(cx, buffer)
    }
}

impl AsyncWrite for ChildDuplex {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.input).poll_write(cx, buffer)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.input).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.input).poll_shutdown(cx)
    }
}

struct CodexAppServer {
    input: SplitSink<CodexSocket, Message>,
    responses: mpsc::Receiver<io::Result<Value>>,
    reader: JoinHandle<()>,
    proxy: Option<Child>,
    next_id: u64,
}

impl CodexAppServer {
    async fn connect(socket: &Path) -> io::Result<Self> {
        let stream = UnixStream::connect(socket)
            .await
            .map_err(|error| io::Error::other(format!("Codex App Server连接失败：{error}")))?;
        Self::connect_transport(Box::new(stream), None).await
    }

    async fn connect_transport(
        transport: BoxedCodexTransport,
        proxy: Option<Child>,
    ) -> io::Result<Self> {
        let (socket, _) = client_async("ws://localhost/", transport)
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
            proxy,
            next_id: 0,
        };
        server.initialize().await?;
        Ok(server)
    }

    async fn connect_managed() -> io::Result<Self> {
        Self::connect_managed_with(&codex_program()).await
    }

    async fn connect_managed_with(codex: &Path) -> io::Result<Self> {
        let (transport, proxy) = spawn_managed_proxy(codex).await?;
        Self::connect_transport(transport, Some(proxy)).await
    }

    async fn initialize(&mut self) -> io::Result<()> {
        self.call("initialize", json!({"clientInfo":{"name":"yonder","title":"Yonder","version":env!("CARGO_PKG_VERSION")}}), CODEX_RPC_TIMEOUT).await?;
        self.notify("initialized", json!({})).await
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
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Codex App Server拒绝请求",
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

    async fn resume(&mut self, thread_id: &str) -> io::Result<()> {
        let result = self
            .call(
                "thread/resume",
                json!({"threadId":thread_id,"excludeTurns":true}),
                CODEX_RPC_TIMEOUT,
            )
            .await?;
        if result
            .get("thread")
            .and_then(|thread| thread.get("id"))
            .and_then(Value::as_str)
            == Some(thread_id)
        {
            Ok(())
        } else {
            Err(io::Error::other("Codex thread恢复确认无效"))
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
                json!({"threadId":thread_id,"includeTurns":false}),
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
                // 不拉取整段历史；长会话的完整turn载荷会超过App Server帧上限。
                // 只分页读取最新turn的元数据以取得steer所需的活动turn id。
                let turns = self
                    .call(
                        "thread/turns/list",
                        json!({"threadId":thread_id,"limit":1,"sortDirection":"desc","itemsView":"notLoaded"}),
                        deadline.saturating_duration_since(Instant::now()),
                    )
                    .await?;
                let turn_id = active_turn_id_from_page(&turns)
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

async fn spawn_managed_proxy(codex: &Path) -> io::Result<(BoxedCodexTransport, Child)> {
        let status = tokio::time::timeout(
            CODEX_RPC_TIMEOUT,
            Command::new(codex)
                .args(["app-server", "daemon", "start"])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status(),
        )
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Codex daemon启动超时"))?
        .map_err(|error| io::Error::new(io::ErrorKind::NotFound, format!("Codex CLI不可用：{error}")))?;
        if !status.success() {
            return Err(io::Error::other("Codex daemon启动失败"));
        }

        let mut proxy = Command::new(codex)
            .args(["app-server", "proxy"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|error| io::Error::other(format!("Codex proxy启动失败：{error}")))?;
        let input = proxy
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("Codex proxy输入不可用"))?;
        let output = proxy
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("Codex proxy输出不可用"))?;
        Ok((Box::new(ChildDuplex { input, output }), proxy))
}

impl Drop for CodexAppServer {
    fn drop(&mut self) {
        self.reader.abort();
        if let Some(proxy) = self.proxy.as_mut() {
            let _ = proxy.start_kill();
        }
    }
}

fn read_responses(mut output: SplitStream<CodexSocket>) -> (mpsc::Receiver<io::Result<Value>>, JoinHandle<()>) {
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

fn active_turn_id_from_page(page: &Value) -> Option<&str> {
    page.get("data")?
        .as_array()?
        .iter()
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

fn codex_program() -> PathBuf {
    env::var_os("YONDER_CODEX_BIN")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"))
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
    run_bound(agent_id, yonder_socket, thread_id, codex, false).await
}

pub async fn run_managed(
    agent_id: String,
    yonder_socket: PathBuf,
    thread_id: String,
) -> io::Result<()> {
    if !yonder_protocol::valid_id(&thread_id) {
        return Err(io::Error::other("Codex Hook session_id无效"));
    }
    let mut codex = CodexAppServer::connect_managed().await?;
    codex.resume(&thread_id).await?;
    run_bound(agent_id, yonder_socket, thread_id, codex, true).await
}

async fn connect_yonder(
    agent_id: &str,
    yonder_socket: &Path,
    thread_id: &str,
) -> io::Result<Stream> {
    let stream = Stream::connect(yonder_socket.to_fs_name::<GenericFilePath>()?)
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Yonder未运行"))?;
    let hello = Request::Hello {
        jsonrpc: Version::V2,
        request_id: "hello".into(),
        params: HelloParams {
            agent_id: agent_id.into(),
            capability: Capability::TaskRead,
            deadline: now_ms()?.saturating_add(10_000),
            protocol_version: ProtocolVersion {
                major: PROTOCOL_VERSION.major,
                // 有意钉定 1.33：桥只使用 agent.input 与该版本新增的 replan-input，
                // 不得随产品协议升版而声明尚未实现的新能力面。
                minor: 33,
            },
            session_id: Some(thread_id.into()),
            offered_capabilities: Some(vec![
                OfferedCapability::UserInput,
                OfferedCapability::ReplanInput,
            ]),
        },
    };
    send_yonder(&stream, &hello).await?;
    let mut reader = BufReader::new(&stream);
    match serde_json::from_slice::<Response>(&read_yonder(&mut reader).await?) {
        Ok(Response::Success { .. }) => Ok(stream),
        Ok(Response::Failure { error, .. }) => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            error.message,
        )),
        Err(_) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Yonder握手响应无效",
        )),
    }
}

async fn serve_yonder(
    stream: &Stream,
    codex: &mut CodexAppServer,
    thread_id: &str,
    dedup: &mut DedupResults,
) -> io::Result<()> {
    let mut reader = BufReader::new(stream);
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
        let accepted = handle_input(codex, thread_id, &params, dedup)
            .await
            .map_err(|_| io::Error::other("Codex输入投递未确认"))?;
        send_yonder(
            stream,
            &AgentResponse::Success {
                jsonrpc: Version::V2,
                id: request_id,
                result: AgentInputResult { accepted },
            },
        )
        .await?;
    }
}

fn reconnectable(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::NotFound
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::UnexpectedEof
            | io::ErrorKind::TimedOut
    )
}

fn next_backoff(current: Duration) -> Duration {
    current.saturating_mul(2).min(YONDER_RECONNECT_MAX)
}

async fn run_bound(
    agent_id: String,
    yonder_socket: PathBuf,
    thread_id: String,
    mut codex: CodexAppServer,
    reconnect: bool,
) -> io::Result<()> {
    let mut dedup = DedupResults::new();
    loop {
        let recovery_deadline = Instant::now() + YONDER_RECONNECT_WINDOW;
        let mut backoff = YONDER_RECONNECT_INITIAL;
        let stream = loop {
            match connect_yonder(&agent_id, &yonder_socket, &thread_id).await {
                Ok(stream) => break stream,
                Err(error)
                    if reconnect
                        && reconnectable(&error)
                        && Instant::now() + backoff <= recovery_deadline =>
                {
                    tokio::time::sleep(backoff).await;
                    backoff = next_backoff(backoff);
                }
                Err(error) => return Err(error),
            }
        };
        match serve_yonder(&stream, &mut codex, &thread_id, &mut dedup).await {
            Err(error) if reconnect && reconnectable(&error) => continue,
            result => return result,
        }
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

    #[cfg(unix)]
    fn fake_codex() -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let directory = env::temp_dir().join(format!(
            "yonder-codex-proxy-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let script = directory.join("codex-fixture");
        std::fs::write(
            &script,
            br##"#!/bin/sh
if [ "$1" = "app-server" ] && [ "$2" = "daemon" ] && [ "$3" = "start" ] && [ -z "$4" ]; then exit 0; fi
if [ "$1" != "app-server" ] || [ "$2" != "proxy" ] || [ -n "$3" ]; then exit 2; fi
IFS= read -r line
[ "$line" = "client-ping" ] || exit 3
printf '%s\n' 'server-pong'
"##,
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        script
    }

    async fn fake_app_server() -> CodexAppServer {
        let (client, server) = tokio::io::duplex(64 * 1024);
        tokio::spawn(async move {
            let mut socket = tokio_tungstenite::accept_async(server).await.unwrap();
            while let Some(Ok(Message::Text(text))) = socket.next().await {
                let request: Value = serde_json::from_str(text.as_str()).unwrap();
                let Some(id) = request.get("id").and_then(Value::as_u64) else {
                    continue;
                };
                let result = match request.get("method").and_then(Value::as_str) {
                    Some("initialize") => json!({}),
                    Some("thread/resume") => json!({"thread":{"id":"thread-1"}}),
                    Some("thread/read") => json!({"thread":{"status":{"type":"idle"}}}),
                    Some("turn/start") => json!({"turn":{"id":"turn-1"}}),
                    method => panic!("unexpected method: {method:?}"),
                };
                socket
                    .send(Message::Text(json!({"id":id,"result":result}).to_string().into()))
                    .await
                    .unwrap();
            }
        });
        CodexAppServer::connect_transport(Box::new(client), None)
            .await
            .unwrap()
    }

    #[test]
    fn selects_only_the_in_progress_turn() {
        assert_eq!(
            active_turn_id_from_page(&json!({"data":[{"id":"done","status":"interrupted"}]})),
            None
        );
        assert_eq!(
            active_turn_id_from_page(&json!({"data":[{"id":"active","status":"inProgress","items":[]}]})),
            Some("active")
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

    #[test]
    fn reconnect_backoff_is_bounded_and_protocol_errors_are_terminal() {
        assert_eq!(next_backoff(Duration::from_millis(100)), Duration::from_millis(200));
        assert_eq!(next_backoff(Duration::from_secs(2)), YONDER_RECONNECT_MAX);
        assert!(reconnectable(&io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "closed"
        )));
        assert!(!reconnectable(&io::Error::new(
            io::ErrorKind::PermissionDenied,
            "revoked"
        )));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn managed_proxy_exposes_a_bidirectional_raw_stream() {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

        let script = fake_codex();
        let directory = script.parent().unwrap().to_path_buf();
        let (mut transport, mut proxy) = spawn_managed_proxy(&script).await.unwrap();
        transport.write_all(b"client-ping\n").await.unwrap();
        transport.flush().await.unwrap();
        let mut response = String::new();
        BufReader::new(&mut transport)
            .read_line(&mut response)
            .await
            .unwrap();
        assert_eq!(response, "server-pong\n");
        proxy.wait().await.unwrap();
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[tokio::test]
    async fn websocket_app_server_resumes_the_explicit_thread_and_starts_an_idle_turn() {
        let mut server = fake_app_server().await;
        server.resume("thread-1").await.unwrap();
        server
            .deliver("thread-1", "bounded test input", Duration::from_secs(1))
            .await
            .unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn managed_bridge_registers_again_after_yonder_restarts() {
        use interprocess::local_socket::ListenerOptions;

        let socket = PathBuf::from(format!(
            "/tmp/ya8-{}-{}.sock",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let listener = ListenerOptions::new()
            .name(socket.clone().to_fs_name::<GenericFilePath>().unwrap())
            .try_overwrite(true)
            .create_tokio()
            .unwrap();
        let (registered, mut registrations) = mpsc::channel(2);
        let gateway = tokio::spawn(async move {
            for _ in 0..2 {
                let stream = listener.accept().await.unwrap();
                let mut reader = BufReader::new(&stream);
                let hello = read_yonder(&mut reader).await.unwrap();
                let request: Request = serde_json::from_slice(&hello).unwrap();
                assert!(matches!(
                    request,
                    Request::Hello {
                        params: HelloParams { session_id: Some(ref session), .. },
                        ..
                    } if session == "thread-1"
                ));
                (&stream)
                    .write_all(
                        b"{\"jsonrpc\":\"2.0\",\"id\":\"hello\",\"result\":{\"kind\":\"hello\",\"protocol_version\":{\"major\":1,\"minor\":33},\"platform\":\"macos\",\"capabilities\":[]}}\n",
                    )
                    .await
                    .unwrap();
                registered.send(()).await.unwrap();
            }
        });
        let mut server = fake_app_server().await;
        server.resume("thread-1").await.unwrap();
        let bridge = tokio::spawn(run_bound(
            "codex".into(),
            socket.clone(),
            "thread-1".into(),
            server,
            true,
        ));
        tokio::time::timeout(Duration::from_secs(3), registrations.recv())
            .await
            .unwrap()
            .unwrap();
        tokio::time::timeout(Duration::from_secs(3), registrations.recv())
            .await
            .unwrap()
            .unwrap();
        gateway.await.unwrap();
        bridge.abort();
        let _ = bridge.await;
        let _ = std::fs::remove_file(socket);
    }
}
