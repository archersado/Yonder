use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use interprocess::local_socket::{
    GenericFilePath, ToFsName,
    tokio::{Stream, prelude::*},
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, VecDeque},
    env, io,
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
    task::{Context, Poll},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, ReadBuf},
    net::UnixStream,
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::mpsc,
    task::JoinHandle,
};
use tokio_tungstenite::{WebSocketStream, client_async, tungstenite::Message};
use yonder_protocol::{
    AgentAttachmentBeginParams, AgentAttachmentChunkParams, AgentAttachmentFinishParams,
    AgentAttachmentMime, AgentInputParams, AgentInputResult, AgentNotification, AgentReplyMode,
    AgentReplyParams, AgentReplyPhase, AgentRequest, AgentResponse, AgentSessionCloseReason,
    Capability, HelloParams,
    MAX_AGENT_ATTACHMENT_BYTES, MAX_AGENT_REPLY_BYTES, MAX_AGENT_REPLY_DELTA_BYTES,
    MAX_REQUEST_BYTES, OfferedCapability, PROTOCOL_VERSION, ProtocolVersion, Request, Response,
    Version,
};

const CODEX_RPC_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_CODEX_RESPONSE_FRAME_BYTES: usize = 1024 * 1024;
const MAX_CODEX_REQUEST_FRAME_BYTES: usize = 6 * 1024 * 1024;
const MAX_DEDUP_RESULTS: usize = 1024;
const MAX_PENDING_RESPONSES: usize = 8;
const AGENT_REPLY_TIMEOUT: Duration = Duration::from_secs(300);
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

struct PendingAttachment {
    id: String,
    session_id: String,
    mime: AgentAttachmentMime,
    byte_length: usize,
    sha256: String,
    deadline: u64,
    next_sequence: u16,
    bytes: Vec<u8>,
}

struct ReadyAttachment {
    id: String,
    mime: AgentAttachmentMime,
    deadline: u64,
    bytes: Vec<u8>,
}

enum AttachmentState {
    Empty,
    Receiving(PendingAttachment),
    Ready(ReadyAttachment),
    Rejected { id: String, deadline: u64 },
}

struct AttachmentInbox {
    thread_id: String,
    state: AttachmentState,
}

impl AttachmentInbox {
    fn new(thread_id: &str) -> Self {
        Self {
            thread_id: thread_id.into(),
            state: AttachmentState::Empty,
        }
    }

    fn begin(&mut self, params: AgentAttachmentBeginParams, now: u64) {
        let valid = params.validate(now).is_ok()
            && params.session_id == self.thread_id
            && matches!(self.state, AttachmentState::Empty);
        if valid {
            let byte_length = params.byte_length as usize;
            self.state = AttachmentState::Receiving(PendingAttachment {
                id: params.attachment_id,
                session_id: params.session_id,
                mime: params.mime,
                byte_length,
                sha256: params.sha256.to_ascii_lowercase(),
                deadline: params.deadline,
                next_sequence: 0,
                bytes: Vec::with_capacity(byte_length),
            });
        } else {
            self.state = AttachmentState::Rejected {
                id: params.attachment_id,
                deadline: params.deadline,
            };
        }
    }

    fn chunk(&mut self, params: AgentAttachmentChunkParams) {
        let AttachmentState::Receiving(pending) = &mut self.state else {
            return;
        };
        let decoded = params
            .validate()
            .ok()
            .and_then(|_| BASE64.decode(&params.data_base64).ok());
        let valid = params.attachment_id == pending.id
            && params.session_id == pending.session_id
            && params.sequence == pending.next_sequence
            && decoded.as_ref().is_some_and(|bytes| {
                pending.bytes.len().saturating_add(bytes.len()) <= pending.byte_length
                    && pending.bytes.len().saturating_add(bytes.len())
                        <= MAX_AGENT_ATTACHMENT_BYTES as usize
            });
        if valid {
            pending
                .bytes
                .extend_from_slice(decoded.as_deref().unwrap_or_default());
            pending.next_sequence = pending.next_sequence.saturating_add(1);
        } else {
            let id = pending.id.clone();
            let deadline = pending.deadline;
            self.state = AttachmentState::Rejected { id, deadline };
        }
    }

    fn finish(&mut self, params: &AgentAttachmentFinishParams, now: u64) -> bool {
        if params.validate().is_err() {
            self.state = AttachmentState::Empty;
            return false;
        }
        let state = std::mem::replace(&mut self.state, AttachmentState::Empty);
        let AttachmentState::Receiving(pending) = state else {
            return false;
        };
        let valid = params.attachment_id == pending.id
            && params.session_id == pending.session_id
            && pending.session_id == self.thread_id
            && pending.deadline > now
            && pending.bytes.len() == pending.byte_length
            && format!("{:x}", Sha256::digest(&pending.bytes)) == pending.sha256;
        if !valid {
            return false;
        }
        self.state = AttachmentState::Ready(ReadyAttachment {
            id: pending.id,
            mime: pending.mime,
            deadline: pending.deadline,
            bytes: pending.bytes,
        });
        true
    }

