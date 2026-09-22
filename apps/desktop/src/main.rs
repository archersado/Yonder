use std::{ffi::{c_char, CStr}, sync::{Arc, Mutex, OnceLock}, time::{SystemTime, UNIX_EPOCH}};
use serde::Deserialize;
use tauri::{Manager, WebviewWindow, State, menu::{Menu, MenuItem}, tray::TrayIconBuilder};
use yonder_desktop::TaskHost;
use yonder_adapters::pet_pack;
mod pet_window;
mod voice_input;

struct TaskState(Arc<Mutex<Option<TaskHost>>>);
struct PreviewState(Mutex<yonder_application::region_preview::Session>);
const REGION_PREVIEW_TITLE: &str = "Yonda · 圈选提问";

fn region_preview_clean_title(reason: Option<&str>, event_at_ms: Option<u64>) -> String {
    let reason = match reason {
        Some("escape") => "escape", Some("toolbar-cancel") => "toolbar-cancel", Some("review-cancel") => "review-cancel",
        Some("timeout") => "timeout", Some("close-button") => "close-button", Some("app-switch") => "app-switch", Some("reselect-error") => "reselect-error",
        Some("review-error") => "review-error", Some("submitted") => "submitted", _ => "close",
    };
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |value| value.as_millis() as u64);
    format!("Yonda · 圈选提问 [idle image=0 selection=0 stroke=0 reason={reason} latency_ms={}]", event_at_ms.map_or(0, |at| now.saturating_sub(at)))
}

#[cfg(test)]
mod preview_title_tests {
    use super::*;
    #[test]
    fn cleanup_reason_is_bounded() {
        assert!(region_preview_clean_title(Some("escape"), None).contains("reason=escape"));
        assert!(region_preview_clean_title(Some("app-switch"), None).contains("reason=app-switch"));
        assert!(region_preview_clean_title(Some("untrusted"), None).contains("reason=close"));
    }
}

#[tauri::command]
fn pet_pack_assets(window: WebviewWindow) -> Result<pet_pack::ActivePetPack, String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    let packs = window.app_handle().path().app_data_dir().map_err(|_| "应用数据目录不可用")?.join("pet-packs");
    pet_pack::active_pack(&packs).map_err(|error| match error {
        pet_pack::PetPackError::LimitExceeded => "资源包无法安全导入".into(),
        pet_pack::PetPackError::Io => "当前资源包不可用".into(),
        _ => "资源包不符合规范".into(),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegionRect { x: f64, y: f64, width: f64, height: f64, viewport_width: f64, viewport_height: f64 }

#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn yonda_region_capture(x: i32, y: i32, width: i32, height: i32) -> *mut c_char;
    fn yonda_region_source_application() -> *mut c_char;
    fn yonda_region_watch_application_switch(callback: extern "C" fn());
    fn yonda_region_stop_application_switch_watch();
    fn yonda_region_watch_display_change(callback: extern "C" fn());
    fn yonda_region_stop_display_change_watch();
}

#[cfg(target_os = "macos")]
static REGION_PREVIEW_APP: OnceLock<tauri::AppHandle> = OnceLock::new();

#[cfg(target_os = "macos")]
extern "C" fn clear_region_preview_after_application_switch() {
    if let Some(app) = REGION_PREVIEW_APP.get() {
        let app = app.clone();
        std::thread::spawn(move || { let _ = clear_region_preview(&app, Some("app-switch"), None); });
    }
}

#[cfg(target_os = "macos")]
fn watch_region_application_switch() { unsafe { yonda_region_watch_application_switch(clear_region_preview_after_application_switch); } }

#[cfg(target_os = "macos")]
fn stop_region_application_switch_watch() { unsafe { yonda_region_stop_application_switch_watch(); } }

#[cfg(target_os = "macos")]
extern "C" fn clear_region_preview_after_display_change() {
    if let Some(app) = REGION_PREVIEW_APP.get() {
        let app = app.clone();
        std::thread::spawn(move || { let _ = clear_region_preview(&app, Some("display-change"), None); });
    }
}

#[cfg(target_os = "macos")]
fn watch_region_display_change() { unsafe { yonda_region_watch_display_change(clear_region_preview_after_display_change); } }

#[cfg(target_os = "macos")]
fn stop_region_display_change_watch() { unsafe { yonda_region_stop_display_change_watch(); } }

#[cfg(not(target_os = "macos"))]
fn watch_region_application_switch() {}

#[cfg(not(target_os = "macos"))]
fn stop_region_application_switch_watch() {}

#[cfg(not(target_os = "macos"))]
fn watch_region_display_change() {}

#[cfg(not(target_os = "macos"))]
fn stop_region_display_change_watch() {}

fn preview_error(error: &str) -> String { error.into() }

#[cfg(target_os = "macos")]
fn current_source_application() -> Option<String> {
    unsafe {
        let raw = yonda_region_source_application();
        if raw.is_null() { return None; }
        let value = CStr::from_ptr(raw).to_string_lossy().into_owned();
        yonda_region_free(raw.cast());
        Some(value)
    }
}

#[cfg(not(target_os = "macos"))]
fn current_source_application() -> Option<String> { None }

fn display_region_preview(app: &tauri::AppHandle, preview_state: &PreviewState) -> Result<(), String> {
    let pet = app.get_webview_window("pet").ok_or("小龙不可用")?;
    let preview = app.get_webview_window("region-preview").ok_or("圈选窗口不可用")?;
    let source = preview_state.0.lock().map_err(|_| preview_error("preview-unavailable"))?
        .source_application().unwrap_or("当前桌面").to_owned();
    let source = serde_json::to_string(&source).map_err(|_| preview_error("preview-unavailable"))?;
    let monitor = pet.current_monitor().map_err(|_| "屏幕不可用")?
        .or(pet.primary_monitor().map_err(|_| "屏幕不可用")?).ok_or("屏幕不可用")?;
    let area = monitor.work_area();
    let result = preview.set_title(REGION_PREVIEW_TITLE).and_then(|_| preview.set_position(area.position)).and_then(|_| preview.set_size(area.size))
        .and_then(|_| preview.eval(&format!("window.dispatchEvent(new CustomEvent('yonda-region-open',{{detail:{{sourceApplication:{source}}}}}))"))).and_then(|_| preview.show())
        .map_err(|_| { preview_state.0.lock().ok().map(|mut session| session.clear()); preview_error("preview-unavailable") });
    if result.is_ok() { let _ = preview.set_focus(); watch_region_application_switch(); watch_region_display_change(); }
    result
}

fn show_region_preview(app: &tauri::AppHandle, preview_state: &PreviewState) -> Result<(), String> {
    preview_state.0.lock().map_err(|_| preview_error("preview-unavailable"))?.begin(current_source_application())
        .map_err(|_| preview_error("preview-busy"))?;
    display_region_preview(app, preview_state)
}

async fn pause_for_region(state: Arc<Mutex<Option<TaskHost>>>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || state.lock().map_err(|_| "desktop-stop-unconfirmed".to_owned())?.as_mut().ok_or("desktop-stop-unconfirmed".to_owned())?.pause_desktop_for_user().map(|_|()).map_err(|_|"desktop-stop-unconfirmed".to_owned()))
        .await.map_err(|_| "desktop-stop-unconfirmed".to_owned())?
}

