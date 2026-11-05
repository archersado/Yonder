//! Read-only host-side Windows identity evidence. This cannot prove window lifetime.

use anyhow::{Result, anyhow, ensure};
use sha2::{Digest, Sha256};
use windows_sys::Win32::{
    Foundation::{CloseHandle, FILETIME},
    System::Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    UI::WindowsAndMessaging::{GetWindowThreadProcessId, IsWindow},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostWindowEvidence {
    pub process_id: u32,
    pub native_window_id: String,
    pub process_start_sha256: String,
}

impl HostWindowEvidence {
    pub fn matches_adapter_window(&self, window: &agent_desktop_core::WindowInfo) -> bool {
        window.pid.get() == self.process_id
            && window.id == self.native_window_id
            && window
                .process_instance
                .as_deref()
                .is_some_and(|token| sha256(token.as_bytes()) == self.process_start_sha256)
    }
}

pub fn capture_window(process_id: u32, native_window_id: &str) -> Result<HostWindowEvidence> {
    ensure!(process_id > 0, "invalid target process");
    let hwnd = native_window_id
        .strip_prefix("w-")
        .and_then(|text| text.parse::<usize>().ok())
        .filter(|value| *value != 0)
        .ok_or_else(|| anyhow!("invalid native window ID"))?;
    let hwnd = hwnd as *mut core::ffi::c_void;
    ensure!(unsafe { IsWindow(hwnd) } != 0, "target window is gone");
    let mut owner = 0_u32;
    let thread = unsafe { GetWindowThreadProcessId(hwnd, &mut owner) };
    ensure!(thread != 0 && owner == process_id, "window owner changed");

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id) };
    ensure!(!handle.is_null(), "target process identity unavailable");
    let mut created = FILETIME::default();
    let mut exited = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    let read =
        unsafe { GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user) };
    unsafe { CloseHandle(handle) };
    ensure!(read != 0, "target process creation time unavailable");
    let ticks = (u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime);
    ensure!(ticks != 0, "target process creation time invalid");
    let token = format!(
        "windows-proc-v1:{}:{}",
        ticks / 10_000_000,
        ticks % 10_000_000
    );
    Ok(HostWindowEvidence {
        process_id,
        native_window_id: native_window_id.to_owned(),
        process_start_sha256: sha256(token.as_bytes()),
    })
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_window_id_is_rejected_before_native_calls() {
        for id in ["", "1", "w-0", "w-nope"] {
            assert!(capture_window(1, id).is_err());
        }
    }
}
