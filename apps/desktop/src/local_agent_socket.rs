//! AD-AG-05：macOS当前用户私有Gateway UDS。
use interprocess::local_socket::{GenericFilePath, ListenerOptions, ToFsName, tokio::{Listener, Stream, prelude::*}};
use base64::Engine;
use sha2::{Digest, Sha256};
use std::{io, os::unix::fs::PermissionsExt, path::PathBuf, sync::{Arc, Mutex}, time::{Duration, SystemTime, UNIX_EPOCH}};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use yonder_application::{AuthContext, agent_input::{AgentAttachmentBeginParams, AgentAttachmentChunkParams, AgentAttachmentFinishParams, AgentRequest, AgentResponse, DeliveryOutcome, Version, decode_response, encode_request, hello_accepted, registration}, gateway::{GatewaySession, Platform, local_hello_agent_id}};
use crate::{TaskHost, agent_input::AgentInputHub, emit_pet_agent_connection, emit_pet_presentation,emit_pet_terminal_presentation};
use tauri::WebviewWindow;

const MAX_FRAME_BYTES: usize = 64 * 1024;
const ATTACHMENT_CHUNK_BYTES: usize = 47 * 1024;

async fn send_agent_request(stream: &Stream, request: &AgentRequest) -> io::Result<()> {
    let mut bytes = encode_request(request).map_err(|error| io::Error::other(error.message))?;
    if bytes.len() > MAX_FRAME_BYTES { return Err(io::Error::new(io::ErrorKind::InvalidData, "Agent请求帧过大")); }
    bytes.push(b'\n');
    (&*stream).write_all(&bytes).await
}

async fn read_agent_outcome(reader: &mut BufReader<&Stream>, request_id: &str, timeout: Duration) -> DeliveryOutcome {
    let response = tokio::time::timeout(timeout, async {
        let mut frame = Vec::new();
        let read = reader.take(MAX_FRAME_BYTES as u64 + 1).read_until(b'\n', &mut frame).await?;
        if read == 0 || read > MAX_FRAME_BYTES || frame.last() != Some(&b'\n') { return Err(io::Error::new(io::ErrorKind::InvalidData,"无效Agent确认帧")); }
        frame.pop();
        Ok::<_,io::Error>(frame)
    }).await;
    match response {
        Ok(Ok(frame)) => match decode_response(&frame) {
            Ok(AgentResponse::Success{id,result,..}) if id==request_id&&result.accepted=>DeliveryOutcome::Accepted,
            Ok(AgentResponse::Success{id,result,..}) if id==request_id&&!result.accepted=>DeliveryOutcome::Rejected,
            _=>DeliveryOutcome::Unknown,
        },
        _=>DeliveryOutcome::Unknown,
    }
}


pub struct Server {
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Server {
    pub fn start(host: Arc<Mutex<Option<TaskHost>>>, path: PathBuf, pet: WebviewWindow, hub: AgentInputHub) -> io::Result<Self> {
        let parent = path.parent().ok_or_else(|| io::Error::other("本地Gateway路径不可用"))?;
        std::fs::create_dir_all(parent)?;
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let thread = std::thread::Builder::new().name("local-agent-socket".into()).spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                Ok(runtime) => runtime,
                Err(error) => { let _ = ready_tx.send(Err(error)); return; }
            };
            runtime.block_on(async move {
                let name = match path.clone().to_fs_name::<GenericFilePath>() {
                    Ok(name) => name,
                    Err(error) => { let _ = ready_tx.send(Err(error)); return; }
                };
                let listener = match ListenerOptions::new().name(name).try_overwrite(true).create_tokio() {
                    Ok(listener) => listener,
                    Err(error) => { let _ = ready_tx.send(Err(error)); return; }
                };
                if let Err(error) = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)) {
                    let _ = ready_tx.send(Err(error)); return;
                }
                if ready_tx.send(Ok(())).is_err() { return; }
                serve(listener, host, pet, hub, stopped).await;
                let _ = std::fs::remove_file(path);
            });
        })?;
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self { stop: Some(stop), thread: Some(thread) }),
            Ok(Err(error)) => { let _ = thread.join(); Err(error) },
            Err(_) => { let _ = thread.join(); Err(io::Error::other("本地Gateway启动中断")) },
        }
    }

    fn stop(&mut self) {
        if let Some(stop) = self.stop.take() { let _ = stop.send(()); }
        if let Some(thread) = self.thread.take() { let _ = thread.join(); }
    }
}

async fn serve(listener: Listener, host: Arc<Mutex<Option<TaskHost>>>, pet: WebviewWindow, hub: AgentInputHub, mut stop: tokio::sync::oneshot::Receiver<()>) {
    loop {
        tokio::select! {
            _ = &mut stop => break,
            connection = listener.accept() => match connection {
                Ok(stream) => { let host = host.clone(); let pet = pet.clone(); let hub=hub.clone(); tokio::spawn(async move { let _ = exchange(stream, host, pet, hub).await; }); },
                Err(_) => break,
            }
        }
    }
}