#[tauri::command]
async fn region_preview_open(window: WebviewWindow, state: State<'_, TaskState>, preview: State<'_, PreviewState>) -> Result<(), String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    pause_for_region(Arc::clone(&state.0)).await?;
    show_region_preview(window.app_handle(), &preview)
}

#[tauri::command]
fn region_preview_hide_for_capture(window: WebviewWindow, preview: State<'_, PreviewState>) -> Result<(), String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    let mut session = preview.0.lock().map_err(|_| "preview-unavailable")?;
    session.capture().map_err(|_| "preview-invalid-state")?;
    if window.hide().is_err() { session.clear(); return Err("圈选窗口关闭失败".into()); }
    Ok(())
}

#[tauri::command]
fn clear_region_preview(app: &tauri::AppHandle, reason: Option<&str>, event_at_ms: Option<u64>) -> Result<(), String> {
    stop_region_application_switch_watch();
    stop_region_display_change_watch();
    app.state::<voice_input::VoiceRuntime>().cancel(voice_input::VoiceTarget::Region);
    let window = app.get_webview_window("region-preview").ok_or("圈选窗口不可用")?;
    let preview = app.state::<PreviewState>();
    let mut session = preview.0.lock().map_err(|_| "preview-unavailable")?;
    if session.snapshot().0 == yonder_application::region_preview::Phase::Idle && !window.is_visible().unwrap_or(false) { return Ok(()); }
    session.clear();
    let result = window.eval("window.dispatchEvent(new Event('yonda-region-clear'))").and_then(|_| window.hide()).and_then(|_| {
        let title = region_preview_clean_title(reason, event_at_ms);
        window.set_title(&title)
    }).map_err(|_| "preview-unavailable".into());
    drop(session);
    result
}

#[tauri::command]
fn region_preview_close(window: WebviewWindow, reason: Option<String>, event_at_ms: Option<u64>) -> Result<(), String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    clear_region_preview(window.app_handle(), reason.as_deref(), event_at_ms)
}

#[tauri::command]
async fn region_preview_reselect(window: WebviewWindow, state: State<'_, TaskState>, preview: State<'_, PreviewState>, voice: State<'_, voice_input::VoiceRuntime>) -> Result<(), String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    voice.cancel(voice_input::VoiceTarget::Region);
    pause_for_region(Arc::clone(&state.0)).await?;
    {
        let mut session = preview.0.lock().map_err(|_| "preview-unavailable")?;
        if session.snapshot().0 == yonder_application::region_preview::Phase::Idle {
            session.begin(current_source_application()).map_err(|_| "preview-busy")?;
        } else {
            session.reselect().map_err(|_| "preview-invalid-state")?;
        }
    }
    display_region_preview(window.app_handle(), &preview)
}

