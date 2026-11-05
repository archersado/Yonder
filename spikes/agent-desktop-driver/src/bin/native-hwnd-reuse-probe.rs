//! Isolated read-only HWND reuse diagnostic; never dispatches desktop input.

#[cfg(target_os = "windows")]
fn main() -> anyhow::Result<()> {
    use anyhow::ensure;
    use sha2::{Digest, Sha256};
    use windows_sys::Win32::{
        Foundation::HWND,
        System::Threading::GetCurrentProcessId,
        UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow},
    };
    use yonder_agent_desktop_driver_spike::host_oracle::{windows, windows_lifecycle};

    fn hidden_window() -> anyhow::Result<HWND> {
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let window = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                std::ptr::null(),
                0,
                0,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null(),
            )
        };
        ensure!(!window.is_null(), "fixture window unavailable");
        Ok(window)
    }

    fn destroy(window: HWND) -> anyhow::Result<()> {
        ensure!(
            unsafe { DestroyWindow(window) } != 0,
            "fixture window destruction failed"
        );
        Ok(())
    }

    fn seek_exact_reuse() -> anyhow::Result<()> {
        const MAX_REPLACEMENTS: u32 = 70_000;
        let pid = unsafe { GetCurrentProcessId() };
        let first = hidden_window()?;
        let id = format!("w-{}", first as usize);
        let initial = windows::capture_window(pid, &id)?;
        destroy(first)?;
        let mut slot_reused_with_changed_handle = false;
        let mut reuse_observed = false;
        let mut same_host_fields = false;
        let mut replacements = 0;
        for attempt in 1..=MAX_REPLACEMENTS {
            let current = hidden_window()?;
            replacements = attempt;
            if (current as usize & 0xffff) == (first as usize & 0xffff) {
                if current == first {
                    reuse_observed = true;
                    same_host_fields = windows::capture_window(pid, &id)? == initial;
                    destroy(current)?;
                    break;
                }
                slot_reused_with_changed_handle = true;
            }
            destroy(current)?;
        }
        println!(
            "{}",
            serde_json::json!({
                "schema_version": "yonder-agent-desktop-hwnd-reuse-probe/v1",
                "platform": "windows",
                "mode": "seek_exact",
                "window_id_sha256": format!("{:x}", Sha256::digest(id.as_bytes())),
                "replacements": replacements,
                "reuse_observed": reuse_observed,
                "slot_reused_with_changed_handle": slot_reused_with_changed_handle,
                "same_host_fields": same_host_fields,
                "lifecycle_observer_active": false,
                "window_generation_proven": false,
            })
        );
        Ok(())
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.as_slice() == ["--seek-exact"] {
        return seek_exact_reuse();
    }
    ensure!(args.is_empty(), "unsupported diagnostic option");

    const MAX_REPLACEMENTS: u32 = 512;
    let observer = windows_lifecycle::LifecycleObserver::start()?;
    let pid = unsafe { GetCurrentProcessId() };
    let first = hidden_window()?;
    let id = format!("w-{}", first as usize);
    let initial = windows::capture_window(pid, &id)?;
    let before = observer.barrier_for(&id)?;
    destroy(first)?;

    let mut reused = false;
    let mut slot_reused_with_changed_handle = false;
    let mut same_host_fields = false;
    let mut replacements = 0;
    for attempt in 1..=MAX_REPLACEMENTS {
        let current = hidden_window()?;
        replacements = attempt;
        if current != first && (current as usize & 0xffff) == (first as usize & 0xffff) {
            slot_reused_with_changed_handle = true;
        }
        if current == first {
            reused = true;
            same_host_fields = windows::capture_window(pid, &id)? == initial;
            let after = observer.barrier_for(&id)?;
            println!(
                "{}",
                serde_json::json!({
                    "schema_version": "yonder-agent-desktop-hwnd-reuse-probe/v1",
                    "platform": "windows",
                    "window_id_sha256": format!("{:x}", Sha256::digest(id.as_bytes())),
                    "replacements": replacements,
                    "reuse_observed": reused,
                    "slot_reused_with_changed_handle": slot_reused_with_changed_handle,
                    "same_host_fields": same_host_fields,
                    "marker_observed": before.marker_observed && after.marker_observed,
                    "creates_before": before.creates,
                    "destroys_before": before.destroys,
                    "creates_after": after.creates,
                    "destroys_after": after.destroys,
                    "lifecycle_changed": after.creates > before.creates && after.destroys > before.destroys,
                    "window_generation_proven": false,
                })
            );
            destroy(current)?;
            break;
        }
        destroy(current)?;
    }
    if !reused {
        let after = observer.barrier_for(&id)?;
        println!(
            "{}",
            serde_json::json!({
                "schema_version": "yonder-agent-desktop-hwnd-reuse-probe/v1",
                "platform": "windows",
                "window_id_sha256": format!("{:x}", Sha256::digest(id.as_bytes())),
                "replacements": replacements,
                "reuse_observed": false,
                "slot_reused_with_changed_handle": slot_reused_with_changed_handle,
                "same_host_fields": same_host_fields,
                "marker_observed": before.marker_observed && after.marker_observed,
                "creates_before": before.creates,
                "destroys_before": before.destroys,
                "creates_after": after.creates,
                "destroys_after": after.destroys,
                "window_generation_proven": false,
            })
        );
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("native HWND reuse probe requires Windows")
}
