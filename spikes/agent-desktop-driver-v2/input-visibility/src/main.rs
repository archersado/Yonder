//! 受控 Windows Raw Input 可见性诊断；不参与 CUA 派发准入。

#[cfg(not(windows))]
fn main() {
    eprintln!("此探针仅在 Windows 原生运行");
    std::process::exit(2);
}

#[cfg(windows)]
mod windows_probe {
    use std::{
        mem::size_of,
        process::{Command, Stdio},
        ptr::null_mut,
        sync::atomic::{AtomicU32, Ordering},
        thread,
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        System::Threading::GetCurrentProcessId,
        UI::{
            Input::KeyboardAndMouse::{
                INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput, VK_F24,
            },
            Input::{
                GetRawInputData, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RAWKEYBOARD, RID_INPUT,
                RIDEV_INPUTSINK, RIDEV_REMOVE, RIM_TYPEKEYBOARD, RegisterRawInputDevices,
            },
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetClassNameW,
                GetForegroundWindow, GetWindowThreadProcessId, IsWindow, MSG, PM_REMOVE,
                PeekMessageW, RegisterClassW, SW_SHOW, SetForegroundWindow, ShowWindow,
                TranslateMessage, UnregisterClassW, WM_INPUT, WM_KEYUP, WNDCLASSW,
                WS_OVERLAPPEDWINDOW, WS_VISIBLE,
            },
        },
    };

    const TEST_TAG: u32 = 0x594F_4E44;
    static RAW_MESSAGES: AtomicU32 = AtomicU32::new(0);
    static RAW_READ_ERRORS: AtomicU32 = AtomicU32::new(0);
    static RAW_KEYBOARD: AtomicU32 = AtomicU32::new(0);
    static F24_RAW: AtomicU32 = AtomicU32::new(0);
    static F24_RAW_NULL_DEVICE: AtomicU32 = AtomicU32::new(0);
    static F24_RAW_TAG_MATCH: AtomicU32 = AtomicU32::new(0);
    static F24_KEYUP: AtomicU32 = AtomicU32::new(0);

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_INPUT {
            RAW_MESSAGES.fetch_add(1, Ordering::SeqCst);
            let mut raw = RAWINPUT::default();
            let mut size = size_of::<RAWINPUT>() as u32;
            let result = unsafe {
                GetRawInputData(
                    lparam as _,
                    RID_INPUT,
                    (&mut raw as *mut RAWINPUT).cast(),
                    &mut size,
                    size_of::<RAWINPUTHEADER>() as u32,
                )
            };
            if result == u32::MAX || result < size_of::<RAWINPUTHEADER>() as u32 {
                RAW_READ_ERRORS.fetch_add(1, Ordering::SeqCst);
            } else if raw.header.dwType == RIM_TYPEKEYBOARD {
                if result < (std::mem::offset_of!(RAWINPUT, data) + size_of::<RAWKEYBOARD>()) as u32
                {
                    RAW_READ_ERRORS.fetch_add(1, Ordering::SeqCst);
                    return unsafe { DefWindowProcW(hwnd, message, wparam, lparam) };
                }
                RAW_KEYBOARD.fetch_add(1, Ordering::SeqCst);
                let keyboard = unsafe { raw.data.keyboard };
                if keyboard.VKey == VK_F24 && keyboard.Message == WM_KEYUP {
                    F24_RAW.fetch_add(1, Ordering::SeqCst);
                    if raw.header.hDevice.is_null() {
                        F24_RAW_NULL_DEVICE.fetch_add(1, Ordering::SeqCst);
                    }
                    if keyboard.ExtraInformation == TEST_TAG {
                        F24_RAW_TAG_MATCH.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        } else if message == WM_KEYUP && wparam == usize::from(VK_F24) {
            F24_KEYUP.fetch_add(1, Ordering::SeqCst);
        }
        unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
    }

    struct Window {
        hwnd: HWND,
        previous: HWND,
        class: Vec<u16>,
        module: *mut core::ffi::c_void,
        raw_registered: bool,
    }

    impl Drop for Window {
        fn drop(&mut self) {
            unsafe {
                if self.raw_registered {
                    let removed = RAWINPUTDEVICE {
                        usUsagePage: 1,
                        usUsage: 6,
                        dwFlags: RIDEV_REMOVE,
                        hwndTarget: null_mut(),
                    };
                    RegisterRawInputDevices(&removed, 1, size_of::<RAWINPUTDEVICE>() as u32);
                }
                DestroyWindow(self.hwnd);
                UnregisterClassW(self.class.as_ptr(), self.module);
                if !self.previous.is_null() && IsWindow(self.previous) != 0 {
                    SetForegroundWindow(self.previous);
                }
            }
        }
    }

    fn pump_for(duration: Duration) {
        let until = Instant::now() + duration;
        let mut message = MSG::default();
        while Instant::now() < until {
            while unsafe { PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) } != 0 {
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    fn f24_keyup() -> INPUT {
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VK_F24,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: TEST_TAG as usize,
                },
            },
        }
    }

    pub fn run_child(hwnd_raw: &str, pid_raw: &str) -> Result<(), &'static str> {
        let hwnd = hwnd_raw
            .parse::<usize>()
            .map_err(|_| "invalid_fixture_handle")? as HWND;
        let expected_pid = pid_raw.parse::<u32>().map_err(|_| "invalid_fixture_pid")?;
        let mut actual_pid = 0_u32;
        unsafe { GetWindowThreadProcessId(hwnd, &mut actual_pid) };
        let mut class = [0_u16; 64];
        let length = unsafe { GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32) };
        if actual_pid != expected_pid
            || length <= 0
            || String::from_utf16_lossy(&class[..length as usize]) != "YonderV2InputVisibility"
        {
            return Err("fixture_identity_mismatch");
        }
        let input = f24_keyup();
        let mut sent = 0_u32;
        for _ in 0..10 {
            if unsafe { GetForegroundWindow() } != hwnd {
                break;
            }
            if unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) } != 1 {
                break;
            }
            sent += 1;
            thread::sleep(Duration::from_millis(20));
        }
        println!("{sent}");
        Ok(())
    }

    pub fn run() -> Result<(), &'static str> {
        let class: Vec<u16> = "YonderV2InputVisibility\0".encode_utf16().collect();
        let title: Vec<u16> = "Yonder v2 input visibility fixture\0"
            .encode_utf16()
            .collect();
        let module = unsafe { GetModuleHandleW(std::ptr::null()) };
        if module.is_null() {
            return Err("module_unavailable");
        }
        let description = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: module,
            lpszClassName: class.as_ptr(),
            ..WNDCLASSW::default()
        };
        if unsafe { RegisterClassW(&description) } == 0 {
            return Err("window_class_unavailable");
        }
        let previous = unsafe { GetForegroundWindow() };
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                100,
                100,
                460,
                160,
                null_mut(),
                null_mut(),
                module,
                std::ptr::null(),
            )
        };
        if hwnd.is_null() {
            unsafe { UnregisterClassW(class.as_ptr(), module) };
            return Err("fixture_window_unavailable");
        }
        let mut window = Window {
            hwnd,
            previous,
            class,
            module,
            raw_registered: false,
        };
        let registration = RAWINPUTDEVICE {
            usUsagePage: 1,
            usUsage: 6,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        };
        if unsafe { RegisterRawInputDevices(&registration, 1, size_of::<RAWINPUTDEVICE>() as u32) }
            == 0
        {
            return Err("raw_input_registration_failed");
        }
        window.raw_registered = true;
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
        }
        let focus_request_accepted = unsafe { SetForegroundWindow(hwnd) } != 0;
        pump_for(Duration::from_millis(250));
        if unsafe { GetForegroundWindow() } != hwnd {
            println!(
                "{{\"schema_version\":\"yonder-v2-input-visibility/v1\",\"result\":\"foreground_unavailable\",\"foreground_present\":{},\"focus_request_accepted\":{},\"sent\":false,\"admission_ready\":false}}",
                !unsafe { GetForegroundWindow() }.is_null(),
                focus_request_accepted
            );
            return Ok(());
        }
        let raw_before = RAW_KEYBOARD.load(Ordering::SeqCst);
        let raw_messages_before = RAW_MESSAGES.load(Ordering::SeqCst);
        let raw_errors_before = RAW_READ_ERRORS.load(Ordering::SeqCst);
        let f24_raw_before = F24_RAW.load(Ordering::SeqCst);
        let null_before = F24_RAW_NULL_DEVICE.load(Ordering::SeqCst);
        let tagged_before = F24_RAW_TAG_MATCH.load(Ordering::SeqCst);
        let keyup_before = F24_KEYUP.load(Ordering::SeqCst);
        let input = f24_keyup();
        let mut sent = 0_u32;
        let mut focus_lost = false;
        for _ in 0..10 {
            if unsafe { GetForegroundWindow() } != hwnd {
                focus_lost = true;
                break;
            }
            let current = unsafe { SendInput(1, &input, size_of::<INPUT>() as i32) };
            if current != 1 {
                break;
            }
            sent += 1;
            pump_for(Duration::from_millis(50));
        }
        pump_for(Duration::from_millis(250));
        let other_sent = if sent == 10 && unsafe { GetForegroundWindow() } == hwnd {
            let executable =
                std::env::current_exe().map_err(|_| "current_executable_unavailable")?;
            let child = Command::new(executable)
                .arg("--inject")
                .arg((hwnd as usize).to_string())
                .arg(unsafe { GetCurrentProcessId() }.to_string())
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|_| "child_start_failed")?;
            let output = child.wait_with_output().map_err(|_| "child_wait_failed")?;
            if !output.status.success() {
                return Err("child_injection_failed");
            }
            let value = String::from_utf8(output.stdout).map_err(|_| "child_output_invalid")?;
            value
                .trim()
                .parse::<u32>()
                .map_err(|_| "child_count_invalid")?
        } else {
            0
        };
        pump_for(Duration::from_millis(250));
        let raw_delta = RAW_KEYBOARD
            .load(Ordering::SeqCst)
            .saturating_sub(raw_before);
        let raw_messages_delta = RAW_MESSAGES
            .load(Ordering::SeqCst)
            .saturating_sub(raw_messages_before);
        let raw_errors_delta = RAW_READ_ERRORS
            .load(Ordering::SeqCst)
            .saturating_sub(raw_errors_before);
        let f24_raw_delta = F24_RAW
            .load(Ordering::SeqCst)
            .saturating_sub(f24_raw_before);
        let null_delta = F24_RAW_NULL_DEVICE
            .load(Ordering::SeqCst)
            .saturating_sub(null_before);
        let tagged_delta = F24_RAW_TAG_MATCH
            .load(Ordering::SeqCst)
            .saturating_sub(tagged_before);
        let keyup_delta = F24_KEYUP
            .load(Ordering::SeqCst)
            .saturating_sub(keyup_before);
        println!(
            "{{\"schema_version\":\"yonder-v2-input-visibility/v1\",\"result\":\"observed\",\"current_sent_count\":{},\"other_sent_count\":{},\"focus_lost\":{},\"f24_keyup_messages\":{},\"raw_input_messages\":{},\"raw_read_errors\":{},\"raw_keyboard_messages\":{},\"f24_raw_messages\":{},\"f24_raw_null_device\":{},\"f24_raw_tag_match\":{},\"admission_ready\":false}}",
            sent,
            other_sent,
            focus_lost,
            keyup_delta,
            raw_messages_delta,
            raw_errors_delta,
            raw_delta,
            f24_raw_delta,
            null_delta,
            tagged_delta
        );
        Ok(())
    }
}

#[cfg(windows)]
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = if args.get(1).is_some_and(|value| value == "--inject") {
        match (args.get(2), args.get(3)) {
            (Some(hwnd), Some(pid)) => windows_probe::run_child(hwnd, pid),
            _ => Err("missing_fixture_identity"),
        }
    } else {
        windows_probe::run()
    };
    if let Err(code) = result {
        eprintln!(
            "{{\"schema_version\":\"yonder-v2-input-visibility/v1\",\"result\":\"{}\",\"admission_ready\":false}}",
            code
        );
        std::process::exit(1);
    }
}