#[cfg(target_os = "macos")]
fn capture_region_image(x: i32, y: i32, width: i32, height: i32) -> Result<String, String> {
    unsafe {
        let raw = yonda_region_capture(x, y, width, height);
        if raw.is_null() { return Err("需要允许 Yonda 录制屏幕后才能预览截图".into()); }
        let image = CStr::from_ptr(raw).to_string_lossy().into_owned();
        yonda_region_free(raw.cast());
        if image.is_empty() { return Err("截图不可用".into()); }
        Ok(image)
    }
}

#[tauri::command]
async fn region_preview_capture(window: WebviewWindow, rect: RegionRect, preview: State<'_, PreviewState>) -> Result<String, String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    if !(rect.viewport_width.is_finite() && rect.viewport_height.is_finite() && rect.x.is_finite() && rect.y.is_finite() && rect.width.is_finite() && rect.height.is_finite())
        || rect.viewport_width <= 0.0 || rect.viewport_height <= 0.0 || rect.width < 12.0 || rect.height < 12.0 { preview.0.lock().ok().map(|mut session| session.clear()); return Err("capture-failed".into()); }
    let monitor = window.current_monitor().map_err(|_| "屏幕不可用")?
        .or(window.primary_monitor().map_err(|_| "屏幕不可用")?).ok_or("屏幕不可用")?;
    let area = monitor.work_area();
    let x = f64::from(area.position.x) + rect.x.max(0.0) * f64::from(area.size.width) / rect.viewport_width;
    let y = f64::from(area.position.y) + rect.y.max(0.0) * f64::from(area.size.height) / rect.viewport_height;
    let width = (rect.width * f64::from(area.size.width) / rect.viewport_width).ceil();
    let height = (rect.height * f64::from(area.size.height) / rect.viewport_height).ceil();
    if x + width > f64::from(area.position.x + area.size.width as i32) || y + height > f64::from(area.position.y + area.size.height as i32)
        || width * height > 16_777_216.0 { preview.0.lock().ok().map(|mut session| session.clear()); return Err("capture-failed".into()); }
    if preview.0.lock().map_err(|_| "preview-unavailable")?.snapshot().0 != yonder_application::region_preview::Phase::Capturing { return Err("preview-invalid-state".into()); }
    #[cfg(target_os = "macos")]
    { let scale = monitor.scale_factor();
      let captured = match tokio::task::spawn_blocking(move || capture_region_image((x / scale).round() as i32, (y / scale).round() as i32, (width / scale).round() as i32, (height / scale).round() as i32)).await {
        Ok(captured) => captured,
        Err(_) => { preview.0.lock().ok().map(|mut session| session.clear()); return Err("capture-failed".into()); }
      };
      match captured {
        Ok(image) => match preview.0.lock().map_err(|_| "preview-unavailable")?.review(image) {
          Ok(image) => Ok(image),
          Err(yonder_application::region_preview::Error::ImageTooLarge) => Err("capture-too-large".into()),
          Err(_) => Err("capture-failed".into()),
        },
        Err(_) => { preview.0.lock().ok().and_then(|mut session| session.review_without_image().ok()); Err("permission-required".into()) }
      }
    }
    #[cfg(not(target_os = "macos"))]
    { let _ = (x, y, width, height); Err("Windows 圈选预览仍在验证中".into()) }
}

#[tauri::command]
async fn region_preview_submit(window: WebviewWindow, question: String, preview: State<'_, PreviewState>, hub: State<'_, yonder_desktop::agent_input::AgentInputHub>) -> Result<&'static str, String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    let image = preview.0.lock().map_err(|_| "preview-unavailable")?.begin_submission().map_err(|_| "preview-invalid-state")?;
    let hub = hub.inner().clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || match image { Some(image) => hub.deliver_selection(&question, &image), None => hub.deliver_selection_text(&question) }).await;
    preview.0.lock().map_err(|_| "preview-unavailable")?.clear();
    stop_region_application_switch_watch();
    stop_region_display_change_watch();
    match outcome.map_err(|_| "delivery-unavailable")? {
        Ok(yonder_application::agent_input::DeliveryOutcome::Accepted) => Ok("accepted"),
        Ok(yonder_application::agent_input::DeliveryOutcome::Rejected) => Ok("rejected"),
        Ok(yonder_application::agent_input::DeliveryOutcome::Unknown) => Ok("unknown"),
        Err("当前Agent不支持截图提问") => Err("attachment-unsupported".into()),
        Err("当前没有可接收输入的Agent") => Err("agent-unavailable".into()),
        Err("请选择接收输入的Agent") => Err("agent-target-required".into()),
        Err("截图数据无效") => Err("attachment-invalid".into()),
        Err(_) => Err("delivery-unavailable".into()),
    }
}

#[tauri::command]
fn region_preview_text_only(window: WebviewWindow, preview: State<'_, PreviewState>) -> Result<(), String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    preview.0.lock().map_err(|_| "preview-unavailable")?.review_without_image().map_err(|_| "preview-invalid-state".into())
}

