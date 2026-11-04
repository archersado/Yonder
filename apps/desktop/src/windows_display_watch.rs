use std::{
    ptr::{null, null_mut},
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
        HWND_MESSAGE, MSG, PostMessageW, PostQuitMessage, RegisterClassW, TranslateMessage,
        UnregisterClassW, WM_CLOSE, WM_DESTROY, WM_DISPLAYCHANGE, WNDCLASSW,
    },
};

struct Watcher {
    hwnd: usize,
    thread: JoinHandle<()>,
}

static WATCHER: Mutex<Option<Watcher>> = Mutex::new(None);
static CALLBACK: Mutex<Option<extern "C" fn()>> = Mutex::new(None);
static ACTIVE: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_DISPLAYCHANGE => {
            if ACTIVE.swap(false, Ordering::SeqCst) {
                let callback = CALLBACK.lock().ok().and_then(|value| *value);
                if let Some(callback) = callback {
                    callback();
                }
            }
            0
        }
        WM_CLOSE => {
            unsafe { DestroyWindow(hwnd) };
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn message_loop(ready: mpsc::SyncSender<Result<usize, ()>>) {
    let class_name: Vec<u16> = "YonderRegionDisplayWatch\0".encode_utf16().collect();
    let instance = unsafe { GetModuleHandleW(null()) };
    if instance.is_null() {
        let _ = ready.send(Err(()));
        return;
    }
    let window_class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        lpszClassName: class_name.as_ptr(),
        ..Default::default()
    };
    if unsafe { RegisterClassW(&window_class) } == 0 {
        let _ = ready.send(Err(()));
        return;
    }
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            null_mut(),
            instance,
            null(),
        )
    };
    if hwnd.is_null() {
        unsafe { UnregisterClassW(class_name.as_ptr(), instance) };
        let _ = ready.send(Err(()));
        return;
    }
    if ready.send(Ok(hwnd as usize)).is_err() {
        unsafe { DestroyWindow(hwnd) };
    } else {
        let mut message = MSG::default();
        while unsafe { GetMessageW(&mut message, null_mut(), 0, 0) } > 0 {
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }
    unsafe { UnregisterClassW(class_name.as_ptr(), instance) };
}

pub fn start(callback: extern "C" fn()) -> Result<(), ()> {
    stop();
    *CALLBACK.lock().map_err(|_| ())? = Some(callback);
    let (sender, receiver) = mpsc::sync_channel(1);
    let thread = match thread::Builder::new()
        .name("region-display-watch".into())
        .spawn(move || message_loop(sender))
    {
        Ok(thread) => thread,
        Err(_) => {
            *CALLBACK.lock().map_err(|_| ())? = None;
            return Err(());
        }
    };
    let hwnd = match receiver.recv_timeout(Duration::from_secs(2)) {
        Ok(Ok(hwnd)) => hwnd,
        _ => {
            // Dropping the receiver makes a late ready.send fail; the message
            // thread then destroys its own window and unregisters the class.
            // Do not join here because a stalled Win32 initialization has not
            // created an HWND that this thread can close yet.
            drop(thread);
            if let Ok(mut callback) = CALLBACK.lock() {
                *callback = None;
            }
            return Err(());
        }
    };
    let mut watcher_slot = match WATCHER.lock() {
        Ok(watcher_slot) => watcher_slot,
        Err(_) => {
            unsafe { PostMessageW(hwnd as HWND, WM_CLOSE, 0, 0) };
            let _ = thread.join();
            if let Ok(mut callback) = CALLBACK.lock() {
                *callback = None;
            }
            return Err(());
        }
    };
    *watcher_slot = Some(Watcher { hwnd, thread });
    ACTIVE.store(true, Ordering::SeqCst);
    Ok(())
}

pub fn stop() {
    ACTIVE.store(false, Ordering::SeqCst);
    let watcher = WATCHER.lock().ok().and_then(|mut value| value.take());
    if let Some(watcher) = watcher {
        unsafe { PostMessageW(watcher.hwnd as HWND, WM_CLOSE, 0, 0) };
        let _ = watcher.thread.join();
    }
    if let Ok(mut callback) = CALLBACK.lock() {
        *callback = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NOTIFICATIONS: AtomicUsize = AtomicUsize::new(0);

    extern "C" fn notified() {
        NOTIFICATIONS.fetch_add(1, Ordering::SeqCst);
    }

    #[test]
    fn display_notification_has_bounded_lifecycle() {
        NOTIFICATIONS.store(0, Ordering::SeqCst);
        start(notified).unwrap();
        let hwnd = WATCHER.lock().unwrap().as_ref().unwrap().hwnd;
        unsafe { PostMessageW(hwnd as HWND, WM_DISPLAYCHANGE, 32, 0) };
        for _ in 0..50 {
            if NOTIFICATIONS.load(Ordering::SeqCst) == 1 {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(NOTIFICATIONS.load(Ordering::SeqCst), 1);
        stop();
        assert!(WATCHER.lock().unwrap().is_none());

        start(notified).unwrap();
        stop();
        assert!(WATCHER.lock().unwrap().is_none());
    }
}
