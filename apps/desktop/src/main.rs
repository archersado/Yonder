use std::{ffi::{c_char, CStr}, sync::{Arc, Mutex}};
use serde::Deserialize;
use tauri::{Manager, WebviewWindow, State, menu::{Menu, MenuItem}, tray::TrayIconBuilder};
use yonder_desktop::TaskHost;
mod pet_window;
mod voice_input;

struct TaskState(Arc<Mutex<Option<TaskHost>>>);
struct PreviewState(Mutex<yonder_application::region_preview::Session>);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegionRect { x: f64, y: f64, width: f64, height: f64, viewport_width: f64, viewport_height: f64 }

#[cfg(target_os = "macos")]
unsafe extern "C" { fn yonda_region_capture(x: i32, y: i32, width: i32, height: i32) -> *mut c_char; }

fn desktop_control_active(state: &TaskState) -> Result<bool, String> {
    state.0.lock().map_err(|_| "桌面状态不可用")?.as_ref()
        .ok_or("任务状态未就绪")?.desktop_control_active().map_err(|_| "桌面状态不可用".into())
}

fn preview_error(error: &str) -> String { error.into() }

fn show_region_preview(app: &tauri::AppHandle, state: &TaskState, preview_state: &PreviewState) -> Result<(), String> {
    if desktop_control_active(state)? { return Err(preview_error("desktop-control-active")); }
    preview_state.0.lock().map_err(|_| preview_error("preview-unavailable"))?.begin()
        .map_err(|_| preview_error("preview-busy"))?;
    let pet = app.get_webview_window("pet").ok_or("小龙不可用")?;
    let preview = app.get_webview_window("region-preview").ok_or("圈选窗口不可用")?;
    let monitor = pet.current_monitor().map_err(|_| "屏幕不可用")?
        .or(pet.primary_monitor().map_err(|_| "屏幕不可用")?).ok_or("屏幕不可用")?;
    let area = monitor.work_area();
    preview.set_position(area.position).and_then(|_| preview.set_size(area.size))
        .and_then(|_| preview.eval("window.dispatchEvent(new Event('yonda-region-open'))")).and_then(|_| preview.show()).and_then(|_| preview.set_focus())
        .map_err(|_| { preview_state.0.lock().ok().map(|mut session| session.clear()); preview_error("preview-unavailable") })
}

#[tauri::command]
fn region_preview_open(window: WebviewWindow, state: State<'_, TaskState>, preview: State<'_, PreviewState>) -> Result<(), String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    show_region_preview(window.app_handle(), &state, &preview)
}

#[tauri::command]
fn region_preview_hide_for_capture(window: WebviewWindow) -> Result<(), String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    window.hide().map_err(|_| "圈选窗口关闭失败".into())
}

#[tauri::command]
fn region_preview_close(window: WebviewWindow, preview: State<'_, PreviewState>) -> Result<(), String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    preview.0.lock().map_err(|_| "preview-unavailable")?.clear();
    window.eval("window.dispatchEvent(new Event('yonda-region-clear'))").and_then(|_| window.hide()).map_err(|_| "preview-unavailable".into())
}

#[tauri::command]
fn region_preview_reselect(window: WebviewWindow, state: State<'_, TaskState>, preview: State<'_, PreviewState>) -> Result<(), String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    preview.0.lock().map_err(|_| "preview-unavailable")?.clear();
    show_region_preview(window.app_handle(), &state, &preview)
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
    preview.0.lock().map_err(|_| "preview-unavailable")?.capture().map_err(|_| "preview-invalid-state")?;
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
        Err(_) => { preview.0.lock().ok().map(|mut session| session.clear()); Err("permission-required".into()) }
      }
    }
    #[cfg(not(target_os = "macos"))]
    { let _ = (x, y, width, height); Err("Windows 圈选预览仍在验证中".into()) }
}