#[tauri::command]
fn region_preview_show_review(window: WebviewWindow, rect: RegionRect, has_image: bool) -> Result<(), String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    let monitor = window.current_monitor().map_err(|_| "屏幕不可用")?
        .or(window.primary_monitor().map_err(|_| "屏幕不可用")?).ok_or("屏幕不可用")?;
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let physical_size = tauri::PhysicalSize::new((440.0 * scale).round() as u32, ((if has_image { 560.0 } else { 350.0 }) * scale).round() as u32);
    let x = (f64::from(area.position.x) + rect.x * f64::from(area.size.width) / rect.viewport_width + 12.0).round() as i32;
    let y = (f64::from(area.position.y) + rect.y * f64::from(area.size.height) / rect.viewport_height + 12.0).round() as i32;
    let max_x = area.position.x.saturating_add(area.size.width.saturating_sub(physical_size.width) as i32);
    let max_y = area.position.y.saturating_add(area.size.height.saturating_sub(physical_size.height) as i32);
    window.set_size(physical_size).and_then(|_| window.set_position(tauri::PhysicalPosition::new(x.clamp(area.position.x, max_x), y.clamp(area.position.y, max_y))))
        .and_then(|_| window.show()).and_then(|_| window.set_focus()).map_err(|_| "确认卡打开失败".into())
}

fn show_region_feedback(app: &tauri::AppHandle, preview: &PreviewState, error: &str) {
    stop_region_application_switch_watch();
    stop_region_display_change_watch();
    preview.0.lock().ok().map(|mut session| session.clear());
    if let Some(window) = app.get_webview_window("region-preview") {
        let message = serde_json::to_string(error).unwrap_or_else(|_| "\"preview-unavailable\"".into());
        let _ = window.set_title(REGION_PREVIEW_TITLE).and_then(|_| window.set_size(tauri::LogicalSize::new(440.0, 240.0))).and_then(|_| window.eval(&format!("window.dispatchEvent(new CustomEvent('yonda-region-error',{{detail:{message}}}))"))).and_then(|_| window.show());
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" { fn yonda_region_free(pointer: *mut std::ffi::c_void); }

#[tauri::command]
fn pet_is_visible(window: WebviewWindow) -> Result<bool, String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    Ok(window.is_visible().map_err(|_| "可见性不可用")? && !window.is_minimized().map_err(|_| "可见性不可用")?)
}

#[tauri::command]
async fn pet_task_state(window: WebviewWindow, state: State<'_, TaskState>) -> Result<(bool, &'static str), String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    let host = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || {
        host.lock().map_err(|_| "任务状态不可用")?.as_mut()
            .ok_or("任务状态不可用")?.presentation().map_err(|_| "任务状态不可用".to_owned())
    }).await.map_err(|_| "任务状态查询中断".to_owned())?
}

#[tauri::command]
fn pet_agent_connected(window: WebviewWindow, hub: State<'_, yonder_desktop::agent_input::AgentInputHub>) -> Result<bool, String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    Ok(hub.connected())
}

fn show_menu(app: &tauri::AppHandle) -> Result<(), String> {
    let pet = app.get_webview_window("pet").ok_or("小龙不可用")?;
    let menu = app.get_webview_window("task-space").ok_or("任务菜单不可用")?;
    let monitor = pet.current_monitor().map_err(|_| "屏幕不可用")?
        .or(pet.primary_monitor().map_err(|_| "屏幕不可用")?).ok_or("屏幕不可用")?;
    let area = monitor.work_area();
    let position = pet.outer_position().map_err(|_| "位置不可用")?;
    let pet_size = pet.outer_size().map_err(|_| "尺寸不可用")?;
    let size = menu.outer_size().map_err(|_| "菜单尺寸不可用")?;
    let gap = (8.0 * monitor.scale_factor()) as i32;
    let below = position.y + pet_size.height as i32 + gap;
    let y = if i64::from(below) + i64::from(size.height) <= i64::from(area.position.y) + i64::from(area.size.height) {
        below
    } else { position.y - size.height as i32 - gap };
    let max_x = area.position.x.saturating_add(area.size.width.saturating_sub(size.width) as i32);
    let max_y = area.position.y.saturating_add(area.size.height.saturating_sub(size.height) as i32);
    let target = tauri::PhysicalPosition::new(position.x.clamp(area.position.x, max_x), y.clamp(area.position.y, max_y));
    menu.set_position(target).and_then(|_| menu.show()).and_then(|_| menu.set_focus())
        .and_then(|_| menu.eval("window.dispatchEvent(new Event('yonda-tasks-open'))"))
        .map_err(|_| "任务菜单打开失败".into())
}

