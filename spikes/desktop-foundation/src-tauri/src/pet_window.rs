//! E0 窗口展示状态；不保存或创建任务。
use std::sync::Mutex;
use tauri::{LogicalSize, PhysicalPosition, PhysicalSize, State, WebviewWindow};

#[derive(Clone, Copy)]
struct Restore {
    position: PhysicalPosition<i32>,
    logical_size: LogicalSize<f64>,
    edge: &'static str,
}

pub struct PetWindowState {
    restore: Mutex<Option<Restore>>,
    // E0 没有执行器，任务数已知为 0。正式宿主必须由任务事实源更新；None 禁止隐藏。
    pub running_tasks: Mutex<Option<usize>>,
}

impl Default for PetWindowState {
    fn default() -> Self {
        Self { restore: Mutex::new(None), running_tasks: Mutex::new(Some(0)) }
    }
}

fn can_dock(tasks: Option<usize>) -> bool { tasks == Some(0) }

fn fit(position: PhysicalPosition<i32>, size: PhysicalSize<u32>, origin: PhysicalPosition<i32>, screen: PhysicalSize<u32>) -> PhysicalPosition<i32> {
    PhysicalPosition::new(
        position.x.clamp(origin.x, origin.x.saturating_add(screen.width.saturating_sub(size.width) as i32)),
        position.y.clamp(origin.y, origin.y.saturating_add(screen.height.saturating_sub(size.height) as i32)),
    )
}

fn dock_layout(position: PhysicalPosition<i32>, size: PhysicalSize<u32>, origin: PhysicalPosition<i32>, screen: PhysicalSize<u32>, scale: f64) -> (&'static str, PhysicalPosition<i32>, LogicalSize<f64>) {
    let left = i64::from(position.x) - i64::from(origin.x);
    let top = i64::from(position.y) - i64::from(origin.y);
    let right = i64::from(screen.width) - left - i64::from(size.width);
    let bottom = i64::from(screen.height) - top - i64::from(size.height);
    let edge = [("bottom", bottom), ("left", left), ("right", right), ("top", top)]
        .into_iter().min_by_key(|(_, distance)| (*distance).max(0)).unwrap().0;
    let logical = if edge == "left" || edge == "right" { LogicalSize::new(56.0, 112.0) } else { LogicalSize::new(112.0, 56.0) };
    let dock: PhysicalSize<u32> = logical.to_physical(scale);
    let mut target = PhysicalPosition::new(
        position.x + (i64::from(size.width) - i64::from(dock.width)) as i32 / 2,
        position.y + (i64::from(size.height) - i64::from(dock.height)) as i32 / 2,
    );
    match edge {
        "left" => target.x = origin.x,
        "right" => target.x = origin.x + screen.width.saturating_sub(dock.width) as i32,
        "top" => target.y = origin.y,
        _ => target.y = origin.y + screen.height.saturating_sub(dock.height) as i32,
    }
    (edge, fit(target, dock, origin, screen), logical)
}

#[tauri::command]
pub fn pet_dock(window: WebviewWindow, state: State<'_, PetWindowState>) -> Result<&'static str, String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    let tasks = state.running_tasks.lock().map_err(|_| "任务状态不可用")?;
    if !can_dock(*tasks) { return Err("后台任务运行中或状态未知，保持展开".into()); }
    let mut saved = state.restore.lock().map_err(|_| "窗口状态不可用")?;
    if let Some(original) = *saved { return Ok(original.edge); }
    let monitor = window.current_monitor().map_err(|_| "无法读取屏幕")?
        .or(window.primary_monitor().map_err(|_| "无法读取主屏幕")?)
        .ok_or("没有可用屏幕")?;
    let position = window.outer_position().map_err(|_| "无法读取窗口位置")?;
    let scale = window.scale_factor().map_err(|_| "无法读取缩放")?;
    let current_size = window.inner_size().map_err(|_| "无法读取尺寸")?;
    let area = monitor.work_area();
    // 左右贴显示器边；上下避开菜单栏与 Dock，保证探头眼睛可点击。
    let origin = PhysicalPosition::new(monitor.position().x, area.position.y);
    let bounds = PhysicalSize::new(monitor.size().width, area.size.height);
    let (edge, target, dock_size) = dock_layout(position, current_size, origin, bounds, scale);
    let original = Restore { position, logical_size: current_size.to_logical(scale), edge };
    let physical: PhysicalSize<u32> = dock_size.to_physical(scale);
    window.set_size(dock_size).map_err(|_| "无法缩小窗口")?;
    if window.set_position(target).is_err() {
        let _ = window.set_size(original.logical_size);
        return Err("停靠失败，保持展开".into());
    }
    *saved = Some(original);
    eprintln!("桌宠停靠：edge={edge}, position={target:?}, size={physical:?}");
    Ok(edge)
}

