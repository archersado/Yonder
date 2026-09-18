use serde::Serialize;
use std::{ffi::{c_char, CStr}, sync::{mpsc, Arc, Mutex, OnceLock}};
use tauri::{AppHandle, Manager};
use yonder_application::agent_input::DeliveryOutcome;
use yonder_desktop::agent_input::AgentInputHub;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VoiceEvent { phase: &'static str, text: String }

pub struct VoiceRuntime { phase: Arc<Mutex<&'static str>> }
static EVENTS: OnceLock<mpsc::Sender<VoiceEvent>> = OnceLock::new();

impl VoiceRuntime {
    pub fn new(app: AppHandle, hub: AgentInputHub) -> Self {
        let phase = Arc::new(Mutex::new("idle"));
        let (tx, rx) = mpsc::channel::<VoiceEvent>();
        let _ = EVENTS.set(tx);
        let current = Arc::clone(&phase);
        std::thread::spawn(move || for event in rx {
            if event.phase == "final" {
                if event.text.trim().is_empty() {
                    publish(&app, &current, VoiceEvent { phase: "failed", text: "没有识别到可发送的语音".into() });
                    continue;
                }
                publish(&app, &current, VoiceEvent { phase: "processing", text: event.text.clone() });
                let phase = match hub.deliver_voice(&event.text) {
                    Ok(DeliveryOutcome::Accepted) => "success",
                    Ok(DeliveryOutcome::Rejected) => "delivery_rejected",
                    Ok(DeliveryOutcome::Unknown) => "delivery_unknown",
                    Err(_) => "delivery_unavailable",
                };
                publish(&app, &current, VoiceEvent { phase, text: event.text });
                continue;
            }
            publish(&app, &current, event);
        });
        Self { phase }
    }

    pub fn phase(&self) -> &'static str { self.phase.lock().map(|v| *v).unwrap_or("failed") }
}

fn publish(app: &AppHandle, current: &Arc<Mutex<&'static str>>, event: VoiceEvent) {
            if let Ok(mut value) = current.lock() { *value = event.phase; }
            if let Ok(json) = serde_json::to_string(&event) {
                let script = format!("window.dispatchEvent(new CustomEvent('yonda-voice',{{detail:{json}}}))");
                for label in ["pet", "voice-input"] {
                    if let Some(window) = app.get_webview_window(label) { let _ = window.eval(&script); }
                }
            }
            if let Some(pet) = app.get_webview_window("pet") {
                let listening = matches!(event.phase, "requesting" | "listening" | "processing");
                let _ = pet.set_title(if listening { "Yonda · 正在聆听" } else { "Yonda" });
            }
}

#[cfg(target_os = "macos")]
unsafe extern "C" { fn yonda_voice_start(callback: extern "C" fn(i32, *const c_char)); fn yonda_voice_stop(); fn yonda_voice_cancel(); }

#[cfg(target_os = "macos")]
extern "C" fn receive(kind: i32, text: *const c_char) {
    let text = if text.is_null() { String::new() } else { unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned() };
    let phase = match kind { 0 => "requesting", 1 => "listening", 2 => "listening", 3 => "final", 4 => "failed", 5 => "processing", _ => "cancelled" };
    if let Some(events) = EVENTS.get() { let _ = events.send(VoiceEvent { phase, text }); }
}

pub fn start() -> Result<(), String> {
    #[cfg(target_os = "macos")] { unsafe { yonda_voice_start(receive) }; Ok(()) }
    #[cfg(not(target_os = "macos"))] { Err("当前Windows语音Adapter仍在Spike验证中".into()) }
}
pub fn stop() { #[cfg(target_os = "macos")] unsafe { yonda_voice_stop() } }
pub fn cancel() { #[cfg(target_os = "macos")] unsafe { yonda_voice_cancel() } }
