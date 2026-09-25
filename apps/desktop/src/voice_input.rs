use serde::Serialize;
use std::{ffi::{c_char, CStr}, sync::{atomic::{AtomicI64, Ordering}, mpsc, Arc, Mutex, OnceLock}};
use tauri::{AppHandle, Manager};
use yonder_application::agent_input::DeliveryOutcome;
use yonder_desktop::agent_input::AgentInputHub;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct VoiceEvent { #[serde(skip)] session_id: i64, phase: &'static str, text: String }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VoiceTarget { Idle, Direct, Region }

#[derive(Debug, Eq, PartialEq)]
enum FinalRoute { Empty, Direct, Region }

#[derive(Clone, Copy)]
struct VoiceSession { id: i64, target: VoiceTarget, final_claimed: bool }

fn final_route(target: VoiceTarget, text: &str) -> Option<FinalRoute> {
    if target == VoiceTarget::Idle { None }
    else if text.trim().is_empty() { Some(FinalRoute::Empty) }
    else if target == VoiceTarget::Region { Some(FinalRoute::Region) }
    else { Some(FinalRoute::Direct) }
}

fn claim_route(session: &mut Option<VoiceSession>, event: &VoiceEvent) -> Option<VoiceTarget> {
    let active = session.as_mut()?;
    if active.id != event.session_id { return None; }
    if event.phase == "final" {
        if active.final_claimed { return None; }
        active.final_claimed = true;
    }
    Some(active.target)
}

fn clear_session(session: &mut Option<VoiceSession>, session_id: i64) {
    if session.as_ref().is_some_and(|active| active.id == session_id) { *session = None; }
}

pub struct VoiceRuntime {
    app: AppHandle,
    phase: Arc<Mutex<&'static str>>,
    session: Arc<Mutex<Option<VoiceSession>>>,
    next_session_id: AtomicI64,
}
static EVENTS: OnceLock<mpsc::Sender<VoiceEvent>> = OnceLock::new();

impl VoiceRuntime {
    pub fn new(app: AppHandle, hub: AgentInputHub) -> Self {
        let phase = Arc::new(Mutex::new("idle"));
        let session = Arc::new(Mutex::new(None));
        let (tx, rx) = mpsc::channel::<VoiceEvent>();
        let _ = EVENTS.set(tx);
        let current = Arc::clone(&phase);
        let active_session = Arc::clone(&session);
        let worker_app = app.clone();
        std::thread::spawn(move || for event in rx {
            let route = active_session.lock().ok().and_then(|mut session| claim_route(&mut session, &event));
            let Some(route) = route else { continue; };
            if event.phase == "final" {
                let session_id = event.session_id;
                match final_route(route, &event.text) {
                    Some(FinalRoute::Empty) => publish(&worker_app, &current, route, VoiceEvent { session_id, phase: "failed", text: "没有识别到可发送的语音".into() }),
                    Some(FinalRoute::Region) => publish(&worker_app, &current, route, event),
                    Some(FinalRoute::Direct) => {
                        publish(&worker_app, &current, route, VoiceEvent { session_id, phase: "processing", text: event.text.clone() });
                        let phase = match hub.deliver_voice(&event.text) {
                            Ok(DeliveryOutcome::Accepted) => "success",
                            Ok(DeliveryOutcome::Rejected) => "delivery_rejected",
                            Ok(DeliveryOutcome::Unknown) => "delivery_unknown",
                            Err(_) => "delivery_unavailable",
                        };
                        publish(&worker_app, &current, route, VoiceEvent { session_id, phase, text: event.text });
                    }
                    None => {}
                }
                if let Ok(mut session) = active_session.lock() { clear_session(&mut session, session_id); }
                continue;
            }
            let terminal = matches!(event.phase, "failed" | "cancelled");
            let session_id = event.session_id;
            publish(&worker_app, &current, route, event);
            if terminal { if let Ok(mut session) = active_session.lock() { clear_session(&mut session, session_id); } }
        });
        Self { app, phase, session, next_session_id: AtomicI64::new(0) }
    }

    pub fn phase(&self) -> &'static str { self.phase.lock().map(|value| *value).unwrap_or("failed") }

    pub fn start(&self, destination: VoiceTarget) -> Result<(), String> {
        let mut session = self.session.lock().map_err(|_| "语音状态不可用")?;
        if session.is_some() { return Err("语音输入正在进行".into()); }
        let session_id = self.next_session_id.fetch_add(1, Ordering::Relaxed) + 1;
        *session = Some(VoiceSession { id: session_id, target: destination, final_claimed: false });
        if let Err(error) = start_native(session_id) { *session = None; return Err(error); }
        Ok(())
    }

    pub fn stop(&self, expected: VoiceTarget) -> Result<(), String> {
        if !self.session.lock().map_err(|_| "语音状态不可用")?.as_ref().is_some_and(|session| session.target == expected) { return Err("语音输入状态已变化".into()); }
        stop_native(); Ok(())
    }

    pub fn cancel(&self, expected: VoiceTarget) {
        if self.session.lock().is_ok_and(|mut session| if session.as_ref().is_some_and(|active| active.target == expected) { *session = None; true } else { false }) {
            cancel_native();
            publish(&self.app, &self.phase, expected, VoiceEvent { session_id: 0, phase: "cancelled", text: String::new() });
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
unsafe extern "C" {
    fn yonda_voice_start(session_id: i64, callback: extern "C" fn(i64, i32, *const c_char));
    fn yonda_voice_stop();
    fn yonda_voice_cancel();
    #[cfg(test)]
    fn yonda_voice_vad_test(levels: *const f32, durations: *const f64, count: i32) -> i32;
    #[cfg(test)]
    fn yonda_voice_finish_gate_test(signal_count: i32) -> i32;
}

#[cfg(target_os = "macos")]
extern "C" fn receive(session_id: i64, kind: i32, text: *const c_char) {
    let text = if text.is_null() { String::new() } else { unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned() };
    let phase = match kind { 0 => "requesting", 1 => "listening", 2 => "listening", 3 => "final", 4 => "failed", 5 => "processing", _ => "cancelled" };
    if let Some(events) = EVENTS.get() { let _ = events.send(VoiceEvent { session_id, phase, text }); }
}

fn start_native(session_id: i64) -> Result<(), String> {
    #[cfg(target_os = "macos")] { unsafe { yonda_voice_start(session_id, receive) }; Ok(()) }
    #[cfg(not(target_os = "macos"))] { let _ = session_id; Err("当前Windows语音Adapter仍在Spike验证中".into()) }
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

    #[test]
    fn final_is_claimed_once_and_old_session_is_rejected() {
        let mut session = Some(VoiceSession { id: 7, target: VoiceTarget::Direct, final_claimed: false });
        let old = VoiceEvent { session_id: 6, phase: "final", text: "旧内容".into() };
        let current = VoiceEvent { session_id: 7, phase: "final", text: "整段内容".into() };
        assert_eq!(claim_route(&mut session, &old), None);
        assert_eq!(claim_route(&mut session, &current), Some(VoiceTarget::Direct));
        assert_eq!(claim_route(&mut session, &current), None);
        clear_session(&mut session, 6); assert!(session.is_some());
        clear_session(&mut session, 7); assert!(session.is_none());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_voice_activity_requires_speech_before_trailing_silence() {
        let silence_levels = [0.0_f32; 5];
        let silence_durations = [0.25_f64; 5];
        assert_eq!(unsafe { yonda_voice_vad_test(silence_levels.as_ptr(), silence_durations.as_ptr(), silence_levels.len() as i32) }, 0);
        let levels = [0.02_f32, 0.02, 0.0, 0.0, 0.0, 0.0];
        let durations = [0.12_f64, 0.12, 0.30, 0.30, 0.30, 0.30];
        assert_eq!(unsafe { yonda_voice_vad_test(levels.as_ptr(), durations.as_ptr(), levels.len() as i32) }, 5);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_finish_gate_accepts_only_the_first_competing_signal() {
        assert_eq!(unsafe { yonda_voice_finish_gate_test(4) }, 1);
    }
}
