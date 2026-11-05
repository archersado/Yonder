use super::{CleanupEvidence, CleanupProbes, Containment, finalize_cleanup};
use std::{
    io,
    mem::size_of,
    process::{Child, Command},
    ptr,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
        },
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject, TerminateJobObject,
        },
        Threading::{
            CREATE_SUSPENDED, OpenProcess, OpenThread, PROCESS_QUERY_LIMITED_INFORMATION,
            ResumeThread, THREAD_SUSPEND_RESUME, WaitForSingleObject,
        },
    },
};

const SYNCHRONIZE: u32 = 0x0010_0000;

pub struct WindowsJob {
    handle: HANDLE,
}

impl WindowsJob {
    pub fn new() -> io::Result<Self> {
        // SAFETY: null security/name requests an unnamed job with default security.
        let handle = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: info is initialized and the size matches the requested information class.
        let ok = unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                (&raw const info).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if ok == 0 {
            unsafe { CloseHandle(handle) };
            return Err(io::Error::last_os_error());
        }
        Ok(Self { handle })
    }
}

impl Containment for WindowsJob {
    fn spawn(&mut self, command: &mut Command) -> io::Result<Child> {
        use std::os::windows::{io::AsRawHandle, process::CommandExt};
        command.creation_flags(CREATE_SUSPENDED);
        let mut child = command.spawn()?;
        // SAFETY: both handles are live and owned for the duration of this call.
        if unsafe { AssignProcessToJobObject(self.handle, child.as_raw_handle() as HANDLE) } == 0 {
            let error = io::Error::last_os_error();
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        if let Err(error) = resume_process_thread(child.id()) {
            // The process is already in the job. Terminating the job and waiting prevents a
            // suspended or partially resumed child from escaping this failed spawn.
            unsafe { TerminateJobObject(self.handle, 23) };
            let _ = child.wait();
            return Err(error);
        }
        Ok(child)
    }
    fn terminate_and_probe(&mut self, child: &mut Child) -> CleanupEvidence {
        let pid = child.id();
        // SAFETY: handle is a live job handle.
        let terminated = unsafe { TerminateJobObject(self.handle, 23) } != 0;
        let waited = child.wait().is_ok();
        // SAFETY: read-only process handle/probe for the known supervised pid.
        let process =
            unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, 0, pid) };
        let gone = if process.is_null() {
            true
        } else {
            let stopped = unsafe { WaitForSingleObject(process, 0) } == WAIT_OBJECT_0;
            unsafe { CloseHandle(process) };
            stopped
        };
        // Job/进程探针不能证明 UI 句柄、hook、权限与临时状态均已释放。
        finalize_cleanup(CleanupProbes {
            process_tree_gone: gone && (terminated || waited),
            ..Default::default()
        })
    }
}

fn resume_process_thread(process_id: u32) -> io::Result<()> {
    // SAFETY: snapshot handle is closed below and THREADENTRY32 carries its declared size.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let mut entry = THREADENTRY32 {
        dwSize: size_of::<THREADENTRY32>() as u32,
        ..Default::default()
    };
    let mut found = false;
    // SAFETY: snapshot and entry are valid for the enumeration loop.
    let mut has_entry = unsafe { Thread32First(snapshot, &raw mut entry) } != 0;
    while has_entry {
        if entry.th32OwnerProcessID == process_id {
            found = true;
            break;
        }
        has_entry = unsafe { Thread32Next(snapshot, &raw mut entry) } != 0;
    }
    unsafe { CloseHandle(snapshot) };
    if !found {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "suspended primary thread not found",
        ));
    }
    // SAFETY: the enumerated thread belongs to the just-created suspended process.
    let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
    if thread.is_null() {
        return Err(io::Error::last_os_error());
    }
    let resumed = unsafe { ResumeThread(thread) };
    unsafe { CloseHandle(thread) };
    if resumed == u32::MAX {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

impl Drop for WindowsJob {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.handle) };
    }
}