fn show_voice_input(app: &tauri::AppHandle) -> Result<(), String> {
    let pet = app.get_webview_window("pet").ok_or("小龙不可用")?;
    let card = app.get_webview_window("voice-input").ok_or("语音卡不可用")?;
    let _ = app.get_webview_window("task-space").map(|window| window.hide());
    let monitor = pet.current_monitor().map_err(|_| "屏幕不可用")?
        .or(pet.primary_monitor().map_err(|_| "屏幕不可用")?).ok_or("屏幕不可用")?;
    let area = monitor.work_area();
    let position = pet.outer_position().map_err(|_| "位置不可用")?;
    let pet_size = pet.outer_size().map_err(|_| "尺寸不可用")?;
    let size = card.outer_size().map_err(|_| "语音卡尺寸不可用")?;
    let gap = (8.0 * monitor.scale_factor()) as i32;
    let right = position.x + pet_size.width as i32 + gap;
    let x = if i64::from(right) + i64::from(size.width) <= i64::from(area.position.x) + i64::from(area.size.width) {
        right
    } else { position.x - size.width as i32 - gap };
    let max_y = area.position.y.saturating_add(area.size.height.saturating_sub(size.height) as i32);
    card.set_position(tauri::PhysicalPosition::new(x, position.y.clamp(area.position.y, max_y)))
        .and_then(|_| card.show()).and_then(|_| card.set_focus())
        .and_then(|_| card.eval("window.dispatchEvent(new Event('yonda-voice-open'))"))
        .map_err(|_| "语音卡打开失败".into())
}

fn show_jev_settings(app: &tauri::AppHandle) -> Result<(), String> {
    let settings = app
        .get_webview_window("jev-settings")
        .ok_or("Jev 设置窗口不可用")?;
    settings
        .show()
        .and_then(|_| settings.set_focus())
        .and_then(|_| settings.eval("window.dispatchEvent(new Event('yonda-jev-open'))"))
        .map_err(|_| "Jev 设置窗口打开失败".into())
}

fn show_agent_settings(app: &tauri::AppHandle) -> Result<(), String> {
    let settings = app
        .get_webview_window("agent-settings")
        .ok_or("Agent管理窗口不可用")?;
    settings
        .show()
        .and_then(|_| settings.set_focus())
        .and_then(|_| settings.eval("window.dispatchEvent(new Event('yonda-agent-open'))"))
        .map_err(|_| "Agent管理窗口打开失败".into())
}

fn unix_now_ms() -> Result<u64, String> {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "系统时间不可用")?
            .as_millis(),
    )
    .map_err(|_| "系统时间不可用".to_owned())
}

#[tauri::command]
fn voice_input_open(window: WebviewWindow, state: State<'_, voice_input::VoiceRuntime>) -> Result<(), String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    show_voice_input(window.app_handle())?;
    state.start(voice_input::VoiceTarget::Direct)
}

#[tauri::command]
fn voice_input_start(window: WebviewWindow, state: State<'_, voice_input::VoiceRuntime>) -> Result<(), String> {
    if window.label() != "voice-input" { return Err("不允许的窗口".into()); }
    state.start(voice_input::VoiceTarget::Direct)
}

#[tauri::command]
fn voice_input_stop(window: WebviewWindow, state: State<'_, voice_input::VoiceRuntime>) -> Result<(), String> {
    if window.label() != "voice-input" { return Err("不允许的窗口".into()); }
    state.stop(voice_input::VoiceTarget::Direct)
}

#[tauri::command]
fn voice_input_close(window: WebviewWindow, state: State<'_, voice_input::VoiceRuntime>) -> Result<(), String> {
    if window.label() != "voice-input" { return Err("不允许的窗口".into()); }
    state.cancel(voice_input::VoiceTarget::Direct);
    window.hide().map_err(|_| "语音卡关闭失败".into())
}

#[tauri::command]
fn region_voice_start(window: WebviewWindow, state: State<'_, voice_input::VoiceRuntime>) -> Result<(), String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    state.start(voice_input::VoiceTarget::Region)
}

#[tauri::command]
fn region_voice_stop(window: WebviewWindow, state: State<'_, voice_input::VoiceRuntime>) -> Result<(), String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    state.stop(voice_input::VoiceTarget::Region)
}

#[tauri::command]
fn voice_input_phase(window: WebviewWindow, state: State<'_, voice_input::VoiceRuntime>) -> Result<&'static str, String> {
    if window.label() != "voice-input" { return Err("不允许的窗口".into()); }
    Ok(state.phase())
}

#[tauri::command]
async fn task_menu_show(window: WebviewWindow, state: State<'_, TaskState>) -> Result<bool, String> {
    let (has_tasks, _) = pet_task_state(window.clone(), state).await?;
    if !has_tasks { return Ok(false); }
    show_menu(window.app_handle())?;
    Ok(true)
}

#[tauri::command]
fn task_menu_hide(window: WebviewWindow) -> Result<(), String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    window.app_handle().get_webview_window("task-space").ok_or("任务菜单不可用")?
        .hide().map_err(|_| "菜单收起失败".into())
}

#[tauri::command]
fn task_menu_close(window: WebviewWindow) -> Result<(), String> {
    if window.label() != "task-space" { return Err("不允许的窗口".into()); }
    window.hide().map_err(|_| "菜单收起失败".into())
}

