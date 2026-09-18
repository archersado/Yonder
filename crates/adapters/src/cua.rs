use serde::{Deserialize, Serialize};
use std::{fs, io::{BufRead, BufReader, Write}, path::{Path, PathBuf}, process::{Child, ChildStdin, Command, Stdio}, sync::{Mutex, mpsc}, time::{Duration,Instant}};
use yonder_application::{ExecutionAttempt, computer_use::{ComputerAction, ComputerObservation, ComputerUsePort, DispatchOutcome, UnknownReason, WorkTarget, WorkTargetPort}};

#[cfg(target_os="macos")]
unsafe extern "C" { fn yonda_frontmost_work_target(self_pid:i32,pid:*mut u32,window_id:*mut u32)->i32; fn yonda_hid_generation(value:*mut u64)->i32; fn yonda_accessibility_trusted()->i32; }

#[cfg(target_os="macos")]
pub struct MacosFrontmostTarget;
#[cfg(target_os="macos")]
impl MacosFrontmostTarget { pub fn available(&self)->bool{unsafe{yonda_accessibility_trusted()!=0}} }
#[cfg(target_os="macos")]
impl WorkTargetPort for MacosFrontmostTarget {
    fn frontmost(&self)->Result<WorkTarget,UnknownReason>{let(mut pid,mut window_id)=(0,0);let code=unsafe{yonda_frontmost_work_target(std::process::id() as i32,&mut pid,&mut window_id)};if code==0{Ok(WorkTarget{pid,window_id})}else{Err(if code==1{UnknownReason::DependencyUnavailable}else{UnknownReason::InvalidInput})}}
}

pub struct CuaWorker { node: PathBuf, script: PathBuf, sdk: PathBuf, evidence:PathBuf, timeout: Duration, session: Mutex<Option<WorkerSession>> }

struct WorkerSession { child: Child, input: ChildStdin, output: mpsc::Receiver<Result<Vec<u8>, ()>> }

#[derive(Serialize)]
struct Request<'a> {
    task_id: &'a str, step_id: &'a str, attempt_id: &'a str, worker_instance_id: &'a str, host_session_id: &'a str,
    pid: u32, window_id: u32, tool_name: &'a str, arguments: serde_json::Value,
}

#[derive(Deserialize)]
struct Response {
    task_id: String, step_id: String, attempt_id: String, worker_instance_id: String, host_session_id: String,
    action_known: bool, action_succeeded: bool, observe_valid: bool,
    #[serde(default)] failure_stage: Option<String>,
    #[serde(default)] element_count:u16,
    #[serde(default)] screenshot_path:Option<String>,
    #[serde(default)] screenshot_mime:Option<String>,
    #[serde(default)] target_visible:Option<bool>,
}

impl CuaWorker {
    /// 三个路径只能由可信组合根提供；SDK入口必须属于固定0.25.0包。
    pub fn new(node: &Path, script: &Path, sdk: &Path, evidence:&Path, timeout: Duration) -> Result<Self, UnknownReason> {
        if timeout.is_zero() || timeout > Duration::from_secs(120) || [node, script, sdk].iter().any(|path| !path.is_absolute() || !path.is_file()) || !evidence.is_absolute() || !evidence.is_dir() {
            return Err(UnknownReason::InvalidInput);
        }
        let package = sdk.parent().and_then(Path::parent).map(|path| path.join("package.json")).ok_or(UnknownReason::DependencyUnavailable)?;
        let version = fs::read_to_string(package).ok().and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
            .and_then(|value| value.get("version").and_then(|value| value.as_str()).map(str::to_owned));
        if version.as_deref() != Some("0.25.0") { return Err(UnknownReason::DependencyUnavailable); }
        let worker=Self { node: node.into(), script: script.into(), sdk: sdk.into(), evidence:evidence.into(), timeout, session: Mutex::new(None) };
        worker.cleanup();Ok(worker)
    }

