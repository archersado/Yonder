use serde::Serialize;
use std::{
    ffi::{CString, c_char},
    sync::{Arc, Mutex, OnceLock, mpsc},
};
use tauri::{AppHandle, Manager};
use yonder_application::agent_input::{AgentReplyParams, AgentReplyPhase};
use yonder_application::voice_reply::{SpeakRequest, VoiceReplySession, VoiceSynthesisPort};

#[derive(Clone, Copy)]
struct NativeEvent {
    generation: u64,
    sequence: u64,
    kind: i32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VoiceReplyEvent {
    phase: &'static str,
    generation: u64,
    sequence: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceReplyConfig {
    pub enabled: bool,
    pub available: bool,
}

#[derive(Clone, Copy)]
struct NativeVoiceSynthesisAdapter;

impl VoiceSynthesisPort for NativeVoiceSynthesisAdapter {
    fn available(&self) -> bool {
        native_available()
    }

    fn speak(&self, request: &SpeakRequest) -> Result<(), &'static str> {
        let text = CString::new(request.text.as_str()).map_err(|_| "voice-text-invalid")?;
        native_speak(request.generation, request.sequence, text.as_ptr());
        Ok(())
    }

    fn stop(&self) {
        native_stop();
    }
}

#[derive(Clone)]
pub struct VoiceReplyRuntime {
    app: AppHandle,
    session: Arc<Mutex<VoiceReplySession>>,
    port: NativeVoiceSynthesisAdapter,
}

static EVENTS: OnceLock<mpsc::Sender<NativeEvent>> = OnceLock::new();

impl VoiceReplyRuntime {
    pub fn new(app: AppHandle) -> Self {
        let session = Arc::new(Mutex::new(VoiceReplySession::new(native_enabled())));
        let (tx, rx) = mpsc::channel();
        let _ = EVENTS.set(tx);
        let worker_session = Arc::clone(&session);
        let worker_app = app.clone();
        std::thread::Builder::new()
            .name("voice-reply-events".into())
            .spawn(move || {
                for event in rx {
                    let phase = {
                        let Ok(mut session) = worker_session.lock() else {
                            continue;
                        };
                        if !session.accepts_callback(event.generation) {
                            continue;
                        }
                        match event.kind {
                            1 => Some("speaking"),
                            2 => Some("segment-completed"),
                            3 => Some("stopped"),
                            4 if session.native_idle(event.generation) => Some("completed"),
                            5 => {
                                session.stop();
                                Some("unavailable")
                            }
                            _ => None,
                        }
                    };
                    if let Some(phase) = phase {
                        publish(&worker_app, phase, event.generation, Some(event.sequence));
                    }
                }
            })
            .expect("语音回复事件线程必须可创建");
        Self {
            app,
            session,
            port: NativeVoiceSynthesisAdapter,
        }
    }