#[tauri::command]
async fn task_query(window: WebviewWindow, state: State<'_, TaskState>, request: String) -> Result<String, String> {
    if window.label() != "task-space" { return Err("不允许的窗口".into()); }
    if yonder_application::gateway::is_takeover_request(request.as_bytes()) { return Err("接管必须使用本地用户入口".into()); }
    let host = Arc::clone(&state.0); let pet = window.app_handle().get_webview_window("pet");
    let (response, presentation) = tauri::async_runtime::spawn_blocking(move || {
        let mut host = host.lock().map_err(|_| "任务存储不可用")?;
        let host = host.as_mut().ok_or("任务存储未就绪，请退出后重试")?;
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| "系统时间不可用")?.as_millis();
        let response = host.query(request.as_bytes(), u64::try_from(now).map_err(|_| "系统时间不可用")?)
            .map_err(|_| "任务读取失败")?;
        let response = String::from_utf8(response).map_err(|_| "任务响应不可用".to_owned())?;
        Ok::<_, String>((response, host.presentation().ok()))
    }).await.map_err(|_| "任务查询中断".to_owned())??;
    if let (Some(pet), Some((has_tasks, task_state))) = (pet, presentation) {
        yonder_desktop::emit_pet_presentation(&pet, has_tasks, task_state);
    }
    Ok(response)
}

#[tauri::command]
async fn user_takeover(window:WebviewWindow,state:State<'_,TaskState>,task_id:String,expected_sequence:String)->Result<String,String>{
    if window.label()!="task-space"{return Err("不允许的窗口".into())}
    let expected=expected_sequence.parse::<u64>().map_err(|_|"任务序号不可用")?;
    let host=Arc::clone(&state.0);let pet=window.app_handle().get_webview_window("pet");
    let (response,presentation)=tauri::async_runtime::spawn_blocking(move||{
        let now=u64::try_from(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|_|"系统时间不可用")?.as_millis()).map_err(|_|"系统时间不可用")?;
        let mut host=host.lock().map_err(|_|"任务存储不可用")?;let host=host.as_mut().ok_or("任务存储未就绪，请退出后重试")?;
        let response=String::from_utf8(host.user_takeover(&task_id,expected,now).map_err(|_|"接管失败")?).map_err(|_|"任务响应不可用")?;
        Ok::<_,String>((response,host.presentation().ok()))
    }).await.map_err(|_|"接管中断".to_owned())??;
    if let(Some(pet),Some((has_tasks,task_state)))=(pet,presentation){yonder_desktop::emit_pet_presentation(&pet,has_tasks,task_state);}
    Ok(response)
}

#[tauri::command]
async fn browser_task_space_open(window:WebviewWindow,state:State<'_,TaskState>,task_id:String,expected_sequence:String)->Result<(),String>{
    if window.label()!="task-space"{return Err("不允许的窗口".into())}
    let expected=expected_sequence.parse::<u64>().map_err(|_|"任务序号不可用")?;
    let host=Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move||{
        host.lock().map_err(|_|"任务存储不可用")?.as_mut().ok_or("任务存储未就绪，请退出后重试")?
            .user_browser_handoff(&task_id,expected).map_err(|_|"无法打开对应的 ego-lite Task Space".to_owned())
    }).await.map_err(|_|"ego-lite 交接中断".to_owned())??;
    window.hide().map_err(|_|"任务菜单收起失败".to_owned())
}

#[tauri::command]
async fn jev_config_get(
    window: WebviewWindow,
    state: State<'_, TaskState>,
) -> Result<yonder_application::jev_config::JevConfig, String> {
    if window.label() != "jev-settings" {
        return Err("不允许的窗口".into());
    }
    let host = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || {
        host.lock()
            .map_err(|_| "任务存储不可用")?
            .as_mut()
            .ok_or("任务存储未就绪，请退出后重试")?
            .jev_config()
            .map_err(|_| "Jev 配置读取失败".to_owned())
    })
    .await
    .map_err(|_| "Jev 配置读取中断".to_owned())?
}

#[tauri::command]
async fn jev_config_save(
    window: WebviewWindow,
    state: State<'_, TaskState>,
    config: yonder_application::jev_config::JevConfig,
) -> Result<yonder_application::jev_config::JevConfig, String> {
    if window.label() != "jev-settings" {
        return Err("不允许的窗口".into());
    }
    let host = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || {
        host.lock()
            .map_err(|_| "任务存储不可用")?
            .as_mut()
            .ok_or("任务存储未就绪，请退出后重试")?
            .save_jev_config(config)
            .map_err(|_| "Jev 配置无效或保存失败".to_owned())
    })
    .await
    .map_err(|_| "Jev 配置保存中断".to_owned())?
}

#[tauri::command]
fn jev_settings_close(window: WebviewWindow) -> Result<(), String> {
    if window.label() != "jev-settings" {
        return Err("不允许的窗口".into());
    }
    window
        .hide()
        .map_err(|_| "Jev 设置窗口关闭失败".to_owned())
}