async fn exchange(stream: Stream, host: Arc<Mutex<Option<TaskHost>>>, pet: WebviewWindow, hub: AgentInputHub) -> io::Result<()> {
    let mut reader = BufReader::new(&stream);
    let mut first = Vec::new();
    let read = (&mut reader).take(MAX_FRAME_BYTES as u64 + 1).read_until(b'\n', &mut first).await?;
    if read == 0 || read > MAX_FRAME_BYTES || first.last() != Some(&b'\n') { return Err(io::Error::new(io::ErrorKind::InvalidData, "本地Gateway首帧无效")); }
    first.pop();
    let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?.as_millis()).map_err(io::Error::other)?;
    let agent_id = local_hello_agent_id(&first, now).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "本地Gateway必须先握手"))?;
    let mut session = GatewaySession::new(AuthContext::Agent(&agent_id), Platform::Macos);
    let mut initial = Some(first);
    loop {
        let frame = match initial.take() {
            Some(frame) => frame,
            None => {
                let mut frame = Vec::new();
                let read = (&mut reader).take(MAX_FRAME_BYTES as u64 + 1).read_until(b'\n', &mut frame).await?;
                if read == 0 { return Ok(()); }
                if read > MAX_FRAME_BYTES || frame.last() != Some(&b'\n') { return Err(io::Error::new(io::ErrorKind::InvalidData, "无效本地Gateway帧")); }
                frame.pop(); frame
            }
        };
        let registration = registration(&frame);
        if yonder_application::gateway::is_execution_request(&frame) { emit_pet_presentation(&pet, true, "executing"); }
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?.as_millis()).map_err(io::Error::other)?;
        let (response, presentation) = {
            let mut locked = host.lock().map_err(|_| io::Error::other("本地Gateway不可用"))?;
            let host_ref = locked.as_mut().ok_or_else(|| io::Error::other("本地Gateway不可用"))?;
            let response = host_ref.query_session(&mut session, &frame, now).map_err(|_| io::Error::other("本地Gateway调用失败"))?;
            let presentation = host_ref.presentation().ok();
            (response, presentation)
        };
        if let Some((has_tasks, state)) = presentation {
            if let Some((terminal,event_id))=yonder_application::gateway::terminal_presentation(&frame,&response){
                emit_pet_terminal_presentation(&pet,has_tasks,terminal,&event_id,has_tasks,state);
            }else{
                emit_pet_presentation(&pet, has_tasks, state);
            }
            if state == "listening" {
                let host = host.clone(); let pet = pet.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(1650)).await;
                    let presentation = host.lock().ok().and_then(|mut value| value.as_mut()?.presentation().ok());
                    if let Some((has_tasks, state)) = presentation { emit_pet_presentation(&pet, has_tasks, state); }
                });
            }
        }
        (&stream).write_all(&response).await?;
        (&stream).write_all(b"\n").await?;
        if let Some((agent_id, session_id, supports_attachment)) = registration {
            if !hello_accepted(&response) { continue; }
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let (close_tx, mut close_rx) = tokio::sync::mpsc::unbounded_channel();
            let (token, connected) = hub.register(agent_id, session_id.clone(), supports_attachment, tx, close_tx);
            emit_pet_agent_connection(&pet, connected);
            let result = async {
                loop {
                    let delivery = tokio::select! {
                        delivery = rx.recv() => match delivery { Some(delivery) => delivery, None => break },
                        _ = close_rx.recv() => break,
                        disconnected = reader.fill_buf() => match disconnected {
                            Ok([]) => break,
                            Ok(_) => return Err(io::Error::new(io::ErrorKind::InvalidData, "意外Agent帧")),
                            Err(error) => return Err(error),
                        },
                    };
                    let request_id = delivery.input.input_id.clone();
                    let mut outcome = DeliveryOutcome::Accepted;
                    if let Some(attachment) = delivery.attachment {
                        let Some(attachment_id) = delivery.input.attachment_id.clone() else { let _=delivery.reply.send(DeliveryOutcome::Unknown); return Ok(()); };
                        let begin=AgentAttachmentBeginParams{attachment_id:attachment_id.clone(),session_id:delivery.input.session_id.clone(),mime:attachment.mime,byte_length:attachment.bytes.len().try_into().map_err(|_|io::Error::other("Agent附件过大"))?,sha256:format!("{:x}",Sha256::digest(&attachment.bytes)),deadline:delivery.input.deadline};
                        begin.validate(u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?.as_millis()).map_err(io::Error::other)?).map_err(|error|io::Error::other(error.message))?;
                        send_agent_request(&stream,&AgentRequest::AttachmentBegin{jsonrpc:Version::V2,params:begin}).await?;
                        for (sequence,bytes) in attachment.bytes.chunks(ATTACHMENT_CHUNK_BYTES).enumerate(){
                            let params=AgentAttachmentChunkParams{attachment_id:attachment_id.clone(),session_id:delivery.input.session_id.clone(),sequence:sequence.try_into().map_err(|_|io::Error::other("Agent附件分块过多"))?,data_base64:base64::engine::general_purpose::STANDARD.encode(bytes)};
                            params.validate().map_err(|error|io::Error::other(error.message))?;
                            send_agent_request(&stream,&AgentRequest::AttachmentChunk{jsonrpc:Version::V2,params}).await?;
                        }
                        let finish_id=format!("{}_attachment",request_id);
                        let params=AgentAttachmentFinishParams{attachment_id,session_id:delivery.input.session_id.clone()};
                        params.validate().map_err(|error|io::Error::other(error.message))?;
                        send_agent_request(&stream,&AgentRequest::AttachmentFinish{jsonrpc:Version::V2,request_id:finish_id.clone(),params}).await?;
                        outcome=read_agent_outcome(&mut reader,&finish_id,Duration::from_secs(60)).await;
                    }
                    if outcome==DeliveryOutcome::Accepted {
                        send_agent_request(&stream,&AgentRequest::Input { jsonrpc: Version::V2, request_id: request_id.clone(), params: delivery.input }).await?;
                        outcome=read_agent_outcome(&mut reader,&request_id,Duration::from_secs(60)).await;
                    }
                    let _ = delivery.reply.send(outcome);
                    if outcome == DeliveryOutcome::Unknown { return Ok(()); }
                }
                Ok(())
            }.await;
            emit_pet_agent_connection(&pet, hub.unregister(&session_id, token));
            return result;
        }
    }
}

impl Drop for Server { fn drop(&mut self) { self.stop(); } }
