#![cfg(target_os = "windows")]

use std::{
    error::Error,
    process::{Command, Stdio},
};
use yonder_agent_desktop_driver_spike::containment::{CleanupVerdict, Containment, WindowsJob};

#[test]
fn suspended_child_is_job_assigned_then_can_be_terminated_and_waited() -> Result<(), Box<dyn Error>>
{
    let mut job = WindowsJob::new()?;
    let mut command = Command::new("cmd.exe");
    command
        .args(["/d", "/c", "ping -n 30 127.0.0.1 >NUL"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = job.spawn(&mut command)?;
    assert_eq!(
        job.terminate_and_probe(&mut child).verdict(),
        CleanupVerdict::Unproven
    );
    Ok(())
}
