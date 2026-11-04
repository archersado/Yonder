#![cfg(target_os = "windows")]

use std::{
    env,
    ffi::c_void,
    io::{BufRead, BufReader, Read, Write},
    process::{Child, Command, Stdio},
    ptr::null_mut,
    thread,
    time::Duration,
};

type Handle = *mut c_void;
const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION: i32 = 9;
const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
const SYNCHRONIZE: u32 = 0x0010_0000;
const WAIT_OBJECT_0: u32 = 0;

#[repr(C)]
#[derive(Default)]
struct BasicLimitInformation {
    per_process_user_time_limit: i64,
    per_job_user_time_limit: i64,
    limit_flags: u32,
    minimum_working_set_size: usize,
    maximum_working_set_size: usize,
    active_process_limit: u32,
    affinity: usize,
    priority_class: u32,
    scheduling_class: u32,
}

#[repr(C)]
#[derive(Default)]
struct IoCounters {
    read_operation_count: u64,
    write_operation_count: u64,
    other_operation_count: u64,
    read_transfer_count: u64,
    write_transfer_count: u64,
    other_transfer_count: u64,
}

#[repr(C)]
#[derive(Default)]
struct ExtendedLimitInformation {
    basic_limit_information: BasicLimitInformation,
    io_info: IoCounters,
    process_memory_limit: usize,
    job_memory_limit: usize,
    peak_process_memory_used: usize,
    peak_job_memory_used: usize,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateJobObjectW(attributes: *mut c_void, name: *const u16) -> Handle;
    fn SetInformationJobObject(job: Handle, class: i32, information: *const c_void, length: u32) -> i32;
    fn AssignProcessToJobObject(job: Handle, process: Handle) -> i32;
    fn TerminateJobObject(job: Handle, exit_code: u32) -> i32;
    fn OpenProcess(access: u32, inherit: i32, process_id: u32) -> Handle;
    fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
    fn CloseHandle(handle: Handle) -> i32;
}

struct Job(Handle);

impl Job {
    fn create() -> Result<Self, String> {
        let handle = unsafe { CreateJobObjectW(null_mut(), std::ptr::null()) };
        if handle.is_null() {
            return Err("CreateJobObjectW failed".into());
        }
        let mut information = ExtendedLimitInformation::default();
        information.basic_limit_information.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                handle,
                JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
                (&information as *const ExtendedLimitInformation).cast(),
                std::mem::size_of::<ExtendedLimitInformation>() as u32,
            )
        };
        if configured == 0 {
            unsafe { CloseHandle(handle) };
            return Err("SetInformationJobObject failed".into());
        }
        Ok(Self(handle))
    }

    fn assign(&self, child: &Child) -> Result<(), String> {
        use std::os::windows::io::AsRawHandle;
        let assigned = unsafe { AssignProcessToJobObject(self.0, child.as_raw_handle().cast()) };
        if assigned == 0 { Err("AssignProcessToJobObject failed".into()) } else { Ok(()) }
    }

    fn terminate(&self) -> Result<(), String> {
        if unsafe { TerminateJobObject(self.0, 23) } == 0 {
            Err("TerminateJobObject failed".into())
        } else {
            Ok(())
        }
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

fn process_stopped(process_id: u32) -> bool {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, 0, process_id) };
    if handle.is_null() {
        return true;
    }
    let stopped = unsafe { WaitForSingleObject(handle, 3_000) } == WAIT_OBJECT_0;
    unsafe { CloseHandle(handle) };
    stopped
}

fn child_modes(mode: &str, arguments: &[String]) -> Option<Result<(), String>> {
    match mode {
        "echo-args" => {
            for argument in arguments {
                println!("{}:{}", argument.len(), argument);
            }
            Some(Ok(()))
        }
        "job-parent" => {
            let mut ready = String::new();
            std::io::stdin().read_line(&mut ready).map_err(|error| error.to_string()).ok()?;
            let child = Command::new(env::current_exe().ok()?)
                .arg("job-child")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok()?;
            println!("{}", child.id());
            std::io::stdout().flush().ok()?;
            loop { thread::sleep(Duration::from_secs(1)); }
        }
        "job-child" => loop { thread::sleep(Duration::from_secs(1)); },
        "spam" => {
            let block = vec![b'x'; 4_096];
            for _ in 0..40 {
                std::io::stdout().write_all(&block).ok()?;
            }
            Some(Ok(()))
        }
        _ => None,
    }
}

fn main() -> Result<(), String> {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if let Some(mode) = arguments.first() {
        if let Some(result) = child_modes(mode, &arguments[1..]) {
            return result;
        }
    }

    let executable = env::current_exe().map_err(|error| error.to_string())?;
    let literal = ["plain", "a&b", "$(not-executed)", "semi;colon", "space value", "\"quoted\""];
    let output = Command::new(&executable)
        .arg("echo-args")
        .args(literal)
        .output()
        .map_err(|error| error.to_string())?;
    let returned: Vec<String> = String::from_utf8(output.stdout)
        .map_err(|error| error.to_string())?
        .lines()
        .map(|line| line.split_once(':').map(|(_, value)| value.to_owned()).unwrap_or_default())
        .collect();
    let literal_arguments = returned == literal;

    let job = Job::create()?;
    let mut parent = Command::new(&executable)
        .arg("job-parent")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| error.to_string())?;
    job.assign(&parent)?;
    parent.stdin.as_mut().ok_or("missing parent stdin")?.write_all(b"go\n").map_err(|error| error.to_string())?;
    let mut reader = BufReader::new(parent.stdout.take().ok_or("missing parent stdout")?);
    let mut child_id = String::new();
    reader.read_line(&mut child_id).map_err(|error| error.to_string())?;
    let child_id: u32 = child_id.trim().parse().map_err(|_| "invalid child id")?;
    job.terminate()?;
    let parent_id = parent.id();
    let _ = parent.wait();
    let process_tree_stopped = process_stopped(parent_id) && process_stopped(child_id);

    let spam = Command::new(&executable)
        .arg("spam")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    let mut captured = Vec::new();
    let mut total = 0usize;
    let mut stream = spam.stdout.ok_or("missing spam stdout")?;
    let mut buffer = [0u8; 8_192];
    loop {
        let read = stream.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 { break; }
        total += read;
        let remaining = 65_536usize.saturating_sub(captured.len());
        captured.extend_from_slice(&buffer[..read.min(remaining)]);
    }
    let output_bounded = captured.len() == 65_536 && total > captured.len();
    let passed = literal_arguments && process_tree_stopped && output_bounded;
    println!(
        "{{\"platform\":\"windows\",\"literal_arguments\":{literal_arguments},\"shell_used\":false,\"job_object_tree_stopped\":{process_tree_stopped},\"output_bytes\":{},\"total_output_bytes\":{total},\"truncated\":{},\"passed\":{passed}}}",
        captured.len(), output_bounded
    );
    if passed { Ok(()) } else { Err("Windows command sample failed".into()) }
}
