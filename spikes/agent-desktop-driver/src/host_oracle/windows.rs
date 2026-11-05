//! Read-only host-side Windows identity evidence. This cannot prove window lifetime.

use anyhow::{Result, anyhow, ensure};
use sha2::{Digest, Sha256};
use windows_sys::Win32::{
    Foundation::{CloseHandle, FILETIME, HANDLE},
    Security::{
        GetSidIdentifierAuthority, GetSidSubAuthority, GetSidSubAuthorityCount,
        GetTokenInformation, IsValidSid, SECURITY_MANDATORY_LABEL_AUTHORITY, TOKEN_MANDATORY_LABEL,
        TOKEN_QUERY, TokenIntegrityLevel,
    },
    System::{
        RemoteDesktop::ProcessIdToSessionId,
        StationsAndDesktops::{GetThreadDesktop, GetUserObjectInformationW, UOI_IO, UOI_NAME},
        Threading::{
            GetCurrentProcess, GetCurrentProcessId, GetCurrentThreadId, GetProcessTimes,
            OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
        },
    },
    UI::WindowsAndMessaging::{GetPropW, GetWindowThreadProcessId, IsWindow},
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
    let hwnd = parse_hwnd(native_window_id)?;
    ensure!(unsafe { IsWindow(hwnd) } != 0, "target window is gone");
    let mut owner = 0_u32;
    let thread = unsafe { GetWindowThreadProcessId(hwnd, &mut owner) };
    ensure!(thread != 0 && owner == process_id, "window owner changed");
    verify_desktop_scope(process_id, thread)?;

    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id) };
    ensure!(!handle.is_null(), "target process identity unavailable");
    let result = (|| -> Result<HostWindowEvidence> {
        let target_integrity = integrity_rid(handle)?;
        let observer_integrity = integrity_rid(unsafe { GetCurrentProcess() })?;
        ensure!(
            target_integrity <= observer_integrity,
            "target integrity exceeds observer"
        );
        let mut created = FILETIME::default();
        let mut exited = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        let read =
            unsafe { GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user) };
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
    })();
    unsafe { CloseHandle(handle) };
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity01WindowEvidence {
    pub window: HostWindowEvidence,
    pub role: u8,
    pub generation: usize,
    pub click_count: usize,
    pub window_token_sha256: String,
}

/// Reads only properties created by Yonder's controlled IDENTITY-01 fixture.
/// It does not establish an identity oracle for arbitrary applications.
pub fn capture_identity01_window(
    process_id: u32,
    native_window_id: &str,
) -> Result<Identity01WindowEvidence> {
    let window = capture_window(process_id, native_window_id)?;
    let hwnd = parse_hwnd(native_window_id)?;
    let role = fixture_property(hwnd, "YonderCUIdentity01Role")?;
    ensure!(role == 1 || role == 2, "fixture window role invalid");
    let generation = fixture_property(hwnd, "YonderCUIdentity01Generation")?;
    let click_count = fixture_property(hwnd, "YonderCUIdentity01ClickCount")?;
    ensure!(
        generation != 0 && click_count != 0,
        "fixture property invalid"
    );
    let window_token_sha256 = sha256(
        format!(
            "identity-01/windows/v1\0{}\0{generation}",
            window.process_start_sha256
        )
        .as_bytes(),
    );
    Ok(Identity01WindowEvidence {
        window,
        role: role as u8,
        generation,
        click_count,
        window_token_sha256,
    })
}

fn parse_hwnd(native_window_id: &str) -> Result<*mut core::ffi::c_void> {
    native_window_id
        .strip_prefix("w-")
        .and_then(|text| text.parse::<usize>().ok())
        .filter(|value| *value != 0)
        .map(|value| value as *mut core::ffi::c_void)
        .ok_or_else(|| anyhow!("invalid native window ID"))
}

