//! Read-only Windows hook diagnostic. It is never an admission observer.

use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::{Result, anyhow, ensure};
use windows_sys::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, KBDLLHOOKSTRUCT, LLKHF_INJECTED,
        LLMHF_INJECTED, MSG, MSLLHOOKSTRUCT, PM_NOREMOVE, PeekMessageW, PostThreadMessageW,
        SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WH_KEYBOARD_LL, WH_MOUSE_LL,
        WM_QUIT,
    },
};

use super::windows::current_input_desktop_name;

thread_local! {
    static HOOK_STATE: RefCell<Option<Arc<State>>> = const { RefCell::new(None) };
}

#[derive(Debug)]
struct State {
    generation: AtomicU64,
    user_count: AtomicU64,
    unknown_count: AtomicU64,
    thread_running: AtomicBool,
    lost: AtomicBool,
    stop: AtomicBool,
}

impl State {
    fn new() -> Self {
        Self {
            generation: AtomicU64::new(1),
            user_count: AtomicU64::new(0),
            unknown_count: AtomicU64::new(0),
            thread_running: AtomicBool::new(false),
            lost: AtomicBool::new(false),
            stop: AtomicBool::new(false),
        }
    }

    fn increment_generation(&self) {
        if self
            .generation
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                value.checked_add(1)
            })
            .is_err()
        {
            self.mark_lost();
        }
    }

    fn record(&self, injected: bool) {
        let counter = if injected {
            &self.unknown_count
        } else {
            &self.user_count
        };
        if counter
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                value.checked_add(1)
            })
            .is_err()
        {
            self.mark_lost();
        } else {
            self.increment_generation();
        }
    }

    fn mark_lost(&self) {
        if !self.lost.swap(true, Ordering::SeqCst) {
            self.thread_running.store(false, Ordering::SeqCst);
            let _ = self
                .generation
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                    value.checked_add(1)
                });
        }
    }
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let event = unsafe { &*(lparam as *const MSLLHOOKSTRUCT) };
        HOOK_STATE.with(|slot| {
            if let Some(state) = slot.borrow().as_ref() {
                state.record(event.flags & LLMHF_INJECTED != 0);
            }
        });
    }
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let event = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
        HOOK_STATE.with(|slot| {
            if let Some(state) = slot.borrow().as_ref() {
                state.record(event.flags & LLKHF_INJECTED != 0);
            }
        });
    }
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}

#[derive(Debug, Clone, Copy)]
pub struct InputHookSample {
    pub generation: u64,
    pub user_count: u64,
    pub unknown_count: u64,
    pub diagnostic_thread_running: bool,
    pub thread_or_desktop_lost: bool,
    pub admission_ready: bool,
}

pub struct InputHookDiagnostic {
    state: Arc<State>,
    desktop_name: Vec<u16>,
    thread_id: u32,
    thread: Option<JoinHandle<()>>,
}

impl InputHookDiagnostic {
    pub fn start() -> Result<Self> {
        let state = Arc::new(State::new());
        let (sender, receiver) = mpsc::sync_channel(1);
        let thread_state = Arc::clone(&state);
        let thread = thread::spawn(move || {
            let desktop = current_input_desktop_name();
            let Ok(desktop_name) = desktop else {
                thread_state.mark_lost();
                let _ = sender.send(Err(anyhow!("input desktop unavailable")));
                return;
            };
            HOOK_STATE.with(|slot| *slot.borrow_mut() = Some(Arc::clone(&thread_state)));
            let mouse = unsafe {
                SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), std::ptr::null_mut(), 0)
            };
            if mouse.is_null() {
                thread_state.mark_lost();
                let _ = sender.send(Err(anyhow!("mouse input observer unavailable")));
                return;
            }
            let keyboard = unsafe {
                SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), std::ptr::null_mut(), 0)
            };
            if keyboard.is_null() {
                unsafe { UnhookWindowsHookEx(mouse) };
                thread_state.mark_lost();
                let _ = sender.send(Err(anyhow!("keyboard input observer unavailable")));
                return;
            }
            let mut message = MSG::default();
            unsafe { PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_NOREMOVE) };
            if thread_state.stop.load(Ordering::SeqCst) {
                unsafe {
                    UnhookWindowsHookEx(keyboard);
                    UnhookWindowsHookEx(mouse);
                }
                thread_state.mark_lost();
                return;
            }
            let thread_id = unsafe { GetCurrentThreadId() };
            thread_state.thread_running.store(true, Ordering::SeqCst);
            let _ = sender.send(Ok((thread_id, desktop_name)));
            loop {
                let result = unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) };
                if result <= 0 {
                    break;
                }
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            thread_state.mark_lost();
            unsafe {
                UnhookWindowsHookEx(keyboard);
                UnhookWindowsHookEx(mouse);
            }
            HOOK_STATE.with(|slot| *slot.borrow_mut() = None);
        });
        let started = receiver.recv_timeout(Duration::from_secs(5));
        let (thread_id, desktop_name) = match started {
            Ok(Ok(value)) => value,
            Ok(Err(error)) => return Err(error),
            Err(_) => {
                state.stop.store(true, Ordering::SeqCst);
                return Err(anyhow!("input observer startup timeout"));
            }
        };
        ensure!(thread_id != 0, "input observer thread unavailable");
        Ok(Self {
            state,
            desktop_name,
            thread_id,
            thread: Some(thread),
        })
    }

    pub fn sample(&self) -> InputHookSample {
        if self.thread.as_ref().is_none_or(JoinHandle::is_finished)
            || !current_input_desktop_name().is_ok_and(|name| name == self.desktop_name)
        {
            self.state.mark_lost();
        }
        InputHookSample {
            generation: self.state.generation.load(Ordering::SeqCst),
            user_count: self.state.user_count.load(Ordering::SeqCst),
            unknown_count: self.state.unknown_count.load(Ordering::SeqCst),
            diagnostic_thread_running: self.state.thread_running.load(Ordering::SeqCst),
            thread_or_desktop_lost: self.state.lost.load(Ordering::SeqCst),
            admission_ready: false,
        }
    }
}

impl Drop for InputHookDiagnostic {
    fn drop(&mut self) {
        self.state.stop.store(true, Ordering::SeqCst);
        self.state.thread_running.store(false, Ordering::SeqCst);
        unsafe { PostThreadMessageW(self.thread_id, WM_QUIT, 0, 0) };
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