    fn start(&self) -> Result<WorkerSession, UnknownReason> {
        let mut child=Command::new(&self.node).arg(&self.script).arg(&self.sdk).arg(&self.evidence)
            .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|_|UnknownReason::WorkerFailed)?;
        let input=child.stdin.take().ok_or(UnknownReason::WorkerFailed)?;
        let output=child.stdout.take().ok_or(UnknownReason::WorkerFailed)?;
        let (sender,receiver)=mpsc::channel();
        std::thread::spawn(move||{
            let mut reader=BufReader::new(output);
            loop {
                let mut line=Vec::new();
                match reader.read_until(b'\n',&mut line) {
                    Ok(0)=>break,
                    Ok(_) if line.len()<=65_536=>if sender.send(Ok(line)).is_err(){break},
                    _=>{let _=sender.send(Err(()));break},
                }
            }
        });
        Ok(WorkerSession{child,input,output:receiver})
    }

    fn stop(session:&mut Option<WorkerSession>){if let Some(mut session)=session.take(){let _=session.child.kill();let _=session.child.wait();}}
    fn cleanup(&self){if let Ok(entries)=std::fs::read_dir(&self.evidence){for entry in entries.flatten(){if entry.file_type().is_ok_and(|kind|kind.is_file()){let _=std::fs::remove_file(entry.path());}}}}

    fn run(&self, attempt: &ExecutionAttempt, target: &WorkTarget, action: &ComputerAction) -> DispatchOutcome {
        if !yonder_application::valid_id(&attempt.task_id) || !yonder_application::valid_id(&attempt.step_id)
            || !yonder_application::valid_id(&attempt.attempt_id) || !yonder_application::valid_id(&attempt.worker_instance_id)
            || !yonder_application::valid_id(&attempt.host_session_id) || target.pid == 0 || target.window_id == 0
            || action.tool_name.is_empty() || action.tool_name.len() > 64 || action.arguments_json.len() > 16*1024 {
            return DispatchOutcome::Unknown(UnknownReason::InvalidInput);
        }
        let arguments=match serde_json::from_str(&action.arguments_json){Ok(value @ serde_json::Value::Object(_))=>value,_=>return DispatchOutcome::Unknown(UnknownReason::InvalidInput)};
        let request = Request { task_id: &attempt.task_id, step_id: &attempt.step_id, attempt_id: &attempt.attempt_id,
            worker_instance_id: &attempt.worker_instance_id, host_session_id: &attempt.host_session_id,
            pid: target.pid, window_id: target.window_id, tool_name:&action.tool_name, arguments };
        let bytes = match serde_json::to_vec(&request) { Ok(bytes) => bytes, Err(_) => return DispatchOutcome::Unknown(UnknownReason::InvalidInput) };
        let mut slot=match self.session.lock(){Ok(slot)=>slot,Err(_)=>return DispatchOutcome::Unknown(UnknownReason::WorkerFailed)};
        if slot.is_none(){*slot=match self.start(){Ok(session)=>Some(session),Err(reason)=>return DispatchOutcome::Unknown(reason)}}
        if slot.as_mut().is_none_or(|session|session.input.write_all(&bytes).and_then(|_|session.input.write_all(b"\n")).and_then(|_|session.input.flush()).is_err()) {
            Self::stop(&mut slot);return DispatchOutcome::Unknown(UnknownReason::WorkerFailed);
        }
        let started=Instant::now();
        #[cfg(target_os="macos")]
        let mut initial_hid=0u64;
        #[cfg(target_os="macos")]
        if unsafe{yonda_hid_generation(&mut initial_hid)}==0{Self::stop(&mut slot);return DispatchOutcome::Unknown(UnknownReason::DependencyUnavailable)}
        let output = loop {
            match slot.as_ref().unwrap().output.recv_timeout(Duration::from_millis(25)) {
                Ok(Ok(output))=>break output,
                Ok(Err(()))=>{Self::stop(&mut slot);return DispatchOutcome::Unknown(UnknownReason::InvalidResponse)},
                Err(mpsc::RecvTimeoutError::Disconnected)=>{Self::stop(&mut slot);return DispatchOutcome::Unknown(UnknownReason::WorkerFailed)},
                Err(mpsc::RecvTimeoutError::Timeout)=>{},
            }
            #[cfg(target_os="macos")]
            {let mut current=initial_hid;if unsafe{yonda_hid_generation(&mut current)}==0{Self::stop(&mut slot);return DispatchOutcome::Unknown(UnknownReason::DependencyUnavailable)}if current!=initial_hid{Self::stop(&mut slot);return DispatchOutcome::Unknown(UnknownReason::UserInput)}}
            if started.elapsed()>=self.timeout{Self::stop(&mut slot);return DispatchOutcome::Unknown(UnknownReason::TimedOut)}
        };
        let outcome=classify(attempt, &output,&self.evidence);
        if matches!(outcome,DispatchOutcome::Unknown(_)){Self::stop(&mut slot);self.cleanup()}
        outcome
    }
}

impl Drop for CuaWorker { fn drop(&mut self){if let Ok(slot)=self.session.get_mut(){Self::stop(slot)}self.cleanup()} }

impl ComputerUsePort for CuaWorker {
    fn dispatch(&self, attempt: &ExecutionAttempt, target: &WorkTarget, action: &ComputerAction) -> DispatchOutcome {
        self.run(attempt,target,action)
    }
    fn end_session(&self){if let Ok(mut slot)=self.session.lock(){Self::stop(&mut slot)}self.cleanup()}
}