fn fixture_property(hwnd: *mut core::ffi::c_void, name: &str) -> Result<usize> {
    let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let value = unsafe { GetPropW(hwnd, wide.as_ptr()) } as usize;
    ensure!(value != 0, "fixture property unavailable");
    Ok(value)
}

fn verify_desktop_scope(process_id: u32, target_thread: u32) -> Result<()> {
    let mut target_session = 0_u32;
    let mut observer_session = 0_u32;
    ensure!(
        unsafe { ProcessIdToSessionId(process_id, &mut target_session) } != 0
            && unsafe { ProcessIdToSessionId(GetCurrentProcessId(), &mut observer_session) } != 0,
        "desktop session unavailable"
    );
    ensure!(target_session == observer_session, "target session differs");
    let target_desktop = unsafe { GetThreadDesktop(target_thread) };
    let observer_desktop = unsafe { GetThreadDesktop(GetCurrentThreadId()) };
    ensure!(
        !target_desktop.is_null() && !observer_desktop.is_null(),
        "desktop identity unavailable"
    );
    ensure!(
        desktop_is_input(target_desktop)? && desktop_is_input(observer_desktop)?,
        "target is outside input desktop"
    );
    ensure!(
        desktop_name(target_desktop)? == desktop_name(observer_desktop)?,
        "target desktop differs"
    );
    Ok(())
}

/// Identifies the interactive desktop of the calling thread for observer binding.
pub fn current_input_desktop_name() -> Result<Vec<u16>> {
    let desktop = unsafe { GetThreadDesktop(GetCurrentThreadId()) };
    ensure!(!desktop.is_null(), "observer desktop unavailable");
    ensure!(
        desktop_is_input(desktop)?,
        "observer is outside input desktop"
    );
    desktop_name(desktop)
}

fn desktop_is_input(desktop: HANDLE) -> Result<bool> {
    let mut value = 0_i32;
    let mut needed = 0_u32;
    ensure!(
        unsafe {
            GetUserObjectInformationW(
                desktop,
                UOI_IO,
                (&mut value as *mut i32).cast(),
                size_of::<i32>() as u32,
                &mut needed,
            )
        } != 0,
        "input desktop status unavailable"
    );
    Ok(value != 0)
}