    pub fn config(&self) -> VoiceReplyConfig {
        let enabled = self
            .session
            .lock()
            .map(|session| session.enabled())
            .unwrap_or(false);
        VoiceReplyConfig {
            enabled,
            available: self.port.available(),
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        let effective = enabled && self.port.available();
        native_set_enabled(effective);
        let should_stop = self
            .session
            .lock()
            .map(|mut session| session.set_enabled(effective))
            .unwrap_or(true);
        if should_stop {
            self.port.stop();
            publish(&self.app, "stopped", 0, None);
        }
    }

    pub fn handle_reply(&self, params: &AgentReplyParams) {
        let enabled = self
            .session
            .lock()
            .map(|session| session.enabled())
            .unwrap_or(false);
        if !enabled {
            return;
        }
        match params.phase {
            AgentReplyPhase::Started => {
                let generation = self
                    .session
                    .lock()
                    .ok()
                    .and_then(|mut session| session.begin(&params.reply_id).ok());
                if let Some(generation) = generation {
                    self.port.stop();
                    if self.port.available() {
                        publish(&self.app, "preparing", generation, None);
                    } else {
                        self.fail_unavailable();
                    }
                }
            }
            AgentReplyPhase::Delta => {
                let result = self.session.lock().ok().map(|mut session| {
                    session.push_delta(&params.reply_id, params.content.as_deref().unwrap_or(""))
                });
                match result {
                    Some(Ok(requests)) => self.dispatch(requests),
                    Some(Err(yonder_application::voice_reply::VoiceReplyError::ReplyTooLarge)) => {
                        self.fail_unavailable()
                    }
                    _ => {}
                }
            }
            AgentReplyPhase::Completed => {
                let result = self.session.lock().ok().and_then(|mut session| {
                    let requests = session.finish(&params.reply_id).ok()?;
                    let generation = session.generation();
                    let completed = session.complete_if_idle(generation);
                    Some((requests, generation, completed))
                });
                if let Some((requests, generation, completed)) = result {
                    self.dispatch(requests);
                    if completed {
                        publish(&self.app, "completed", generation, None);
                    }
                }
            }
            AgentReplyPhase::Failed | AgentReplyPhase::Interrupted => self.stop(),
        }
    }

    pub fn stop(&self) {
        let generation = self
            .session
            .lock()
            .map(|mut session| session.stop())
            .unwrap_or(0);
        self.port.stop();
        publish(&self.app, "stopped", generation, None);
    }

    fn dispatch(&self, requests: Vec<SpeakRequest>) {
        for request in requests {
            if self.port.speak(&request).is_err() {
                self.fail_unavailable();
                break;
            }
        }
    }

    fn fail_unavailable(&self) {
        let generation = self
            .session
            .lock()
            .map(|mut session| session.stop())
            .unwrap_or(0);
        self.port.stop();
        publish(&self.app, "unavailable", generation, None);
    }
}

impl Drop for VoiceReplyRuntime {
    fn drop(&mut self) {
        if Arc::strong_count(&self.session) == 1 {
            native_release();
        }
    }
}

fn publish(app: &AppHandle, phase: &'static str, generation: u64, sequence: Option<u64>) {
    let Ok(detail) = serde_json::to_string(&VoiceReplyEvent {
        phase,
        generation,
        sequence,
    }) else {
        return;
    };
    let script =
        format!("window.dispatchEvent(new CustomEvent('yonda-voice-reply',{{detail:{detail}}}))");
    for label in ["region-preview", "agent-settings"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.eval(&script);
        }
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn yonda_voice_reply_available() -> bool;
    fn yonda_voice_reply_enabled() -> bool;
    fn yonda_voice_reply_set_enabled(enabled: bool);
    fn yonda_voice_reply_speak(
        generation: u64,
        sequence: u64,
        text: *const c_char,
        callback: extern "C" fn(u64, u64, i32),
    );
    fn yonda_voice_reply_stop();
    fn yonda_voice_reply_release();
}

extern "C" fn receive(generation: u64, sequence: u64, kind: i32) {
    if let Some(events) = EVENTS.get() {
        let _ = events.send(NativeEvent {
            generation,
            sequence,
            kind,
        });
    }
}

fn native_available() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        yonda_voice_reply_available()
    }
    #[cfg(not(target_os = "macos"))]
    false
}

fn native_enabled() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        yonda_voice_reply_enabled()
    }
    #[cfg(not(target_os = "macos"))]
    false
}

fn native_set_enabled(enabled: bool) {
    #[cfg(target_os = "macos")]
    unsafe {
        yonda_voice_reply_set_enabled(enabled)
    }
    #[cfg(not(target_os = "macos"))]
    let _ = enabled;
}

fn native_speak(generation: u64, sequence: u64, text: *const c_char) {
    #[cfg(target_os = "macos")]
    unsafe {
        yonda_voice_reply_speak(generation, sequence, text, receive)
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (generation, sequence, text);
}

fn native_stop() {
    #[cfg(target_os = "macos")]
    unsafe {
        yonda_voice_reply_stop()
    }
}

fn native_release() {
    #[cfg(target_os = "macos")]
    unsafe {
        yonda_voice_reply_release()
    }
}