#[tauri::command]
fn region_preview_show_review(window: WebviewWindow, rect: RegionRect) -> Result<(), String> {
    if window.label() != "region-preview" { return Err("不允许的窗口".into()); }
    let monitor = window.current_monitor().map_err(|_| "屏幕不可用")?
        .or(window.primary_monitor().map_err(|_| "屏幕不可用")?).ok_or("屏幕不可用")?;
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let physical_size = tauri::PhysicalSize::new((440.0 * scale).round() as u32, (560.0 * scale).round() as u32);
    let x = (f64::from(area.position.x) + rect.x * f64::from(area.size.width) / rect.viewport_width + 12.0).round() as i32;
    let y = (f64::from(area.position.y) + rect.y * f64::from(area.size.height) / rect.viewport_height + 12.0).round() as i32;
    let max_x = area.position.x.saturating_add(area.size.width.saturating_sub(physical_size.width) as i32);
    let max_y = area.position.y.saturating_add(area.size.height.saturating_sub(physical_size.height) as i32);
    window.set_size(physical_size).and_then(|_| window.set_position(tauri::PhysicalPosition::new(x.clamp(area.position.x, max_x), y.clamp(area.position.y, max_y))))
        .and_then(|_| window.show()).map_err(|_| "确认卡打开失败".into())
}

fn show_region_feedback(app: &tauri::AppHandle, preview: &PreviewState, error: &str) {
    preview.0.lock().ok().map(|mut session| session.clear());
    if let Some(window) = app.get_webview_window("region-preview") {
        let message = serde_json::to_string(error).unwrap_or_else(|_| "\"preview-unavailable\"".into());
        let _ = window.set_size(tauri::LogicalSize::new(440.0, 240.0)).and_then(|_| window.eval(&format!("window.dispatchEvent(new CustomEvent('yonda-region-error',{{detail:{message}}}))"))).and_then(|_| window.show());
    }
}

#[cfg(target_os = "macos")]
unsafe extern "C" { fn yonda_region_free(pointer: *mut std::ffi::c_void); }

#[tauri::command]
fn pet_is_visible(window: WebviewWindow) -> Result<bool, String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    Ok(window.is_visible().map_err(|_| "可见性不可用")? && !window.is_minimized().map_err(|_| "可见性不可用")?)
}

fn cursor_inside(window: &WebviewWindow) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::NSWindow;
        let native = unsafe { &*window.ns_window().map_err(|_| "小龙不可用")?.cast::<NSWindow>() };
        // 使用同一原生窗口的逻辑坐标，避免全局坐标混用Retina像素和点。
        let cursor = native.mouseLocationOutsideOfEventStream();
        let size = native.frame().size;
        return Ok(cursor.x >= 0.0 && cursor.y >= 0.0 && cursor.x < size.width && cursor.y < size.height);
    }
    #[cfg(not(target_os = "macos"))]
    {
    let cursor = window.cursor_position().map_err(|_| "鼠标位置不可用")?;
    let position = window.outer_position().map_err(|_| "位置不可用")?;
    let size = window.outer_size().map_err(|_| "尺寸不可用")?;
    Ok(cursor.x >= f64::from(position.x) && cursor.y >= f64::from(position.y)
        && cursor.x < f64::from(position.x) + f64::from(size.width)
        && cursor.y < f64::from(position.y) + f64::from(size.height))
    }
}

#[tauri::command]
fn pet_hover_region(window: WebviewWindow) -> Result<(bool, bool), String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    let menu = window.app_handle().get_webview_window("task-space").ok_or("菜单不可用")?;
    Ok((cursor_inside(&window)?, menu.is_visible().map_err(|_| "菜单不可用")? && cursor_inside(&menu)?))
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

fn show_menu(app: &tauri::AppHandle, focus: bool) -> Result<(), String> {
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
    menu.set_position(target).and_then(|_| menu.show()).and_then(|_| if focus { menu.set_focus() } else { Ok(()) })
        .and_then(|_| menu.eval("window.dispatchEvent(new Event('yonda-tasks-open'))"))
        .and_then(|_| pet.eval(if focus {
            "window.dispatchEvent(new CustomEvent('yonda-menu-open', {detail:'manual'}))"
        } else { "window.dispatchEvent(new CustomEvent('yonda-menu-open', {detail:'hover'}))" }))
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

#[tauri::command]
fn voice_input_open(window: WebviewWindow) -> Result<(), String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    show_voice_input(window.app_handle())?;
    voice_input::start()
}