    fn take(&mut self, attachment_id: &str, now: u64) -> Option<ReadyAttachment> {
        let state = std::mem::replace(&mut self.state, AttachmentState::Empty);
        match state {
            AttachmentState::Ready(attachment)
                if attachment.id == attachment_id && attachment.deadline > now =>
            {
                Some(attachment)
            }
            _ => None,
        }
    }

    fn discard(&mut self, attachment_id: Option<&str>) {
        let matches = match (&self.state, attachment_id) {
            (_, None) => true,
            (AttachmentState::Receiving(value), Some(id)) => value.id == id,
            (AttachmentState::Ready(value), Some(id)) => value.id == id,
            (AttachmentState::Rejected { id: value, .. }, Some(id)) => value == id,
            (AttachmentState::Empty, Some(_)) => false,
        };
        if matches {
            self.state = AttachmentState::Empty;
        }
    }

    fn expiry_delay(&self, now: u64) -> Option<Duration> {
        let deadline = match &self.state {
            AttachmentState::Receiving(value) => value.deadline,
            AttachmentState::Ready(value) => value.deadline,
            AttachmentState::Rejected { deadline, .. } => *deadline,
            AttachmentState::Empty => return None,
        };
        Some(Duration::from_millis(deadline.saturating_sub(now)))
    }
}

impl ReadyAttachment {
    fn data_url(&self) -> String {
        let mime = match self.mime {
            AgentAttachmentMime::ImagePng => "image/png",
            AgentAttachmentMime::ImageJpeg => "image/jpeg",
            AgentAttachmentMime::ImageWebp => "image/webp",
        };
        format!("data:{mime};base64,{}", BASE64.encode(&self.bytes))
    }
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
    notifications: mpsc::UnboundedReceiver<Value>,
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
        let (responses, notifications, reader) = read_responses(output);
        let mut server = Self {
            input,
            responses,
            notifications,
            reader,
            proxy,
            next_id: 0,
        };
        server
            .initialize()
            .await
            .map_err(|_| io::Error::other("Codex App Server初始化失败"))?;
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
        if bytes.len() > MAX_CODEX_REQUEST_FRAME_BYTES {
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
        if self.is_loaded(thread_id).await? {
            Ok(())
        } else {
            Err(io::Error::other("Codex thread未在指定App Server中加载"))
        }
    }

    async fn is_loaded(&mut self, thread_id: &str) -> io::Result<bool> {
        let result = self
            .call("thread/loaded/list", json!({}), CODEX_RPC_TIMEOUT)
            .await
            .map_err(|_| io::Error::other("Codex App Server已加载thread查询失败"))?;
        Ok(result
            .get("data")
            .and_then(Value::as_array)
            .is_some_and(|threads| {
                threads
                    .iter()
                    .any(|value| value.as_str() == Some(thread_id))
            }))
    }

    async fn attach_or_resume(&mut self, thread_id: &str) -> io::Result<()> {
        // `thread/loaded/list` 只表示共享 daemon 持有该 thread，不表示
        // 当前 App Server 客户端已订阅 item/turn 通知。无论是否已加载，
        // 都必须经同一共享 daemon/proxy 执行 resume 才能回传 Agent 回复。
        let _ = self.is_loaded(thread_id).await?;
        self.resume(thread_id).await
    }

    async fn resume(&mut self, thread_id: &str) -> io::Result<()> {
        let result = self
            .call(
                "thread/resume",
                json!({"threadId":thread_id,"excludeTurns":true}),
                CODEX_RPC_TIMEOUT,
            )
            .await
            .map_err(|_| io::Error::other("Codex App Server thread恢复失败"))?;
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
        attachment: Option<&ReadyAttachment>,
        timeout: Duration,
    ) -> io::Result<String> {
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
        let mut input = vec![json!({"type":"text","text":content})];
        if let Some(attachment) = attachment {
            input.push(json!({"type":"image","url":attachment.data_url(),"detail":"auto"}));
        }
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
                    Ok(turn_id.to_owned())
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
                if let Some(turn_id) = result
                    .get("turn")
                    .and_then(|turn| turn.get("id"))
                    .and_then(Value::as_str)
                {
                    Ok(turn_id.to_owned())
                } else {
                    Err(io::Error::other("Codex start确认无效"))
                }
            }
            "notLoaded" => Err(io::Error::other("Codex thread已卸载")),
            _ => Err(io::Error::other("Codex thread当前不可接收输入")),
        }
    }

    async fn next_notification(&mut self) -> io::Result<Value> {
        self.notifications.recv().await.ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "Codex App Server通知流已断开")
        })
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
    .map_err(|error| {
        io::Error::new(io::ErrorKind::NotFound, format!("Codex CLI不可用：{error}"))
    })?;
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