fn classify(attempt: &ExecutionAttempt, output: &[u8], evidence:&Path) -> DispatchOutcome {
    let response: Response = match serde_json::from_slice(output) { Ok(value) => value, Err(_) => return DispatchOutcome::Unknown(UnknownReason::InvalidResponse) };
    if response.task_id != attempt.task_id || response.step_id != attempt.step_id || response.attempt_id != attempt.attempt_id
        || response.worker_instance_id != attempt.worker_instance_id || response.host_session_id != attempt.host_session_id {
        return DispatchOutcome::Unknown(UnknownReason::IdentityMismatch);
    }
    if !response.action_known { #[cfg(debug_assertions)] eprintln!("cua worker failed at {}",response.failure_stage.as_deref().unwrap_or("unknown")); return DispatchOutcome::Unknown(UnknownReason::WorkerFailed); }
    if !response.observe_valid { #[cfg(debug_assertions)] eprintln!("cua worker failed at {}",response.failure_stage.as_deref().unwrap_or("unknown")); return DispatchOutcome::Unknown(UnknownReason::ObserveFailed); }
    let screenshot=match (response.screenshot_path,response.screenshot_mime){
        (Some(path),Some(mime)) if matches!(mime.as_str(),"image/png"|"image/jpeg"|"image/webp")=>{let path=PathBuf::from(path);let valid=path.parent()==Some(evidence)&&std::fs::symlink_metadata(&path).is_ok_and(|value|value.file_type().is_file()&&value.len()<=4*1024*1024);if !valid{return DispatchOutcome::Unknown(UnknownReason::InvalidResponse)}Some((path.to_string_lossy().into_owned(),mime))},
        (None,None)=>None,
        _=>return DispatchOutcome::Unknown(UnknownReason::InvalidResponse),
    };
    DispatchOutcome::Known { action_succeeded: response.action_succeeded, observation:Some(ComputerObservation{element_count:response.element_count,screenshot_path:screenshot.as_ref().map(|value|value.0.clone()),screenshot_mime:screenshot.map(|value|value.1),target_visible:response.target_visible}) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yonder_application::AttemptPhase;

    #[test]
    fn result_requires_exact_identity_and_post_action_observe() {
        let evidence=std::env::temp_dir();
        let attempt = ExecutionAttempt { task_id: "task".into(), step_id: "step".into(), attempt_id: "attempt".into(), worker_instance_id: "worker".into(), host_session_id: "host".into(), phase: AttemptPhase::Prepared, accepted_sequence: 2 };
        let response = |worker: &str, observe: bool| format!(r#"{{"task_id":"task","step_id":"step","attempt_id":"attempt","worker_instance_id":"{worker}","host_session_id":"host","action_known":true,"action_succeeded":true,"observe_valid":{observe}}}"#);
        assert_eq!(classify(&attempt, response("worker", true).as_bytes(),&evidence), DispatchOutcome::Known { action_succeeded: true, observation:Some(ComputerObservation{element_count:0,screenshot_path:None,screenshot_mime:None,target_visible:None}) });
        assert_eq!(classify(&attempt, response("old", true).as_bytes(),&evidence), DispatchOutcome::Unknown(UnknownReason::IdentityMismatch));
        assert_eq!(classify(&attempt, response("worker", false).as_bytes(),&evidence), DispatchOutcome::Unknown(UnknownReason::ObserveFailed));
        assert_eq!(classify(&attempt, b"{}",&evidence), DispatchOutcome::Unknown(UnknownReason::InvalidResponse));
    }

    #[cfg(unix)]
    #[test]
    fn normal_actions_reuse_worker_session() {
        let root=std::env::temp_dir().join(format!("yonda-cua-session-{}",std::process::id()));
        let package=root.join("node_modules/fake");
        std::fs::create_dir_all(package.join("dist")).unwrap();
        std::fs::write(package.join("package.json"),r#"{"version":"0.25.0"}"#).unwrap();
        let sdk=package.join("dist/index.js");std::fs::write(&sdk,b"").unwrap();
        let script=root.join("worker.sh");
        std::fs::write(&script,b"count=0\nwhile IFS= read -r line; do count=$((count+1)); if [ $count -eq 1 ]; then ok=true; else ok=false; fi; printf '{\"task_id\":\"task\",\"step_id\":\"step\",\"attempt_id\":\"attempt\",\"worker_instance_id\":\"worker\",\"host_session_id\":\"host\",\"action_known\":true,\"action_succeeded\":%s,\"observe_valid\":true}\\n' $ok; done\n").unwrap();
        let evidence=root.join("evidence");std::fs::create_dir(&evidence).unwrap();
        let worker=CuaWorker::new(Path::new("/bin/sh"),&script,&sdk,&evidence,Duration::from_secs(2)).unwrap();
        let screenshot=evidence.join("task-attempt.png");std::fs::write(&screenshot,b"png").unwrap();
        worker.end_session();assert!(!screenshot.exists());
        let mut session=worker.start().unwrap();
        session.input.write_all(b"one\n").unwrap();session.input.flush().unwrap();
        assert!(String::from_utf8(session.output.recv_timeout(Duration::from_secs(2)).unwrap().unwrap()).unwrap().contains("\"action_succeeded\":true"));
        session.input.write_all(b"two\n").unwrap();session.input.flush().unwrap();
        assert!(String::from_utf8(session.output.recv_timeout(Duration::from_secs(2)).unwrap().unwrap()).unwrap().contains("\"action_succeeded\":false"));
        let mut slot=Some(session);CuaWorker::stop(&mut slot);drop(worker);std::fs::remove_dir_all(root).unwrap();
    }

}
