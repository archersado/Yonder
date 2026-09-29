//! AD-AG-05：macOS当前用户私有Gateway UDS。
use interprocess::local_socket::{GenericFilePath, ListenerOptions, ToFsName, tokio::{Listener, Stream, prelude::*}};
use base64::Engine;
use sha2::{Digest, Sha256};
use std::{io, os::unix::fs::PermissionsExt, path::PathBuf, sync::{Arc, Mutex}, time::{Duration, SystemTime, UNIX_EPOCH}};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use yonder_application::{AuthContext, agent_input::{AgentAttachmentBeginParams, AgentAttachmentChunkParams, AgentAttachmentFinishParams, AgentRequest, AgentResponse, DeliveryOutcome, Version, decode_response, encode_request, hello_accepted, registration}, gateway::{GatewaySession, Platform, local_hello_agent_id}};
use crate::{CuaControlHub, TaskHost, agent_input::AgentInputHub, emit_pet_agent_connection, emit_pet_presentation,emit_pet_terminal_presentation, position_window_in_pet_work_area};
use tauri::{Manager, WebviewWindow};

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
    pub fn start(host: Arc<Mutex<Option<TaskHost>>>, path: PathBuf, pet: WebviewWindow, hub: AgentInputHub, cua_control: WebviewWindow, cua_hub: CuaControlHub) -> io::Result<Self> {
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
                serve(listener, host, pet, hub, cua_control, cua_hub, stopped).await;
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

async fn serve(listener: Listener, host: Arc<Mutex<Option<TaskHost>>>, pet: WebviewWindow, hub: AgentInputHub, cua_control: WebviewWindow, cua_hub: CuaControlHub, mut stop: tokio::sync::oneshot::Receiver<()>) {
    loop {
        tokio::select! {
            _ = &mut stop => break,
            connection = listener.accept() => match connection {
                Ok(stream) => { let host = host.clone(); let pet = pet.clone(); let hub=hub.clone(); let cua_control=cua_control.clone(); let cua_hub=cua_hub.clone(); tokio::spawn(async move { let _ = exchange(stream, host, pet, hub, cua_control, cua_hub).await; }); },
                Err(_) => break,
            }
        }
    }
}

fn hint_task_id(hint: &yonder_application::gateway::ExecutionPresentationHint) -> &str {
    use yonder_application::gateway::ExecutionPresentationHint;
    match hint {
        ExecutionPresentationHint::StoredStep { task_id }
        | ExecutionPresentationHint::DeclaredStep { task_id, .. }
        | ExecutionPresentationHint::PlanSlot { task_id, .. } => task_id,
    }
}

fn show_cua_control(pet: &WebviewWindow, window: &WebviewWindow, presentation: &crate::CuaControlPresentation) -> io::Result<()> {
    position_window_in_pet_work_area(pet, window, 560.0, 174.0).map_err(io::Error::other)?;
    let detail=serde_json::json!({"presentation":presentation});
    window.eval(&format!("window.dispatchEvent(new CustomEvent('yonda-cua-control-start',{{detail:{detail}}}))"))
        .and_then(|_|window.show()).map_err(io::Error::other)
}

async fn exchange(stream: Stream, host: Arc<Mutex<Option<TaskHost>>>, pet: WebviewWindow, hub: AgentInputHub, cua_control: WebviewWindow, cua_hub: CuaControlHub) -> io::Result<()> {
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
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?.as_millis()).map_err(io::Error::other)?;
        let execution_hint = yonder_application::gateway::execution_presentation_hint(&frame, &agent_id, now);
        let cua_hint = yonder_application::gateway::cua_execution_presentation_hint(&frame, &agent_id, now);
        let control_checkpoint = cua_hint.as_ref().and_then(|_| cua_hub.checkpoint());
        let (response, presentation, suppress_stale_presentation, open_task_space, provisional_control, accepted_plan_control) = {
            let mut locked = host.lock().map_err(|_| io::Error::other("本地Gateway不可用"))?;
            let host_ref = locked.as_mut().ok_or_else(|| io::Error::other("本地Gateway不可用"))?;
            let mut provisional_control=false;
            if let Some(hint) = execution_hint.as_ref() {
                let step_label = host_ref.execution_step_label(&agent_id, hint);
                emit_pet_presentation(&pet, true, "executing", step_label.as_deref());
            }
            if let Some(hint) = execution_hint.as_ref() {
                let task_id=hint_task_id(hint);
                let desktop_control=cua_hint.as_ref().is_some_and(|cua|hint_task_id(cua)==task_id);
                let presentation=host_ref.execution_control_presentation(&agent_id,hint,desktop_control);
                if let Some(presentation)=presentation.filter(|presentation|presentation.task_id==*task_id) {
                    if cua_hub.begin(presentation.clone()) {
                    provisional_control=true;
                    if let Err(error)=show_cua_control(&pet,&cua_control,&presentation) {
                        cua_hub.finish(task_id);
                        return Err(error);
                    }
                    }
                }
            }
            let response = host_ref.query_session(&mut session, &frame, now);
            let accepted_plan_control=response.as_ref().ok()
                .and_then(|bytes|yonder_application::gateway::accepted_plan_presentation_hint(&frame,bytes,&agent_id,now))
                .and_then(|hint|host_ref.execution_control_presentation(&agent_id,&hint,true));
            let open_task_space = response.is_ok() && host_ref.take_task_space_open_pending();
            // 接管由可信窗口命令在同一个 TaskHost 锁边界内唯一消费。这里若已登记，
            // 不得在释放锁后用动作返回前取得的 running 投影覆盖窗口命令的 paused 投影。
            let suppress_stale_presentation = cua_hint.as_ref().is_some_and(|hint|cua_hub.takeover_requested(hint_task_id(hint)));
            let presentation = host_ref.presentation().ok();
            (response, presentation, suppress_stale_presentation, open_task_space, provisional_control, accepted_plan_control)
        };
        if open_task_space {
            // Local Socket 在 Tokio worker 中处理；窗口操作须回到 Cocoa 主线程，
            // 否则创建虽成功但任务窗口不会被真正展示。
            let app = pet.app_handle().clone();
            app.clone().run_on_main_thread(move || {
                if let Some(pet) = app.get_webview_window("pet") {
                    if let Err(error) = crate::show_task_space_after_agent_create(&pet) {
                        eprintln!("Agent 创建后的任务窗口展示失败: {error}");
                    }
                }
            }).map_err(io::Error::other)?;
        }
        let accepted=response.as_ref().is_ok_and(|bytes|yonder_application::gateway::response_succeeded(bytes));
        if provisional_control&&!accepted {
            if let Some(previous)=control_checkpoint {
                if cua_hub.begin(previous.clone()) { let _=show_cua_control(&pet,&cua_control,&previous); }
            } else if let Some(hint)=cua_hint.as_ref() {
                if cua_hub.finish(hint_task_id(hint)) { let _=cua_control.hide(); }
            }
        }
        let response=response.map_err(|_|io::Error::other("本地Gateway调用失败"))?;
        if let Some(presentation)=accepted_plan_control {
            if cua_hub.begin(presentation.clone()) { let _=show_cua_control(&pet,&cua_control,&presentation); }
        }
        let replan_wake=yonder_application::gateway::replan_wake(&response);
        if let Some(wake)=replan_wake.as_ref() {
            cua_hub.mark_replanning(&wake.task_id,&wake.reason);
            if let Some(presentation)=cua_hub.presentation(){let _=show_cua_control(&pet,&cua_control,&presentation);}
        }
        if let Some(task_id)=yonder_application::gateway::terminal_task_id(&response) {
            if cua_hub.finish(&task_id) || cua_hub.presentation().is_none() { let _=cua_control.hide(); }
        }
        if !suppress_stale_presentation { if let Some((has_tasks, state, step_label)) = presentation {
            if let Some((terminal,event_id))=yonder_application::gateway::terminal_presentation(&frame,&response){
                emit_pet_terminal_presentation(&pet,has_tasks,terminal,&event_id,has_tasks,state,step_label.as_deref());
            }else{
                emit_pet_presentation(&pet, has_tasks, state, step_label.as_deref());
            }
            if state == "listening" {
                let host = host.clone(); let pet = pet.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(1650)).await;
                    let presentation = host.lock().ok().and_then(|mut value| value.as_mut()?.presentation().ok());
                    if let Some((has_tasks, state, step_label)) = presentation { emit_pet_presentation(&pet, has_tasks, state, step_label.as_deref()); }
                });
            }
        }}
        (&stream).write_all(&response).await?;
        (&stream).write_all(b"\n").await?;
        if let Some(wake)=replan_wake {
            let hub=hub.clone();let cua_hub=cua_hub.clone();let cua_control=cua_control.clone();let pet=pet.clone();let owner=agent_id.clone();
            tokio::spawn(async move {
                let task_id=wake.task_id.clone();
                let delivered=tokio::task::spawn_blocking(move||hub.deliver_replan(&owner,&wake.task_id,&wake.sequence,&wake.reason)).await;
                let state=match delivered {
                    Ok(Ok(yonder_application::agent_input::DeliveryOutcome::Accepted))=>"accepted",
                    Ok(Err("任务归属Agent未连接"))=>"missing",
                    Ok(Err("任务归属Agent存在多个会话"))=>"ambiguous",
                    _=>"failed",
                };
                cua_hub.mark_replan_delivery(&task_id,state);
                if let Some(presentation)=cua_hub.presentation(){let _=show_cua_control(&pet,&cua_control,&presentation);}
            });
        }
        if let Some((agent_id, session_id, supports_attachment, supports_replan)) = registration {
            if !hello_accepted(&response) { continue; }
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let (close_tx, mut close_rx) = tokio::sync::mpsc::unbounded_channel();
            let (token, connected) = hub.register(agent_id, session_id.clone(), supports_attachment, supports_replan, tx, close_tx);
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