#[tauri::command]
fn voice_input_start(window: WebviewWindow) -> Result<(), String> {
    if window.label() != "voice-input" { return Err("不允许的窗口".into()); }
    voice_input::start()
}

#[tauri::command]
fn voice_input_stop(window: WebviewWindow) -> Result<(), String> {
    if window.label() != "voice-input" { return Err("不允许的窗口".into()); }
    voice_input::stop(); Ok(())
}

#[tauri::command]
fn voice_input_close(window: WebviewWindow) -> Result<(), String> {
    if window.label() != "voice-input" { return Err("不允许的窗口".into()); }
    voice_input::cancel();
    window.hide().map_err(|_| "语音卡关闭失败".into())
}

#[tauri::command]
fn voice_input_phase(window: WebviewWindow, state: State<'_, voice_input::VoiceRuntime>) -> Result<&'static str, String> {
    if window.label() != "voice-input" { return Err("不允许的窗口".into()); }
    Ok(state.phase())
}

#[tauri::command]
async fn task_menu_show(window: WebviewWindow, state: State<'_, TaskState>, focus: bool) -> Result<bool, String> {
    let (has_tasks, _) = pet_task_state(window.clone(), state).await?;
    if !has_tasks { return Ok(false); }
    show_menu(window.app_handle(), focus)?;
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

fn main() {
    tauri::Builder::default()
        .manage(pet_window::PetWindowState::default())
        .invoke_handler(tauri::generate_handler![task_query, user_takeover, browser_task_space_open, task_menu_show, task_menu_hide, task_menu_close, pet_is_visible, pet_hover_region, pet_task_state, pet_agent_connected, pet_window::pet_dock, pet_window::pet_wake, voice_input_open, voice_input_start, voice_input_stop, voice_input_close, voice_input_phase, region_preview_open, region_preview_hide_for_capture, region_preview_capture, region_preview_show_review, region_preview_close, region_preview_reselect])
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            let pet = app.get_webview_window("pet").ok_or("小龙窗口未创建")?;
            app.get_webview_window("task-space").ok_or("任务菜单未创建")?.hide()?;
            app.get_webview_window("voice-input").ok_or("语音卡未创建")?.hide()?;
            app.get_webview_window("region-preview").ok_or("圈选窗口未创建")?.hide()?;
            #[cfg(target_os = "macos")]
            {
                use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior as Behavior};
                // 小龙与任务菜单采用相同Space/全屏辅助行为。
                for window in [pet.clone(), app.get_webview_window("task-space").ok_or("任务菜单未创建")?, app.get_webview_window("voice-input").ok_or("语音卡未创建")?, app.get_webview_window("region-preview").ok_or("圈选窗口未创建")?] {
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
            let quit = MenuItem::with_id(app, "quit", "退出任务面板", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &region, &quit])?;
            let mut rgba = vec![0u8; 16 * 16 * 4];
            for y in 3..13 { for x in 3..13 {
                if y <= 4 || (7..=8).contains(&x) { rgba[(y * 16 + x) * 4 + 3] = 255; }
            }}
            TrayIconBuilder::with_id("task-space-tray")
                .icon(tauri::image::Image::new_owned(rgba, 16, 16)).icon_as_template(true)
                .tooltip("Yonda 任务总览").menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "tasks" => { let _ = show_menu(app, true); },
                    "region" => { let state = app.state::<TaskState>(); let preview = app.state::<PreviewState>(); if let Err(error) = show_region_preview(app, &state, &preview) { show_region_feedback(app, &preview, &error); } },
                    "quit" => app.exit(0),
                    _ => {}
                }).build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "task-space" || window.label() == "voice-input" || window.label() == "region-preview" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if window.label() == "voice-input" { voice_input::cancel(); }
                    if window.label() == "region-preview" { window.app_handle().state::<PreviewState>().0.lock().ok().map(|mut session| session.clear()); if let Some(preview) = window.app_handle().get_webview_window("region-preview") { let _ = preview.eval("window.dispatchEvent(new Event('yonda-region-clear'))"); } }
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("Yonda任务面板启动失败");
}
