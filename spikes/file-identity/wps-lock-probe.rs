use std::{ffi::{CStr, CString, c_char, c_int, c_void}, fs::OpenOptions, os::unix::ffi::OsStrExt, path::Path};

unsafe extern "C" {
    fn proc_listpidspath(kind: u32, kind_info: u32, path: *const c_char, flags: u32, buffer: *mut c_void, size: c_int) -> c_int;
    fn proc_name(pid: c_int, buffer: *mut c_void, size: u32) -> c_int;
}

fn holders(path: &Path) -> Vec<(i32, String)> {
    let path = CString::new(path.as_os_str().as_bytes()).unwrap();
    let size = unsafe { proc_listpidspath(1, 0, path.as_ptr(), 2, std::ptr::null_mut(), 0) };
    assert!(size >= 0);
    let mut pids = vec![0_i32; size as usize / size_of::<i32>() + 16];
    let bytes = unsafe { proc_listpidspath(1, 0, path.as_ptr(), 2, pids.as_mut_ptr().cast(), (pids.len() * size_of::<i32>()) as i32) };
    assert!(bytes >= 0);
    pids.truncate(bytes as usize / size_of::<i32>());
    pids.into_iter().filter(|pid| *pid > 0 && *pid != std::process::id() as i32).map(|pid| {
        let mut name = [0_i8; 256];
        let written = unsafe { proc_name(pid, name.as_mut_ptr().cast(), name.len() as u32) };
        let name = if written > 0 { unsafe { CStr::from_ptr(name.as_ptr()) }.to_string_lossy().into_owned() } else { String::new() };
        (pid, name)
    }).collect()
}

fn main() {
    let path = std::env::args_os().nth(1).expect("需要文件路径");
    let expect_open = std::env::args().nth(2).expect("需要open或closed") == "open";
    let path = Path::new(&path).canonicalize().unwrap();
    let holders = holders(&path);
    let file = OpenOptions::new().read(true).write(true).open(&path).unwrap();
    let advisory_lock_detected = file.try_lock().is_err();
    let wps_detected = holders.iter().any(|(_, name)| name == "wpsoffice");
    assert_eq!(wps_detected, expect_open, "WPS文件引用与预期不符: {holders:?}");
    assert_eq!(advisory_lock_detected, expect_open, "WPS系统锁与预期不符");
    println!(r#"{{"expected_open":{expect_open},"wps_detected":{wps_detected},"advisory_lock_detected":{advisory_lock_detected},"passed":true}}"#);
}