fn desktop_name(desktop: HANDLE) -> Result<Vec<u16>> {
    let mut needed = 0_u32;
    unsafe { GetUserObjectInformationW(desktop, UOI_NAME, std::ptr::null_mut(), 0, &mut needed) };
    ensure!(
        (2..=512).contains(&needed) && needed.is_multiple_of(2),
        "desktop name length invalid"
    );
    let mut name = vec![0_u16; needed as usize / 2];
    ensure!(
        unsafe {
            GetUserObjectInformationW(
                desktop,
                UOI_NAME,
                name.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        } != 0,
        "desktop name unavailable"
    );
    let end = name
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(name.len());
    ensure!(end > 0, "desktop name empty");
    name.truncate(end);
    Ok(name)
}

fn integrity_rid(process: HANDLE) -> Result<u32> {
    let mut token = std::ptr::null_mut();
    ensure!(
        unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } != 0,
        "process token unavailable"
    );
    let result = (|| -> Result<u32> {
        let mut needed = 0_u32;
        unsafe {
            GetTokenInformation(
                token,
                TokenIntegrityLevel,
                std::ptr::null_mut(),
                0,
                &mut needed,
            )
        };
        ensure!(
            needed >= size_of::<TOKEN_MANDATORY_LABEL>() as u32 && needed <= 4096,
            "integrity label length invalid"
        );
        let mut storage = vec![0_u64; (needed as usize).div_ceil(size_of::<u64>())];
        ensure!(
            unsafe {
                GetTokenInformation(
                    token,
                    TokenIntegrityLevel,
                    storage.as_mut_ptr().cast(),
                    needed,
                    &mut needed,
                )
            } != 0,
            "integrity label unavailable"
        );
        let label = unsafe { storage.as_ptr().cast::<TOKEN_MANDATORY_LABEL>().read() };
        let sid = label.Label.Sid;
        ensure!(
            !sid.is_null() && unsafe { IsValidSid(sid) } != 0,
            "integrity SID invalid"
        );
        let authority = unsafe { GetSidIdentifierAuthority(sid) };
        ensure!(
            !authority.is_null()
                && unsafe { (*authority).Value == SECURITY_MANDATORY_LABEL_AUTHORITY.Value },
            "integrity SID authority invalid"
        );
        let count = unsafe { GetSidSubAuthorityCount(sid) };
        ensure!(
            !count.is_null() && unsafe { *count } > 0,
            "integrity SID empty"
        );
        let rid = unsafe { GetSidSubAuthority(sid, u32::from(*count) - 1) };
        ensure!(!rid.is_null(), "integrity RID unavailable");
        Ok(unsafe { *rid })
    })();
    unsafe { CloseHandle(token) };
    result
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn malformed_window_id_is_rejected_before_native_calls() {
        for id in ["", "1", "w-0", "w-nope"] {
            assert!(capture_window(1, id).is_err());
        }
    }

    #[test]
    fn own_hidden_window_is_in_supported_scope() -> Result<()> {
        use windows_sys::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow};

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
        ensure!(!window.is_null(), "test window unavailable");
        let id = format!("w-{}", window as usize);
        let result = capture_window(unsafe { GetCurrentProcessId() }, &id);
        unsafe { DestroyWindow(window) };
        ensure!(result.is_ok(), "own input desktop window was rejected");
        Ok(())
    }

    #[test]
    fn different_desktop_is_rejected() -> Result<()> {
        use std::{sync::mpsc, time::Duration};
        use windows_sys::Win32::System::StationsAndDesktops::{
            CloseDesktop, CreateDesktopW, DESKTOP_CREATEWINDOW, DESKTOP_READOBJECTS,
            DESKTOP_WRITEOBJECTS, SetThreadDesktop,
        };

        let name: Vec<u16> = format!("YonderSpikeDesktop{}\0", std::process::id())
            .encode_utf16()
            .collect();
        let desktop = unsafe {
            CreateDesktopW(
                name.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                DESKTOP_CREATEWINDOW | DESKTOP_READOBJECTS | DESKTOP_WRITEOBJECTS,
                std::ptr::null(),
            )
        };
        ensure!(!desktop.is_null(), "test desktop unavailable");
        let (ready_tx, ready_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel();
        let desktop_address = desktop as usize;
        let thread = std::thread::spawn(move || {
            let attached = unsafe { SetThreadDesktop(desktop_address as *mut core::ffi::c_void) };
            let _ = ready_tx.send((attached != 0).then(|| {
                (
                    unsafe { GetCurrentThreadId() },
                    current_input_desktop_name()
                        .err()
                        .map(|error| error.to_string()),
                )
            }));
            let _ = stop_rx.recv();
        });
        let target_thread = ready_rx
            .recv_timeout(Duration::from_secs(2))
            .context("test desktop thread did not start")?;
        let verdict = target_thread.as_ref().map(|(thread_id, _)| {
            verify_desktop_scope(unsafe { GetCurrentProcessId() }, *thread_id)
        });
        let _ = stop_tx.send(());
        thread.join().map_err(|_| anyhow!("test thread panicked"))?;
        ensure!(
            unsafe { CloseDesktop(desktop) } != 0,
            "test desktop did not close"
        );
        ensure!(
            target_thread.is_some(),
            "test thread could not attach desktop"
        );
        ensure!(
            target_thread
                .as_ref()
                .and_then(|(_, rejection)| rejection.as_deref())
                == Some("observer is outside input desktop"),
            "observer did not reject non-input desktop"
        );
        let rejection = verdict
            .and_then(Result::err)
            .context("non-input desktop was accepted")?;
        ensure!(
            rejection.to_string() == "target is outside input desktop",
            "non-input desktop failed for another reason"
        );
        Ok(())
    }
}