#[tauri::command]
async fn agent_registry_list(
    window: WebviewWindow,
    state: State<'_, TaskState>,
) -> Result<Vec<yonder_application::agent_registry::AgentRegistration>, String> {
    if window.label() != "agent-settings" {
        return Err("不允许的窗口".into());
    }
    let host = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || {
        host.lock()
            .map_err(|_| "任务存储不可用")?
            .as_mut()
            .ok_or("任务存储未就绪，请退出后重试")?
            .list_agents()
            .map_err(|_| "Agent列表读取失败".to_owned())
    })
    .await
    .map_err(|_| "Agent列表读取中断".to_owned())?
}

#[tauri::command]
async fn agent_registry_register(
    window: WebviewWindow,
    state: State<'_, TaskState>,
    agent_id: String,
) -> Result<yonder_application::agent_registry::AgentRegistration, String> {
    if window.label() != "agent-settings" {
        return Err("不允许的窗口".into());
    }
    let host = Arc::clone(&state.0);
    tauri::async_runtime::spawn_blocking(move || {
        let now = unix_now_ms()?;
        host.lock()
            .map_err(|_| "任务存储不可用")?
            .as_mut()
            .ok_or("任务存储未就绪，请退出后重试")?
            .register_agent(&agent_id, now)
            .map_err(|_| "Agent ID无效或登记失败".to_owned())
    })
    .await
    .map_err(|_| "Agent登记中断".to_owned())?
}

#[tauri::command]
async fn agent_registry_set_status(
    window: WebviewWindow,
    state: State<'_, TaskState>,
    hub: State<'_, yonder_desktop::agent_input::AgentInputHub>,
    agent_id: String,
    status: yonder_application::agent_registry::AgentRegistrationStatus,
) -> Result<yonder_application::agent_registry::AgentRegistration, String> {
    if window.label() != "agent-settings" {
        return Err("不允许的窗口".into());
    }
    let host = Arc::clone(&state.0);
    let result = tauri::async_runtime::spawn_blocking(move || {
        let now = unix_now_ms()?;
        host.lock()
            .map_err(|_| "任务存储不可用")?
            .as_mut()
            .ok_or("任务存储未就绪，请退出后重试")?
            .set_agent_status(&agent_id, status, now)
            .map_err(|_| "Agent状态更新失败".to_owned())
    })
    .await
    .map_err(|_| "Agent状态更新中断".to_owned())??;
    if result.status != yonder_application::agent_registry::AgentRegistrationStatus::Enabled {
        hub.disconnect_agent(&result.agent_id);
    }
    Ok(result)
}

#[tauri::command]
fn agent_settings_close(window: WebviewWindow) -> Result<(), String> {
    if window.label() != "agent-settings" {
        return Err("不允许的窗口".into());
    }
    window
        .hide()
        .map_err(|_| "Agent管理窗口关闭失败".to_owned())
}

