//! Experimental read-only WinEvent lifecycle observer. It does not authorize dispatch.

use std::{
    cell::{Cell, RefCell},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow, ensure};
use windows_sys::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        WindowsAndMessaging::{
            CHILDID_SELF, CreateWindowExW, DestroyWindow, DispatchMessageW, EVENT_OBJECT_CREATE,
            EVENT_OBJECT_DESTROY, MSG, OBJID_WINDOW, PM_REMOVE, PeekMessageW, TranslateMessage,
            WINEVENT_OUTOFCONTEXT,
        },
    },
};

const EVENT_LIMIT: usize = 4096;
const BARRIER_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Clone, Copy)]
struct WindowEvent {
    kind: u32,
    hwnd: usize,
}

thread_local! {
    static EVENTS: RefCell<Vec<WindowEvent>> = const { RefCell::new(Vec::new()) };
    static LOST: Cell<bool> = const { Cell::new(false) };
}

unsafe extern "system" fn on_event(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    object: i32,
    child: i32,
    _thread: u32,
    _time: u32,
) {
    if object != OBJID_WINDOW || child != CHILDID_SELF as i32 {
        return;
    }
    EVENTS.with(|events| {
        if let Ok(mut events) = events.try_borrow_mut() {
            if events.len() < EVENT_LIMIT {
                events.push(WindowEvent {
                    kind: event,
                    hwnd: hwnd as usize,
                });
            } else {
                LOST.with(|lost| lost.set(true));
            }
        } else {
            LOST.with(|lost| lost.set(true));
        }
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LifecycleSample {
    pub creates: usize,
    pub destroys: usize,
    pub marker_observed: bool,
}

enum Command {
    Barrier {
        target_hwnd: usize,
        reply: Sender<Result<LifecycleSample>>,
    },
    Stop,
}

/// Owns one dedicated WinEvent thread. Dropping it unhooks before joining.
pub struct LifecycleObserver {
    commands: Sender<Command>,
    worker: Option<JoinHandle<()>>,
}

impl LifecycleObserver {
    pub fn start() -> Result<Self> {
        let (commands, incoming) = mpsc::channel();
        let (ready, launched) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || run(incoming, ready));
        match launched.recv_timeout(BARRIER_TIMEOUT) {
            Ok(Ok(())) => Ok(Self {
                commands,
                worker: Some(worker),
            }),
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(_) => Err(anyhow!("WinEvent observer startup timed out")),
        }
    }

    /// Drains earlier WinEvents behind a system-generated hidden-window marker.
    /// Counts are diagnostic only; no window-generation proof is claimed.
    pub fn barrier_for(&self, native_window_id: &str) -> Result<LifecycleSample> {
        let target_hwnd = native_window_id
            .strip_prefix("w-")
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value != 0)
            .ok_or_else(|| anyhow!("invalid native window ID"))?;
        let (reply, response) = mpsc::channel();
        self.commands
            .send(Command::Barrier { target_hwnd, reply })
            .map_err(|_| anyhow!("WinEvent observer stopped"))?;
        response
            .recv_timeout(BARRIER_TIMEOUT + Duration::from_millis(500))
            .map_err(|_| anyhow!("WinEvent barrier timed out"))?
    }
}

impl Drop for LifecycleObserver {
    fn drop(&mut self) {
        let _ = self.commands.send(Command::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run(incoming: Receiver<Command>, ready: mpsc::SyncSender<Result<()>>) {
    let hook = unsafe {
        SetWinEventHook(
            EVENT_OBJECT_CREATE,
            EVENT_OBJECT_DESTROY,
            std::ptr::null_mut(),
            Some(on_event),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        )
    };
    if hook.is_null() {
        let _ = ready.send(Err(anyhow!("WinEvent hook unavailable")));
        return;
    }
    let _ = ready.send(Ok(()));
    loop {
        pump_messages();
        match incoming.recv_timeout(Duration::from_millis(5)) {
            Ok(Command::Barrier { target_hwnd, reply }) => {
                let _ = reply.send(barrier(target_hwnd));
            }
            Ok(Command::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
    unsafe { UnhookWinEvent(hook) };
}

fn pump_messages() {
    let mut message = MSG::default();
    while unsafe { PeekMessageW(&mut message, std::ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

fn barrier(target_hwnd: usize) -> Result<LifecycleSample> {
    ensure!(
        !LOST.with(Cell::get),
        "WinEvent queue overflowed or reentered"
    );
    let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
    let marker = unsafe {
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
    ensure!(!marker.is_null(), "WinEvent marker window unavailable");
    let deadline = Instant::now() + BARRIER_TIMEOUT;
    let observed = loop {
        pump_messages();
        if LOST.with(Cell::get) {
            break Err(anyhow!("WinEvent queue overflowed or reentered"));
        }
        if EVENTS.with(|events| {
            events
                .borrow()
                .iter()
                .any(|event| event.kind == EVENT_OBJECT_CREATE && event.hwnd == marker as usize)
        }) {
            break Ok(());
        }
        if Instant::now() >= deadline {
            break Err(anyhow!("WinEvent marker was not observed"));
        }
        thread::sleep(Duration::from_millis(1));
    };
    unsafe { DestroyWindow(marker) };
    observed?;
    let (creates, destroys) = EVENTS.with(|events| {
        let events = events.borrow();
        (
            events
                .iter()
                .filter(|event| event.hwnd == target_hwnd && event.kind == EVENT_OBJECT_CREATE)
                .count(),
            events
                .iter()
                .filter(|event| event.hwnd == target_hwnd && event.kind == EVENT_OBJECT_DESTROY)
                .count(),
        )
    });
    Ok(LifecycleSample {
        creates,
        destroys,
        marker_observed: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_window_events_pass_the_barrier() -> Result<()> {
        let observer = LifecycleObserver::start()?;
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        let target = unsafe {
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
        ensure!(!target.is_null(), "test window unavailable");
        let id = format!("w-{}", target as usize);
        let before = observer.barrier_for(&id)?;
        unsafe { DestroyWindow(target) };
        let after = observer.barrier_for(&id)?;
        ensure!(before.marker_observed && after.marker_observed);
        ensure!(before.creates >= 1, "create event not observed");
        ensure!(
            after.destroys > before.destroys,
            "destroy event not observed"
        );
        Ok(())
    }
}
