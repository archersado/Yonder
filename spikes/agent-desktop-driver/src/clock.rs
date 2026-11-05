use std::io;

#[cfg(target_os = "windows")]
pub fn monotonic_millis() -> io::Result<u64> {
    // SAFETY: GetTickCount64 has no arguments and no failure mode.
    Ok(unsafe { windows_sys::Win32::System::SystemInformation::GetTickCount64() })
}

#[cfg(unix)]
pub fn monotonic_millis() -> io::Result<u64> {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: time points to a valid timespec and CLOCK_MONOTONIC is read-only.
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &raw mut time) } != 0 {
        return Err(io::Error::last_os_error());
    }
    let seconds =
        u64::try_from(time.tv_sec).map_err(|_| io::Error::other("negative monotonic clock"))?;
    let nanos =
        u64::try_from(time.tv_nsec).map_err(|_| io::Error::other("negative monotonic clock"))?;
    seconds
        .checked_mul(1_000)
        .and_then(|value| value.checked_add(nanos / 1_000_000))
        .ok_or_else(|| io::Error::other("monotonic clock overflow"))
}

#[cfg(not(any(unix, target_os = "windows")))]
pub fn monotonic_millis() -> io::Result<u64> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "monotonic clock unavailable",
    ))
}