#[tauri::command]
pub fn pet_wake(window: WebviewWindow, state: State<'_, PetWindowState>) -> Result<(), String> {
    if window.label() != "pet" { return Err("不允许的窗口".into()); }
    let mut saved = state.restore.lock().map_err(|_| "窗口状态不可用")?;
    let Some(original) = *saved else { return Ok(()); };
    let monitors = window.available_monitors().map_err(|_| "无法读取屏幕")?;
    let monitor = monitors.into_iter().find(|m| {
        let a = m.work_area();
        original.position.x >= a.position.x && original.position.y >= a.position.y
            && original.position.x < a.position.x + a.size.width as i32
            && original.position.y < a.position.y + a.size.height as i32
    }).or(window.primary_monitor().map_err(|_| "无法读取主屏幕")?).ok_or("没有可用屏幕")?;
    let area = monitor.work_area();
    let size = original.logical_size.to_physical(monitor.scale_factor());
    let target = fit(original.position, size, area.position, area.size);
    // 位置设置失败时仍保留小眼睛入口，允许重试。
    window.set_position(target).map_err(|_| "无法恢复位置")?;
    if window.set_size(original.logical_size).is_err() { return Err("无法恢复尺寸，请重试".into()); }
    *saved = None;
    eprintln!("桌宠唤醒：position={target:?}, size={size:?}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn task_gate_and_monitor_bounds() {
        assert!(can_dock(Some(0)));
        assert!(!can_dock(Some(1)));
        assert!(!can_dock(None));
        assert_eq!(fit(PhysicalPosition::new(-9999, 9999), PhysicalSize::new(88, 96),
            PhysicalPosition::new(-1920, 24), PhysicalSize::new(1920, 1056)), PhysicalPosition::new(-1920, 984));
        assert_eq!(fit(PhysicalPosition::new(30, 40), PhysicalSize::new(400, 400),
            PhysicalPosition::new(0, 0), PhysicalSize::new(100, 100)), PhysicalPosition::new(0, 0));
    }

    #[test]
    fn nearest_edge_and_scaled_coordinates() {
        let origin = PhysicalPosition::new(-1920, 24);
        let bounds = PhysicalSize::new(1920, 1056);
        for (x, y, expected) in [(-1910, 400, "left"), (-210, 400, "right"), (-1000, 30, "top"), (-1000, 870, "bottom")] {
            let (edge, target, size) = dock_layout(PhysicalPosition::new(x, y), PhysicalSize::new(200, 200), origin, bounds, 1.0);
            assert_eq!(edge, expected);
            let physical = size.to_physical(1.0);
            assert_eq!(target, fit(target, physical, origin, bounds));
        }
        let (edge, target, size) = dock_layout(PhysicalPosition::new(0, 500), PhysicalSize::new(400, 400), PhysicalPosition::new(0, 48), PhysicalSize::new(3420, 2000), 2.0);
        assert_eq!(edge, "left");
        assert_eq!(size, LogicalSize::new(56.0, 112.0));
        assert_eq!(target, PhysicalPosition::new(0, 588));
        assert_eq!(dock_layout(PhysicalPosition::new(400, 400), PhysicalSize::new(200, 200), PhysicalPosition::new(0, 0), PhysicalSize::new(1000, 1000), 1.0).0, "bottom");
    }
}
