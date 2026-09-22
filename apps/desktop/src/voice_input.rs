use serde::Serialize;
use std::{ffi::{c_char, CStr}, sync::{mpsc, Arc, Mutex, OnceLock}};
use tauri::{AppHandle, Manager};
use yonder_application::agent_input::DeliveryOutcome;
use yonder_desktop::agent_input::AgentInputHub;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VoiceEvent { phase: &'static str, text: String }

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum VoiceTarget { Idle, Direct, Region }

#[derive(Debug, Eq, PartialEq)]
enum FinalRoute { Empty, Direct, Region }

fn final_route(target: VoiceTarget, text: &str) -> Option<FinalRoute> {
    if target == VoiceTarget::Idle { None }
    else if text.trim().is_empty() { Some(FinalRoute::Empty) }
    else if target == VoiceTarget::Region { Some(FinalRoute::Region) }
    else { Some(FinalRoute::Direct) }
}

pub struct VoiceRuntime { app: AppHandle, phase: Arc<Mutex<&'static str>>, target: Arc<Mutex<VoiceTarget>> }
static EVENTS: OnceLock<mpsc::Sender<VoiceEvent>> = OnceLock::new();

impl VoiceRuntime {
    pub fn new(app: AppHandle, hub: AgentInputHub) -> Self {
        let phase = Arc::new(Mutex::new("idle"));
        let target = Arc::new(Mutex::new(VoiceTarget::Idle));
        let (tx, rx) = mpsc::channel::<VoiceEvent>();
        let _ = EVENTS.set(tx);
        let current = Arc::clone(&phase);
        let destination = Arc::clone(&target);
        let worker_app = app.clone();
        std::thread::spawn(move || for event in rx {
            let route = destination.lock().map(|value| *value).unwrap_or(VoiceTarget::Idle);
            if route == VoiceTarget::Idle { continue; }
            if event.phase == "final" {
                match final_route(route, &event.text) {
                    Some(FinalRoute::Empty) => publish(&worker_app, &current, route, VoiceEvent { phase: "failed", text: "没有识别到可发送的语音".into() }),
                    Some(FinalRoute::Region) => publish(&worker_app, &current, route, event),
                    Some(FinalRoute::Direct) => {
                        publish(&worker_app, &current, route, VoiceEvent { phase: "processing", text: event.text.clone() });
                        let phase = match hub.deliver_voice(&event.text) {
                            Ok(DeliveryOutcome::Accepted) => "success",
                            Ok(DeliveryOutcome::Rejected) => "delivery_rejected",
                            Ok(DeliveryOutcome::Unknown) => "delivery_unknown",
                            Err(_) => "delivery_unavailable",
                        };
                        publish(&worker_app, &current, route, VoiceEvent { phase, text: event.text });
                    }
                    None => {}
                }
                if let Ok(mut value) = destination.lock() { *value = VoiceTarget::Idle; }
                continue;
            }
            let terminal = matches!(event.phase, "failed" | "cancelled");
            publish(&worker_app, &current, route, event);
            if terminal { if let Ok(mut value) = destination.lock() { *value = VoiceTarget::Idle; } }
        });
        Self { app, phase, target }
    }

    pub fn phase(&self) -> &'static str { self.phase.lock().map(|v| *v).unwrap_or("failed") }

    pub fn start(&self, destination: VoiceTarget) -> Result<(), String> {
        let mut target = self.target.lock().map_err(|_| "语音状态不可用")?;
        if *target != VoiceTarget::Idle { return Err("语音输入正在进行".into()); }
        *target = destination;
        if let Err(error) = start_native() { *target = VoiceTarget::Idle; return Err(error); }
        Ok(())
    }

    pub fn stop(&self, expected: VoiceTarget) -> Result<(), String> {
        if *self.target.lock().map_err(|_| "语音状态不可用")? != expected { return Err("语音输入状态已变化".into()); }
        stop_native(); Ok(())
    }

    pub fn cancel(&self, expected: VoiceTarget) {
        if self.target.lock().is_ok_and(|mut target| if *target == expected { *target = VoiceTarget::Idle; true } else { false }) {
            cancel_native();
            publish(&self.app, &self.phase, expected, VoiceEvent { phase: "cancelled", text: String::new() });
        }
    }
}

fn publish(app: &AppHandle, current: &Arc<Mutex<&'static str>>, target: VoiceTarget, event: VoiceEvent) {
            if let Ok(mut value) = current.lock() { *value = event.phase; }
            if let Ok(json) = serde_json::to_string(&event) {
                let script = format!("window.dispatchEvent(new CustomEvent('yonda-voice',{{detail:{json}}}))");
                for label in ["pet", if target == VoiceTarget::Region { "region-preview" } else { "voice-input" }] {
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

fn start_native() -> Result<(), String> {
    #[cfg(target_os = "macos")] { unsafe { yonda_voice_start(receive) }; Ok(()) }
    #[cfg(not(target_os = "macos"))] { Err("当前Windows语音Adapter仍在Spike验证中".into()) }
}
fn stop_native() { #[cfg(target_os = "macos")] unsafe { yonda_voice_stop() } }
fn cancel_native() { #[cfg(target_os = "macos")] unsafe { yonda_voice_cancel() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_transcript_has_one_trusted_route() {
        assert_eq!(final_route(VoiceTarget::Idle, "提问"), None);
        assert_eq!(final_route(VoiceTarget::Region, "  "), Some(FinalRoute::Empty));
        assert_eq!(final_route(VoiceTarget::Direct, "提问"), Some(FinalRoute::Direct));
        assert_eq!(final_route(VoiceTarget::Region, "提问"), Some(FinalRoute::Region));
    }
}