fn read_responses(
    mut output: SplitStream<CodexSocket>,
) -> (
    mpsc::Receiver<io::Result<Value>>,
    mpsc::UnboundedReceiver<Value>,
    JoinHandle<()>,
) {
    let (sender, receiver) = mpsc::channel(MAX_PENDING_RESPONSES);
    let (notification_sender, notification_receiver) = mpsc::unbounded_channel();
    let reader = tokio::spawn(async move {
        loop {
            let result: io::Result<Value> = match output.next().await {
                None | Some(Ok(Message::Close(_))) => Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Codex App Server已断开",
                )),
                Some(Ok(Message::Text(text))) if text.len() <= MAX_CODEX_RESPONSE_FRAME_BYTES => {
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
            if let Ok(message) = result.as_ref() {
                if matches!(
                    message.get("method").and_then(Value::as_str),
                    Some(
                        "item/started"
                            | "item/agentMessage/delta"
                            | "item/completed"
                            | "turn/completed"
                    )
                ) {
                    let _ = notification_sender.send(message.clone());
                    continue;
                }
            }
            let terminal = result.is_err();
            if sender.send(result).await.is_err() || terminal {
                break;
            }
        }
    });
    (receiver, notification_receiver, reader)
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

async fn read_yonder_handshake(stream: &Stream) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        let read = (&*stream).read(&mut byte).await?;
        if read == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Yonder已断开"));
        }
        if byte[0] == b'\n' {
            return Ok(bytes);
        }
        bytes.push(byte[0]);
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Yonder Agent帧无效",
            ));
        }
    }
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
    codex.attach_or_resume(&thread_id).await?;
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
                // Bridge 与同一发布包内的 Desktop 共享协议版本；终局核验输入自
                // 1.53 起可用，不能继续钉在旧版本，否则片段执行完只会留在等待态。
                minor: PROTOCOL_VERSION.minor,
            },
            session_id: Some(thread_id.into()),
            offered_capabilities: Some(vec![
                OfferedCapability::UserInput,
                OfferedCapability::UserInputAttachment,
                OfferedCapability::ReplanInput,
                OfferedCapability::UserInputReply,
            ]),
        },
    };
    send_yonder(&stream, &hello).await?;
    match serde_json::from_slice::<Response>(&read_yonder_handshake(&stream).await?) {
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

struct PendingReply {
    input_id: String,
    session_id: String,
    turn_id: String,
    next_sequence: u32,
    visible_bytes: usize,
    truncated: bool,
    items: HashMap<String, bool>,
    deadline: Instant,
}

impl PendingReply {
    fn params(&mut self, phase: AgentReplyPhase, content: Option<String>) -> AgentReplyParams {
        let params = AgentReplyParams {
            input_id: self.input_id.clone(),
            session_id: self.session_id.clone(),
            reply_id: self.turn_id.clone(),
            sequence: self.next_sequence,
            phase,
            content,
            truncated: phase == AgentReplyPhase::Completed && self.truncated,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        params
    }
}

async fn send_reply(
    stream: &Stream,
    pending: &mut PendingReply,
    phase: AgentReplyPhase,
    content: Option<String>,
) -> io::Result<()> {
    let notification = AgentNotification::Reply {
        jsonrpc: Version::V2,
        params: pending.params(phase, content),
    };
    send_yonder(stream, &notification).await
}

fn utf8_prefix(value: &str, limit: usize) -> usize {
    let mut end = value.len().min(limit);
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }
    end
}

async fn forward_reply_text(
    stream: &Stream,
    pending: &mut PendingReply,
    mut text: &str,
) -> io::Result<()> {
    while !text.is_empty() {
        let remaining = MAX_AGENT_REPLY_BYTES.saturating_sub(pending.visible_bytes);
        if remaining == 0 {
            pending.truncated = true;
            break;
        }
        let end = utf8_prefix(text, MAX_AGENT_REPLY_DELTA_BYTES.min(remaining));
        if end == 0 {
            pending.truncated = true;
            break;
        }
        let chunk = &text[..end];
        send_reply(
            stream,
            pending,
            AgentReplyPhase::Delta,
            Some(chunk.to_owned()),
        )
        .await?;
        pending.visible_bytes += chunk.len();
        text = &text[end..];
    }
    Ok(())
}

fn final_agent_message(item: &Value) -> bool {
    item.get("type").and_then(Value::as_str) == Some("agentMessage")
        && matches!(
            item.get("phase").and_then(Value::as_str),
            None | Some("final_answer")
        )
}

async fn handle_codex_notification(
    stream: &Stream,
    pending: &mut PendingReply,
    message: &Value,
) -> io::Result<bool> {
    let method = message
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let params = message.get("params").unwrap_or(&Value::Null);
    if params.get("threadId").and_then(Value::as_str) != Some(pending.session_id.as_str()) {
        return Ok(false);
    }
    match method {
        "item/started" => {
            if params.get("turnId").and_then(Value::as_str) == Some(pending.turn_id.as_str()) {
                if let Some(item) = params.get("item").filter(|item| final_agent_message(item)) {
                    if let Some(item_id) = item
                        .get("id")
                        .and_then(Value::as_str)
                        .filter(|id| yonder_protocol::valid_id(id))
                    {
                        pending.items.entry(item_id.to_owned()).or_insert(false);
                    }
                }
            }
        }
        "item/agentMessage/delta" => {
            if params.get("turnId").and_then(Value::as_str) == Some(pending.turn_id.as_str()) {
                if let Some(item_id) = params.get("itemId").and_then(Value::as_str) {
                    if pending.items.contains_key(item_id) {
                        if let Some(delta) = params.get("delta").and_then(Value::as_str) {
                            forward_reply_text(stream, pending, delta).await?;
                        }
                        if let Some(seen) = pending.items.get_mut(item_id) {
                            *seen = true;
                        }
                    }
                }
            }
        }
        "item/completed" => {
            if params.get("turnId").and_then(Value::as_str) == Some(pending.turn_id.as_str()) {
                if let Some(item) = params.get("item").filter(|item| final_agent_message(item)) {
                    if let Some(item_id) = item
                        .get("id")
                        .and_then(Value::as_str)
                        .filter(|id| yonder_protocol::valid_id(id))
                    {
                        let had_delta = *pending.items.entry(item_id.to_owned()).or_insert(false);
                        if !had_delta {
                            if let Some(text) = item.get("text").and_then(Value::as_str) {
                                forward_reply_text(stream, pending, text).await?;
                            }
                        }
                    }
                }
            }
        }
        "turn/completed" => {
            let turn = params.get("turn").unwrap_or(&Value::Null);
            if turn.get("id").and_then(Value::as_str) == Some(pending.turn_id.as_str()) {
                let phase = match turn.get("status").and_then(Value::as_str) {
                    Some("completed") => AgentReplyPhase::Completed,
                    Some("failed") => AgentReplyPhase::Failed,
                    _ => AgentReplyPhase::Interrupted,
                };
                send_reply(stream, pending, phase, None).await?;
                return Ok(true);
            }
        }
        _ => {}
    }
    Ok(false)
}

async fn read_yonder_with_timeout(
    reader: &mut BufReader<&Stream>,
    delay: Option<Duration>,
) -> io::Result<Option<Vec<u8>>> {
    match delay {
        Some(delay) => match tokio::time::timeout(delay, read_yonder(reader)).await {
            Ok(result) => result.map(Some),
            Err(_) => Ok(None),
        },
        None => read_yonder(reader).await.map(Some),
    }
}

async fn serve_yonder(
    stream: &Stream,
    codex: &mut CodexAppServer,
    thread_id: &str,
    dedup: &mut DedupResults,
) -> io::Result<()> {
    let mut reader = BufReader::new(stream);
    let mut attachments = AttachmentInbox::new(thread_id);
    let mut pending_reply: Option<PendingReply> = None;
    loop {
        enum Incoming {
            Yonder(Option<Vec<u8>>),
            Codex(Value),
        }
        let delay = attachments.expiry_delay(now_ms()?);
        let reply_delay = pending_reply
            .as_ref()
            .map(|reply| reply.deadline.saturating_duration_since(Instant::now()));
        let incoming = if pending_reply.is_some() {
            tokio::select! {
                value=read_yonder_with_timeout(&mut reader,delay)=>Incoming::Yonder(value?),
                value=codex.next_notification()=>Incoming::Codex(value?),
                _=tokio::time::sleep(reply_delay.unwrap_or_default())=>{
                    if let Some(pending)=pending_reply.as_mut(){send_reply(stream,pending,AgentReplyPhase::Interrupted,None).await?;}
                    pending_reply=None;
                    continue;
                }
            }
        } else {
            tokio::select! {
                value=read_yonder_with_timeout(&mut reader,delay)=>Incoming::Yonder(value?),
                value=codex.next_notification()=>Incoming::Codex(value?),
            }
        };
        let bytes = match incoming {
            Incoming::Codex(message) => {
                if let Some(pending) = pending_reply.as_mut() {
                    if handle_codex_notification(stream, pending, &message).await? {
                        pending_reply = None;
                    }
                }
                continue;
            }
            Incoming::Yonder(None) => {
                attachments.discard(None);
                continue;
            }
            Incoming::Yonder(Some(bytes)) => bytes,
        };
        let request = serde_json::from_slice::<AgentRequest>(&bytes)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Yonder Agent请求无效"))?;
        match request {
            AgentRequest::SessionClose { params, .. } => {
                params
                    .validate()
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Yonder Agent会话关闭通知无效"))?;
                if params.session_id != thread_id {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Yonder Agent会话关闭目标不匹配",
                    ));
                }
                match params.reason {
                    AgentSessionCloseReason::Preempted => {
                        return Err(io::Error::new(
                            io::ErrorKind::ConnectionAborted,
                            "Agent输入会话已被抢占",
                        ));
                    }
                }
            }
            AgentRequest::AttachmentBegin { params, .. } => attachments.begin(params, now_ms()?),
            AgentRequest::AttachmentChunk { params, .. } => attachments.chunk(params),
            AgentRequest::AttachmentFinish {
                request_id, params, ..
            } => {
                let accepted = attachments.finish(&params, now_ms()?);
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
            AgentRequest::Input {
                request_id, params, ..
            } => {
                let reply_requested = params.reply_mode == Some(AgentReplyMode::Stream);
                let (accepted, turn_id) = if reply_requested && pending_reply.is_some() {
                    attachments.discard(params.attachment_id.as_deref());
                    (false, None)
                } else {
                    handle_input(codex, thread_id, &params, dedup, &mut attachments)
                        .await
                        .map_err(|_| io::Error::other("Codex输入投递未确认"))?
                };
                send_yonder(
                    stream,
                    &AgentResponse::Success {
                        jsonrpc: Version::V2,
                        id: request_id,
                        result: AgentInputResult { accepted },
                    },
                )
                .await?;
                if accepted && reply_requested {
                    if let Some(turn_id) = turn_id {
                        let mut pending = PendingReply {
                            input_id: params.input_id,
                            session_id: params.session_id,
                            turn_id,
                            next_sequence: 1,
                            visible_bytes: 0,
                            truncated: false,
                            items: HashMap::new(),
                            deadline: Instant::now() + AGENT_REPLY_TIMEOUT,
                        };
                        send_reply(stream, &mut pending, AgentReplyPhase::Started, None).await?;
                        pending_reply = Some(pending);
                    }
                }
            }
        }
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
    attachments: &mut AttachmentInbox,
) -> io::Result<(bool, Option<String>)> {
    if let Some(accepted) = dedup.get(&params.input_id) {
        return Ok((accepted, None));
    }
    let now = now_ms()?;
    if params.session_id != thread_id || params.validate(now).is_err() {
        attachments.discard(params.attachment_id.as_deref());
        dedup.insert(params.input_id.clone(), false);
        return Ok((false, None));
    }
    let attachment = match params.attachment_id.as_deref() {
        Some(attachment_id) => match attachments.take(attachment_id, now) {
            Some(attachment) => Some(attachment),
            None => {
                dedup.insert(params.input_id.clone(), false);
                return Ok((false, None));
            }
        },
        None => None,
    };
    let timeout = Duration::from_millis(params.deadline.saturating_sub(now)).min(CODEX_RPC_TIMEOUT);
    match codex
        .deliver(thread_id, &params.content, attachment.as_ref(), timeout)
        .await
    {
        Ok(turn_id) => {
            dedup.insert(params.input_id.clone(), true);
            Ok((true, Some(turn_id)))
        }
        Err(error) if error.kind() == io::ErrorKind::InvalidInput => {
            dedup.insert(params.input_id.clone(), false);
            Ok((false, None))
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

    async fn fake_app_server_with_loaded_state(
        loaded: bool,
    ) -> (CodexAppServer, mpsc::UnboundedReceiver<Value>) {
        let (client, server) = tokio::io::duplex(64 * 1024);
        let (calls, called) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut socket = tokio_tungstenite::accept_async(server).await.unwrap();
            while let Some(Ok(Message::Text(text))) = socket.next().await {
                let request: Value = serde_json::from_str(text.as_str()).unwrap();
                let _ = calls.send(request.clone());
                let Some(id) = request.get("id").and_then(Value::as_u64) else {
                    continue;
                };
                let result = match request.get("method").and_then(Value::as_str) {
                    Some("initialize") => json!({}),
                    Some("thread/loaded/list") => {
                        if loaded {
                            json!({"data":["thread-1"]})
                        } else {
                            json!({"data":[]})
                        }
                    }
                    Some("thread/resume") => json!({"thread":{"id":"thread-1"}}),
                    Some("thread/read") => json!({"thread":{"status":{"type":"idle"}}}),
                    Some("turn/start") => json!({"turn":{"id":"turn-1"}}),
                    method => panic!("unexpected method: {method:?}"),
                };
                socket
                    .send(Message::Text(
                        json!({"id":id,"result":result}).to_string().into(),
                    ))
                    .await
                    .unwrap();
            }
        });
        (
            CodexAppServer::connect_transport(Box::new(client), None)
                .await
                .unwrap(),
            called,
        )
    }

    async fn fake_app_server_with_calls() -> (CodexAppServer, mpsc::UnboundedReceiver<Value>) {
        fake_app_server_with_loaded_state(true).await
    }

    async fn fake_app_server() -> CodexAppServer {
        fake_app_server_with_calls().await.0
    }

    #[test]
    fn selects_only_the_in_progress_turn() {
        assert_eq!(
            active_turn_id_from_page(&json!({"data":[{"id":"done","status":"interrupted"}]})),
            None
        );
        assert_eq!(
            active_turn_id_from_page(
                &json!({"data":[{"id":"active","status":"inProgress","items":[]}]})
            ),
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
        assert_eq!(
            next_backoff(Duration::from_millis(100)),
            Duration::from_millis(200)
        );
        assert_eq!(next_backoff(Duration::from_secs(2)), YONDER_RECONNECT_MAX);
        assert!(reconnectable(&io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "closed"
        )));
        assert!(!reconnectable(&io::Error::new(
            io::ErrorKind::PermissionDenied,
            "revoked"
        )));
        assert!(!reconnectable(&io::Error::new(
            io::ErrorKind::ConnectionAborted,
            "preempted"
        )));
    }

    #[test]
    fn reply_chunk_boundary_preserves_utf8() {
        assert_eq!(utf8_prefix("你a", 1), 0);
        assert_eq!(utf8_prefix("你a", 3), 3);
        assert_eq!(utf8_prefix("你a", 4), 4);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn codex_notifications_become_ordered_agent_reply_frames() {
        use interprocess::local_socket::ListenerOptions;

        let socket = PathBuf::from(format!(
            "/tmp/yar-{}-{}.sock",
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
        let (client, server) = tokio::join!(
            Stream::connect(socket.clone().to_fs_name::<GenericFilePath>().unwrap()),
            listener.accept()
        );
        let client = client.unwrap();
        let server = server.unwrap();
        let mut pending = PendingReply {
            input_id: "input-1".into(),
            session_id: "thread-1".into(),
            turn_id: "turn-1".into(),
            next_sequence: 1,
            visible_bytes: 0,
            truncated: false,
            items: HashMap::new(),
            deadline: Instant::now() + AGENT_REPLY_TIMEOUT,
        };
        assert!(!handle_codex_notification(&client,&mut pending,&json!({"method":"item/started","params":{"threadId":"thread-1","turnId":"turn-1","item":{"id":"item-1","type":"agentMessage","phase":"final_answer"}}})).await.unwrap());
        assert!(!handle_codex_notification(&client,&mut pending,&json!({"method":"item/agentMessage/delta","params":{"threadId":"thread-1","turnId":"turn-1","itemId":"item-1","delta":"统一回复"}})).await.unwrap());
        assert!(handle_codex_notification(&client,&mut pending,&json!({"method":"turn/completed","params":{"threadId":"thread-1","turn":{"id":"turn-1","status":"completed"}}})).await.unwrap());
        let mut reader = BufReader::new(&server);
        let first =
            yonder_protocol::decode_agent_notification(&read_yonder(&mut reader).await.unwrap())
                .unwrap();
        let second =
            yonder_protocol::decode_agent_notification(&read_yonder(&mut reader).await.unwrap())
                .unwrap();
        let AgentNotification::Reply { params: first, .. } = first;
        let AgentNotification::Reply { params: second, .. } = second;
        assert_eq!(
            (first.sequence, first.phase, first.content),
            (1, AgentReplyPhase::Delta, Some("统一回复".into()))
        );
        assert_eq!(
            (second.sequence, second.phase, second.content),
            (2, AgentReplyPhase::Completed, None)
        );
        drop((client, server, listener));
        let _ = std::fs::remove_file(socket);
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
            .deliver(
                "thread-1",
                "bounded test input",
                None,
                Duration::from_secs(1),
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn managed_app_server_subscribes_an_already_loaded_thread() {
        let (mut server, mut calls) = fake_app_server_with_loaded_state(true).await;
        server.attach_or_resume("thread-1").await.unwrap();
        assert_eq!(calls.recv().await.unwrap()["method"], "initialize");
        assert_eq!(calls.recv().await.unwrap()["method"], "initialized");
        assert_eq!(calls.recv().await.unwrap()["method"], "thread/loaded/list");
        assert_eq!(calls.recv().await.unwrap()["method"], "thread/resume");
    }

    #[tokio::test]
    async fn managed_app_server_resumes_an_unloaded_thread() {
        let (mut server, mut calls) = fake_app_server_with_loaded_state(false).await;
        server.attach_or_resume("thread-1").await.unwrap();
        assert_eq!(calls.recv().await.unwrap()["method"], "initialize");
        assert_eq!(calls.recv().await.unwrap()["method"], "initialized");
        assert_eq!(calls.recv().await.unwrap()["method"], "thread/loaded/list");
        assert_eq!(calls.recv().await.unwrap()["method"], "thread/resume");
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
                        params: HelloParams {
                            protocol_version: ProtocolVersion { major: 1, minor: 53 },
                            session_id: Some(ref session),
                            offered_capabilities: Some(ref capabilities),
                            ..
                        },
                        ..
                    } if session == "thread-1"
                        && capabilities.contains(&OfferedCapability::UserInput)
                        && capabilities.contains(&OfferedCapability::UserInputAttachment)
                        && capabilities.contains(&OfferedCapability::ReplanInput)
                        && capabilities.contains(&OfferedCapability::UserInputReply)
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

    #[cfg(unix)]
    #[tokio::test]
    async fn preempted_managed_bridge_exits_without_registering_again() {
        use interprocess::local_socket::ListenerOptions;

        let socket = PathBuf::from(format!(
            "/tmp/yap-{}-{}.sock",
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
        let mut codex = fake_app_server().await;
        codex.resume("thread-1").await.unwrap();
        let bridge = tokio::spawn(run_bound(
            "codex".into(),
            socket.clone(),
            "thread-1".into(),
            codex,
            true,
        ));
        let stream = listener.accept().await.unwrap();
        let mut reader = BufReader::new(&stream);
        let _: Request = serde_json::from_slice(&read_yonder(&mut reader).await.unwrap()).unwrap();
        (&stream)
            .write_all(
                b"{\"jsonrpc\":\"2.0\",\"id\":\"hello\",\"result\":{\"kind\":\"hello\",\"protocol_version\":{\"major\":1,\"minor\":51},\"platform\":\"macos\",\"capabilities\":[]}}\n",
            )
            .await
            .unwrap();
        send_yonder(
            &stream,
            &AgentRequest::SessionClose {
                jsonrpc: Version::V2,
                params: yonder_protocol::AgentSessionCloseParams {
                    session_id: "thread-1".into(),
                    reason: AgentSessionCloseReason::Preempted,
                },
            },
        )
        .await
        .unwrap();
        let error = tokio::time::timeout(Duration::from_secs(3), bridge)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::ConnectionAborted);
        assert!(tokio::time::timeout(Duration::from_millis(250), listener.accept())
            .await
            .is_err());
        drop(reader);
        drop((stream, listener));
        let _ = std::fs::remove_file(socket);
    }

    #[tokio::test]
    async fn duplicate_input_is_acknowledged_without_a_second_codex_delivery() {
        use yonder_protocol::AgentInputSource;

        let now = now_ms().unwrap();
        let params = AgentInputParams {
            input_id: "input-same".into(),
            session_id: "thread-1".into(),
            source: AgentInputSource::Replan,
            content: "bounded replay test".into(),
            attachment_id: None,
            reply_mode: None,
            created_at: now,
            deadline: now + 10_000,
        };
        let (mut server, mut calls) = fake_app_server_with_calls().await;
        server.resume("thread-1").await.unwrap();
        let mut dedup = DedupResults::new();
        let mut attachments = AttachmentInbox::new("thread-1");
        assert!(
            handle_input(
                &mut server,
                "thread-1",
                &params,
                &mut dedup,
                &mut attachments
            )
            .await
            .unwrap()
            .0
        );
        assert!(
            handle_input(
                &mut server,
                "thread-1",
                &params,
                &mut dedup,
                &mut attachments
            )
            .await
            .unwrap()
            .0
        );
        let mut methods = Vec::new();
        while let Ok(request) = calls.try_recv() {
            if let Some(method) = request.get("method").and_then(Value::as_str) {
                methods.push(method.to_owned());
            }
        }
        assert_eq!(
            methods
                .iter()
                .filter(|method| *method == "thread/read")
                .count(),
            1
        );
        assert_eq!(
            methods
                .iter()
                .filter(|method| *method == "turn/start")
                .count(),
            1
        );
    }

    fn attachment_begin(bytes: &[u8], now: u64) -> AgentAttachmentBeginParams {
        AgentAttachmentBeginParams {
            attachment_id: "image-1".into(),
            session_id: "thread-1".into(),
            mime: AgentAttachmentMime::ImagePng,
            byte_length: bytes.len() as u32,
            sha256: format!("{:x}", Sha256::digest(bytes)),
            deadline: now + 10_000,
        }
    }

    #[test]
    fn attachment_inbox_validates_and_consumes_once() {
        let bytes = b"bounded-png-fixture";
        let now = now_ms().unwrap();
        let mut inbox = AttachmentInbox::new("thread-1");
        inbox.begin(attachment_begin(bytes, now), now);
        inbox.chunk(AgentAttachmentChunkParams {
            attachment_id: "image-1".into(),
            session_id: "thread-1".into(),
            sequence: 0,
            data_base64: BASE64.encode(bytes),
        });
        assert!(inbox.finish(
            &AgentAttachmentFinishParams {
                attachment_id: "image-1".into(),
                session_id: "thread-1".into(),
            },
            now
        ));
        let attachment = inbox.take("image-1", now).unwrap();
        assert_eq!(attachment.bytes, bytes);
        assert!(inbox.take("image-1", now).is_none());
    }

    #[test]
    fn attachment_inbox_rejects_wrong_sequence_hash_and_session() {
        let bytes = b"bounded-png-fixture";
        let now = now_ms().unwrap();
        for mutation in ["sequence", "hash", "session"] {
            let mut inbox = AttachmentInbox::new("thread-1");
            let mut begin = attachment_begin(bytes, now);
            if mutation == "hash" {
                begin.sha256 = "0".repeat(64);
            }
            if mutation == "session" {
                begin.session_id = "thread-2".into();
            }
            inbox.begin(begin, now);
            inbox.chunk(AgentAttachmentChunkParams {
                attachment_id: "image-1".into(),
                session_id: if mutation == "session" {
                    "thread-2"
                } else {
                    "thread-1"
                }
                .into(),
                sequence: if mutation == "sequence" { 1 } else { 0 },
                data_base64: BASE64.encode(bytes),
            });
            assert!(
                !inbox.finish(
                    &AgentAttachmentFinishParams {
                        attachment_id: "image-1".into(),
                        session_id: if mutation == "session" {
                            "thread-2"
                        } else {
                            "thread-1"
                        }
                        .into(),
                    },
                    now
                )
            );
            assert!(inbox.take("image-1", now).is_none());
        }
    }

    #[test]
    fn attachment_inbox_rejects_expired_or_oversized_declarations() {
        let bytes = b"bounded-png-fixture";
        let now = now_ms().unwrap();
        let mut inbox = AttachmentInbox::new("thread-1");
        let mut expired = attachment_begin(bytes, now);
        expired.deadline = now;
        inbox.begin(expired, now);
        assert!(!inbox.finish(
            &AgentAttachmentFinishParams {
                attachment_id: "image-1".into(),
                session_id: "thread-1".into(),
            },
            now
        ));

        let mut oversized = attachment_begin(bytes, now);
        oversized.byte_length = MAX_AGENT_ATTACHMENT_BYTES + 1;
        inbox.begin(oversized, now);
        assert!(!inbox.finish(
            &AgentAttachmentFinishParams {
                attachment_id: "image-1".into(),
                session_id: "thread-1".into(),
            },
            now
        ));
    }

    #[tokio::test]
    async fn attachment_is_delivered_as_same_turn_memory_image() {
        use yonder_protocol::AgentInputSource;

        let bytes = b"bounded-png-fixture";
        let now = now_ms().unwrap();
        let mut inbox = AttachmentInbox::new("thread-1");
        inbox.begin(attachment_begin(bytes, now), now);
        inbox.chunk(AgentAttachmentChunkParams {
            attachment_id: "image-1".into(),
            session_id: "thread-1".into(),
            sequence: 0,
            data_base64: BASE64.encode(bytes),
        });
        assert!(inbox.finish(
            &AgentAttachmentFinishParams {
                attachment_id: "image-1".into(),
                session_id: "thread-1".into(),
            },
            now
        ));
        let params = AgentInputParams {
            input_id: "input-image".into(),
            session_id: "thread-1".into(),
            source: AgentInputSource::Selection,
            content: "bounded image question".into(),
            attachment_id: Some("image-1".into()),
            reply_mode: Some(AgentReplyMode::Stream),
            created_at: now,
            deadline: now + 10_000,
        };
        let (mut server, mut calls) = fake_app_server_with_calls().await;
        server.resume("thread-1").await.unwrap();
        assert!(
            handle_input(
                &mut server,
                "thread-1",
                &params,
                &mut DedupResults::new(),
                &mut inbox
            )
            .await
            .unwrap()
            .0
        );
        let requests: Vec<Value> = std::iter::from_fn(|| calls.try_recv().ok()).collect();
        let start = requests
            .iter()
            .find(|request| request.get("method").and_then(Value::as_str) == Some("turn/start"))
            .unwrap();
        let input = start
            .pointer("/params/input")
            .and_then(Value::as_array)
            .unwrap();
        assert_eq!(input.len(), 2);
        assert_eq!(
            input[0].get("text").and_then(Value::as_str),
            Some("bounded image question")
        );
        assert!(
            input[1]
                .get("url")
                .and_then(Value::as_str)
                .is_some_and(|url| url.starts_with("data:image/png;base64,"))
        );
        assert!(inbox.take("image-1", now).is_none());
    }

    #[tokio::test]
    async fn missing_attachment_is_rejected_without_text_fallback() {
        use yonder_protocol::AgentInputSource;

        let now = now_ms().unwrap();
        let params = AgentInputParams {
            input_id: "input-missing-image".into(),
            session_id: "thread-1".into(),
            source: AgentInputSource::Selection,
            content: "must not be sent alone".into(),
            attachment_id: Some("missing-image".into()),
            reply_mode: Some(AgentReplyMode::Stream),
            created_at: now,
            deadline: now + 10_000,
        };
        let (mut server, mut calls) = fake_app_server_with_calls().await;
        server.resume("thread-1").await.unwrap();
        let mut dedup = DedupResults::new();
        let mut inbox = AttachmentInbox::new("thread-1");
        assert!(
            !handle_input(&mut server, "thread-1", &params, &mut dedup, &mut inbox)
                .await
                .unwrap()
                .0
        );
        assert!(std::iter::from_fn(|| calls.try_recv().ok()).all(|request| {
            !matches!(
                request.get("method").and_then(Value::as_str),
                Some("thread/read" | "turn/start" | "turn/steer")
            )
        }));
    }

    #[test]
    fn maximum_attachment_request_stays_inside_codex_frame_bound() {
        let attachment = ReadyAttachment {
            id: "image-max".into(),
            mime: AgentAttachmentMime::ImagePng,
            deadline: u64::MAX,
            bytes: vec![0; MAX_AGENT_ATTACHMENT_BYTES as usize],
        };
        let request = json!({
            "method":"turn/start",
            "id":1,
            "params":{
                "threadId":"thread-1",
                "input":[
                    {"type":"text","text":"x".repeat(16 * 1024)},
                    {"type":"image","url":attachment.data_url(),"detail":"auto"}
                ]
            }
        });
        assert!(serde_json::to_vec(&request).unwrap().len() <= MAX_CODEX_REQUEST_FRAME_BYTES);
    }
}
