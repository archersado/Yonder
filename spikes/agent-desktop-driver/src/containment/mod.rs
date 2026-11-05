use std::{
    io,
    process::{Child, Command},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupVerdict {
    Released,
    Residual,
    Unproven,
}

/// 只能由 containment 探针产出的 cleanup 记账；调用方不能伪造 Released。
#[derive(Debug)]
pub struct CleanupEvidence {
    verdict: CleanupVerdict,
}

impl CleanupEvidence {
    pub fn verdict(&self) -> CleanupVerdict {
        self.verdict
    }
    #[cfg(test)]
    pub(crate) fn unproven() -> Self {
        Self::from_verdict(CleanupVerdict::Unproven)
    }
    fn from_verdict(verdict: CleanupVerdict) -> Self {
        Self { verdict }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CleanupProbes {
    pub process_tree_gone: bool,
    pub desktop_handles_gone: bool,
    pub input_hooks_gone: bool,
    pub permissions_released: bool,
    pub temp_state_removed: bool,
}

fn finalize_cleanup(probes: CleanupProbes) -> CleanupEvidence {
    if probes.process_tree_gone
        && probes.desktop_handles_gone
        && probes.input_hooks_gone
        && probes.permissions_released
        && probes.temp_state_removed
    {
        CleanupEvidence::from_verdict(CleanupVerdict::Released)
    } else if !probes.process_tree_gone {
        CleanupEvidence::from_verdict(CleanupVerdict::Residual)
    } else {
        CleanupEvidence::from_verdict(CleanupVerdict::Unproven)
    }
}

pub trait Containment {
    fn spawn(&mut self, command: &mut Command) -> io::Result<Child>;
    fn terminate_and_probe(&mut self, child: &mut Child) -> CleanupEvidence;
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub struct MockContainment;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
impl Containment for MockContainment {
    fn spawn(&mut self, command: &mut Command) -> io::Result<Child> {
        command.spawn()
    }
    fn terminate_and_probe(&mut self, child: &mut Child) -> CleanupEvidence {
        let _ = child.kill();
        let _ = child.wait();
        // 仅证明进程退出不足以证明桌面句柄、输入 hook、权限和临时状态均释放。
        CleanupEvidence::from_verdict(CleanupVerdict::Unproven)
    }
}

#[cfg(target_os = "macos")]
pub struct MacProcessGroup {
    pgid: Option<i32>,
}

#[cfg(target_os = "macos")]
impl MacProcessGroup {
    pub fn new() -> Self {
        Self { pgid: None }
    }
}

#[cfg(target_os = "macos")]
impl Containment for MacProcessGroup {
    fn spawn(&mut self, command: &mut Command) -> io::Result<Child> {
        use std::os::unix::process::CommandExt;
        // SAFETY: pre_exec only invokes async-signal-safe setpgid and captures no mutable state.
        unsafe {
            command.pre_exec(|| {
                if libc::setpgid(0, 0) == 0 {
                    Ok(())
                } else {
                    Err(io::Error::last_os_error())
                }
            });
        }
        let child = command.spawn()?;
        self.pgid = Some(child.id() as i32);
        Ok(child)
    }
    fn terminate_and_probe(&mut self, child: &mut Child) -> CleanupEvidence {
        let Some(pgid) = self.pgid.take() else {
            return CleanupEvidence::from_verdict(CleanupVerdict::Unproven);
        };
        // SAFETY: negative pgid targets only the supervised process group created above.
        let killed = unsafe { libc::kill(-pgid, libc::SIGKILL) } == 0;
        let waited = child.wait().is_ok();
        // SAFETY: signal 0 is a read-only existence probe and sends no signal.
        let gone = unsafe { libc::kill(-pgid, 0) } != 0;
        if killed && waited && gone {
            CleanupEvidence::from_verdict(CleanupVerdict::Unproven)
        } else if gone {
            CleanupEvidence::from_verdict(CleanupVerdict::Unproven)
        } else {
            CleanupEvidence::from_verdict(CleanupVerdict::Residual)
        }
    }
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::WindowsJob;