fn main() {
    tauri::Builder::default()
        .manage(pet_window::PetWindowState::default())
        .invoke_handler(tauri::generate_handler![task_query, user_takeover, browser_task_space_open, jev_config_get, jev_config_save, jev_settings_close, agent_registry_list, agent_registry_register, agent_registry_set_status, agent_settings_close, task_menu_show, task_menu_hide, task_menu_close, pet_is_visible, pet_task_state, pet_agent_connected, pet_pack_assets, pet_window::pet_dock, pet_window::pet_wake, voice_input_open, voice_input_start, voice_input_stop, voice_input_close, voice_input_phase, region_voice_start, region_voice_stop, region_preview_open, region_preview_hide_for_capture, region_preview_capture, region_preview_show_review, region_preview_text_only, region_preview_submit, region_preview_close, region_preview_reselect])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let pet = app.get_webview_window("pet").ok_or("小龙窗口未创建")?;
            app.get_webview_window("task-space").ok_or("任务菜单未创建")?.hide()?;
            app.get_webview_window("voice-input").ok_or("语音卡未创建")?.hide()?;
            app.get_webview_window("region-preview").ok_or("圈选窗口未创建")?.hide()?;
            app.get_webview_window("jev-settings").ok_or("Jev 设置窗口未创建")?.hide()?;
            app.get_webview_window("agent-settings").ok_or("Agent管理窗口未创建")?.hide()?;
            #[cfg(target_os = "macos")]
            {
                use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior as Behavior};
                // 小龙与任务菜单采用相同Space/全屏辅助行为。
                for window in [pet.clone(), app.get_webview_window("task-space").ok_or("任务菜单未创建")?, app.get_webview_window("voice-input").ok_or("语音卡未创建")?, app.get_webview_window("region-preview").ok_or("圈选窗口未创建")?, app.get_webview_window("jev-settings").ok_or("Jev 设置窗口未创建")?, app.get_webview_window("agent-settings").ok_or("Agent管理窗口未创建")?] {
                let native = unsafe { &*window.ns_window()?.cast::<NSWindow>() };
                let mut behavior = native.collectionBehavior();
                if objc2::available!(macos = 13.0) {
                    behavior = (behavior & !(Behavior::Primary | Behavior::Auxiliary)) | Behavior::CanJoinAllApplications;
                }
                native.setCollectionBehavior((behavior & !(Behavior::FullScreenPrimary | Behavior::FullScreenNone | Behavior::FullScreenAllowsTiling))
                    | Behavior::FullScreenAuxiliary | Behavior::FullScreenDisallowsTiling);
                }
            }
            pet.center()?;
            pet.show()?;
            let host = app.path().app_data_dir().ok().and_then(|path| TaskHost::open(&path).ok());
            let host = Arc::new(Mutex::new(host));
            app.manage(TaskState(Arc::clone(&host)));
            app.manage(PreviewState(Mutex::new(yonder_application::region_preview::Session::default())));
            #[cfg(target_os = "macos")]
            let _ = REGION_PREVIEW_APP.set(app.handle().clone());
            let input_hub = yonder_desktop::agent_input::AgentInputHub::default();
            app.manage(voice_input::VoiceRuntime::new(app.handle().clone(), input_hub.clone()));
            app.manage(input_hub.clone());
            #[cfg(target_os = "macos")]
            app.manage(yonder_desktop::local_agent_socket::Server::start(
                host.clone(), app.path().app_data_dir()?.join("agent.sock"), pet.clone(), input_hub
            )?);
            if cfg!(all(debug_assertions, target_os = "macos")) && std::env::args_os().any(|arg| arg == "--local-agent-stdio") {
                // AD-AG-03：父进程私有管道，仅显式研发启动；不等待阻塞读线程退出。
                std::thread::Builder::new().name("local-agent-stdio".into()).spawn(move || {
                    if yonder_desktop::local_agent_stdio::serve(host).is_err() {
                        eprintln!("本地Agent研发连接关闭");
                    }
                })?;
            }
            let show = MenuItem::with_id(app, "tasks", "任务总览", true, None::<&str>)?;
            let region = MenuItem::with_id(app, "region", "圈选提问（预览）", true, None::<&str>)?;
            let import = MenuItem::with_id(app, "pet-pack-import", "导入桌宠资源包", true, None::<&str>)?;
            let jev = MenuItem::with_id(app, "jev", "Jev 快脑设置", true, None::<&str>)?;
            let agents = MenuItem::with_id(app, "agents", "Agent 管理", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出任务面板", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &region, &import, &jev, &agents, &quit])?;
            let mut rgba = vec![0u8; 16 * 16 * 4];
            for y in 3..13 { for x in 3..13 {
                if y <= 4 || (7..=8).contains(&x) { rgba[(y * 16 + x) * 4 + 3] = 255; }
            }}
            TrayIconBuilder::with_id("task-space-tray")
                .icon(tauri::image::Image::new_owned(rgba, 16, 16)).icon_as_template(true)
                .tooltip("Yonda 任务总览").menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "tasks" => { let _ = show_menu(app); },
                    "region" => { let app=app.clone(); tauri::async_runtime::spawn(async move { let state=Arc::clone(&app.state::<TaskState>().0); let result=pause_for_region(state).await; let preview=app.state::<PreviewState>(); if let Err(error)=result.and_then(|_|show_region_preview(&app,&preview)){show_region_feedback(&app,&preview,&error);} }); },
                    "pet-pack-import" => {
                        let Some(path) = rfd::FileDialog::new().add_filter("桌宠资源包", &["zip"]).pick_file() else { return };
                        let Some(packs) = app.path().app_data_dir().ok().map(|path| path.join("pet-packs")) else { return };
                        let Some(pet) = app.get_webview_window("pet") else { return };
                        let _ = pet.eval("window.dispatchEvent(new CustomEvent('yonda-pet-pack-status',{detail:'validating'}))");
                        let detail = match pet_pack::import_pet_pack(&path, &packs) {
                            Ok(()) => "success",
                            Err(pet_pack::PetPackError::LimitExceeded) => "unsafe",
                            Err(pet_pack::PetPackError::Io) => "unavailable",
                            Err(_) => "invalid",
                        };
                        let _ = pet.eval(&format!("window.dispatchEvent(new CustomEvent('yonda-pet-pack-status',{{detail:'{detail}'}}))"));
                    },
                    "jev" => { let _ = show_jev_settings(app); },
                    "agents" => { let _ = show_agent_settings(app); },
                    "quit" => app.exit(0),
                    _ => {}
                }).build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "task-space" || window.label() == "voice-input" || window.label() == "region-preview" || window.label() == "jev-settings" || window.label() == "agent-settings" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if window.label() == "voice-input" { window.app_handle().state::<voice_input::VoiceRuntime>().cancel(voice_input::VoiceTarget::Direct); }
                    if window.label() == "region-preview" { let _ = clear_region_preview(window.app_handle(), Some("close"), None); }
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("Yonda任务面板启动失败");
}
