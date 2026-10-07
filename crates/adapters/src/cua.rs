use serde::{Deserialize, Serialize};
use std::{fs, io::{BufRead, BufReader, Write}, path::{Path, PathBuf}, process::{Child, ChildStdin, Command, Stdio}, sync::{Mutex, mpsc}, time::{Duration,Instant}};
#[cfg(target_os="macos")]
use std::ffi::CString;
use yonder_application::{ExecutionAttempt, ObservedElement, computer_use::{ComputerAction, ComputerObservation, ComputerUsePort, DispatchOutcome, UnknownReason, WorkTarget, WorkTargetPort}};

#[cfg(target_os="macos")]
unsafe extern "C" { fn yonda_frontmost_work_target(self_pid:i32,pid:*mut u32,window_id:*mut u32)->i32; fn yonda_accessibility_trusted()->i32; fn yonda_activate_window_for_app(bundle_id:*const std::ffi::c_char)->i32; }

#[cfg(target_os="macos")]
pub struct MacosFrontmostTarget;
#[cfg(target_os="macos")]
impl MacosFrontmostTarget { pub fn available(&self)->bool{unsafe{yonda_accessibility_trusted()!=0}} }
#[cfg(target_os="macos")]
impl WorkTargetPort for MacosFrontmostTarget {
    fn frontmost(&self)->Result<WorkTarget,UnknownReason>{let(mut pid,mut window_id)=(0,0);let code=unsafe{yonda_frontmost_work_target(std::process::id() as i32,&mut pid,&mut window_id)};if code==0{Ok(WorkTarget{pid,window_id})}else{Err(if code==1{UnknownReason::DependencyUnavailable}else{UnknownReason::InvalidInput})}}
}

/// 只由已验证 Driver 身份构造；Agent/UI 不能提供或覆盖应用/窗口身份。
pub struct WindowActivationTarget { application_id:String, window_id:Option<u64> }
impl WindowActivationTarget {
    pub fn application_id(&self)->&str {&self.application_id}
    pub fn window_id(&self)->Option<u64> {self.window_id}
}

/// Yonder 统一的窗口激活能力。macOS 使用 AX 原生桥，其他平台可委托 Sky activate_window。
pub trait WindowActivationPort:Send+Sync { fn activate_window(&self,target:&WindowActivationTarget)->Result<(),UnknownReason>; }

#[cfg(target_os="macos")]
struct MacosWindowActivator;
#[cfg(target_os="macos")]
impl WindowActivationPort for MacosWindowActivator {
    fn activate_window(&self,target:&WindowActivationTarget)->Result<(),UnknownReason>{
        if !valid_sky_app_id(target.application_id()){return Err(UnknownReason::InvalidInput)}
        let value=CString::new(target.application_id()).map_err(|_|UnknownReason::InvalidInput)?;
        match unsafe{yonda_activate_window_for_app(value.as_ptr())}{0=>Ok(()),1=>Err(UnknownReason::DependencyUnavailable),_=>Err(UnknownReason::ObserveFailed)}
    }
}

pub struct CuaWorker { node: PathBuf, script: PathBuf, bridge: PathBuf, evidence:PathBuf, timeout: Duration, window_activation:Option<Box<dyn WindowActivationPort>>, runtime: Mutex<WorkerRuntime> }

struct WorkerSession { child: Child, input: ChildStdin, output: mpsc::Receiver<Result<Vec<u8>, ()>> }

struct WorkerRuntime { process:Option<WorkerSession>, binding:Option<TargetBinding> }

#[derive(Clone,Debug,PartialEq,Eq)]
struct TargetBinding { task_id:String, application_id:String }

#[derive(Serialize)]
struct Request<'a> {
    task_id: &'a str, step_id: &'a str, attempt_id: &'a str, worker_instance_id: &'a str, host_session_id: &'a str,
    pid: u32, window_id: u32, tool_name: &'a str, arguments: serde_json::Value,
    #[serde(skip_serializing_if="Option::is_none")]
    bound_application_id:Option<&'a str>,
}

#[derive(Deserialize)]
struct Response {
    task_id: String, step_id: String, attempt_id: String, worker_instance_id: String, host_session_id: String,
    action_known: bool, action_succeeded: bool, observe_valid: bool,
    action_effect:Option<ActionEffect>,
    #[serde(default)] failure_stage: Option<String>,
    #[serde(default)] element_count:u16,
    #[serde(default)] screenshot_path:Option<String>,
    #[serde(default)] screenshot_mime:Option<String>,
    #[serde(default)] target_visible:Option<bool>,
    #[serde(default)] observation_ref:Option<String>,
    #[serde(default)] transcript:Option<String>,
    #[serde(default)] elements:Vec<ObservedElement>,
    #[serde(default)] launched_app_id:Option<String>,
}

#[derive(Clone,Copy,Deserialize)]
#[serde(rename_all="snake_case")]
enum ActionEffect { Confirmed,Partial,Unverifiable,SuspectedNoop,Refused }

fn valid_sky_app_id(value:&str)->bool {value.len()<=255&&!value.is_empty()&&value.bytes().all(|byte|byte.is_ascii_alphanumeric()||matches!(byte,b'.'|b'-'))}

#[cfg(all(target_os="macos",not(test)))]
fn valid_sky_release_identity(node:&Path,bridge:&Path)->bool {
    for path in [node,bridge] {
        if !Command::new("/usr/bin/codesign").args(["--verify","--strict","--verbose=0"]).arg(path).status().is_ok_and(|status|status.success()){return false}
    }
    let node_identity=Command::new("/usr/bin/codesign").args(["-dv","--verbose=4"]).arg(node).output();
    let identity=Command::new("/usr/bin/codesign").args(["-dv","--verbose=4"]).arg(bridge).output();
    let entitlements=Command::new("/usr/bin/codesign").args(["-d","--entitlements",":-"]).arg(bridge).output();
    let (Ok(node_identity),Ok(identity),Ok(entitlements))=(node_identity,identity,entitlements) else{return false};
    let node_identity=String::from_utf8_lossy(&node_identity.stderr);
    let identity=String::from_utf8_lossy(&identity.stderr);
    let entitlements=format!("{}{}",String::from_utf8_lossy(&entitlements.stdout),String::from_utf8_lossy(&entitlements.stderr));
    node_identity.contains("Identifier=node") && node_identity.contains("TeamIdentifier=2DC432GLL2")
        && identity.contains("Identifier=com.openai.sky.CUAService.cli")
        && identity.contains("TeamIdentifier=2DC432GLL2")
        && entitlements.contains("2DC432GLL2.com.openai.sky.CUAService")
}

#[cfg(any(not(target_os="macos"),test))]
fn valid_sky_release_identity(_node:&Path,_bridge:&Path)->bool {true}

impl CuaWorker {
    /// 三个路径只能由可信组合根提供；SDK入口必须属于固定Sky包。
    pub fn new(script: &Path, sdk: &Path, evidence:&Path, timeout: Duration) -> Result<Self, UnknownReason> {
        #[cfg(target_os="macos")]
        let window_activation=Some(Box::new(MacosWindowActivator) as Box<dyn WindowActivationPort>);
        #[cfg(not(target_os="macos"))]
        let window_activation=None;
        Self::new_for_sdk(script, sdk, evidence, timeout, window_activation)
    }

    fn new_for_sdk(script: &Path, sdk: &Path, evidence:&Path, timeout: Duration, window_activation:Option<Box<dyn WindowActivationPort>>) -> Result<Self, UnknownReason> {
        if timeout.is_zero() || timeout > Duration::from_secs(120) || [script, sdk].iter().any(|path| !path.is_absolute() || !path.is_file()) || !evidence.is_absolute() || !evidence.is_dir() {
            return Err(UnknownReason::InvalidInput);
        }
        let package = sdk.ancestors().take(12).map(|path| path.join("package.json")).find(|path| path.is_file()).ok_or(UnknownReason::DependencyUnavailable)?;
        let identity = fs::read_to_string(&package).ok().and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok());
        let name = identity.as_ref().and_then(|value| value.get("name")).and_then(|value| value.as_str());
        let version = identity.as_ref().and_then(|value| value.get("version")).and_then(|value| value.as_str());
        if name != Some("@oai/sky") || version != Some("0.7.1") { return Err(UnknownReason::DependencyUnavailable); }
        let package_root=package.parent().ok_or(UnknownReason::DependencyUnavailable)?;
        let runtime_root=package_root.ancestors().nth(4).ok_or(UnknownReason::DependencyUnavailable)?;
        let node=runtime_root.join("bin/node");
        let bridge=package_root.join("Codex Computer Use.app/Contents/SharedSupport/SkyComputerUseClient.app/Contents/MacOS/SkyComputerUseClient");
        let node_metadata=fs::symlink_metadata(&node).map_err(|_|UnknownReason::DependencyUnavailable)?;
        let bridge_metadata=fs::symlink_metadata(&bridge).map_err(|_|UnknownReason::DependencyUnavailable)?;
        if !node.is_absolute() || !node_metadata.is_file() || node_metadata.file_type().is_symlink()
            || !bridge.is_absolute() || !bridge_metadata.is_file() || bridge_metadata.file_type().is_symlink()
            || !valid_sky_release_identity(&node,&bridge){return Err(UnknownReason::DependencyUnavailable)}
        let worker=Self { node, script: script.into(), bridge, evidence:evidence.into(), timeout, window_activation,
            runtime:Mutex::new(WorkerRuntime{process:None,binding:None}) };
        worker.cleanup();Ok(worker)
    }

    fn start(&self) -> Result<WorkerSession, UnknownReason> {
        let mut child=Command::new(&self.node).arg(&self.script).arg(&self.bridge).arg(&self.evidence)
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
        let mut runtime=match self.runtime.lock(){Ok(runtime)=>runtime,Err(_)=>return DispatchOutcome::Unknown(UnknownReason::WorkerFailed)};
        if runtime.binding.as_ref().is_some_and(|binding|binding.task_id!=attempt.task_id) {
            Self::stop(&mut runtime.process);runtime.binding=None;
        }
        if action.tool_name=="launch_app" {runtime.binding=None;}
        let bound_application_id=runtime.binding.as_ref().filter(|binding|binding.task_id==attempt.task_id).map(|binding|binding.application_id.clone());
        let request = Request { task_id: &attempt.task_id, step_id: &attempt.step_id, attempt_id: &attempt.attempt_id,
            worker_instance_id: &attempt.worker_instance_id, host_session_id: &attempt.host_session_id,
            pid: target.pid, window_id: target.window_id, tool_name:&action.tool_name, arguments,
            bound_application_id:bound_application_id.as_deref() };
        let bytes = match serde_json::to_vec(&request) { Ok(bytes) => bytes, Err(_) => return DispatchOutcome::Unknown(UnknownReason::InvalidInput) };
        if runtime.process.is_none(){runtime.process=match self.start(){Ok(session)=>Some(session),Err(reason)=>return DispatchOutcome::Unknown(reason)}}
        if action.tool_name=="bring_to_front" {
            if let Some(port)=self.window_activation.as_deref(){if let Err(reason)=activate_cached_window(port,runtime.binding.as_ref(),&attempt.task_id){return DispatchOutcome::Unknown(reason)}}
        }
        if runtime.process.as_mut().is_none_or(|session|session.input.write_all(&bytes).and_then(|_|session.input.write_all(b"\n")).and_then(|_|session.input.flush()).is_err()) {
            Self::stop(&mut runtime.process);return DispatchOutcome::Unknown(UnknownReason::WorkerFailed);
        }
        let started=Instant::now();
        let output = loop {
            match runtime.process.as_ref().unwrap().output.recv_timeout(Duration::from_millis(25)) {
                Ok(Ok(output))=>break output,
                Ok(Err(()))=>{Self::stop(&mut runtime.process);return DispatchOutcome::Unknown(UnknownReason::InvalidResponse)},
                Err(mpsc::RecvTimeoutError::Disconnected)=>{Self::stop(&mut runtime.process);return DispatchOutcome::Unknown(UnknownReason::WorkerFailed)},
                Err(mpsc::RecvTimeoutError::Timeout)=>{},
            }
            if started.elapsed()>=self.timeout{Self::stop(&mut runtime.process);return DispatchOutcome::Unknown(UnknownReason::TimedOut)}
        };
        let outcome=classify(attempt, &output,&self.evidence);
        if action.tool_name=="launch_app" && matches!(outcome,DispatchOutcome::Known{action_succeeded:true,..}) {
            let launched=serde_json::from_slice::<Response>(&output).ok().and_then(|response|response.launched_app_id)
                .filter(|value|valid_sky_app_id(value));
            let Some(bundle_id)=launched else{Self::stop(&mut runtime.process);self.cleanup();return DispatchOutcome::Unknown(UnknownReason::InvalidResponse)};
            runtime.binding=Some(TargetBinding{task_id:attempt.task_id.clone(),application_id:bundle_id});
        }
        if matches!(outcome,DispatchOutcome::Unknown(_)){Self::stop(&mut runtime.process);self.cleanup()}
        // 执行进程和可信应用绑定是两个生命周期。Unknown可销毁Worker与签名Client，
        // 但同任务绑定仍由Adapter运行态持有，供外部慢脑新鲜Observe/replan后重建执行资源。
        outcome
    }
}

fn activate_cached_window(port:&dyn WindowActivationPort,binding:Option<&TargetBinding>,task_id:&str)->Result<(),UnknownReason>{
    let Some(binding)=binding.filter(|binding|binding.task_id==task_id) else{return Err(UnknownReason::InvalidInput)};
    if !valid_sky_app_id(&binding.application_id){return Err(UnknownReason::InvalidResponse)}
    port.activate_window(&WindowActivationTarget{application_id:binding.application_id.clone(),window_id:None})
}

impl Drop for CuaWorker { fn drop(&mut self){if let Ok(runtime)=self.runtime.get_mut(){Self::stop(&mut runtime.process);runtime.binding=None}self.cleanup()} }

impl ComputerUsePort for CuaWorker {
    fn dispatch(&self, attempt: &ExecutionAttempt, target: &WorkTarget, action: &ComputerAction) -> DispatchOutcome {
        self.run(attempt,target,action)
    }
    fn end_session(&self){if let Ok(mut runtime)=self.runtime.lock(){Self::stop(&mut runtime.process);runtime.binding=None}self.cleanup()}
}

fn classify(attempt: &ExecutionAttempt, output: &[u8], evidence:&Path) -> DispatchOutcome {
    let response: Response = match serde_json::from_slice(output) { Ok(value) => value, Err(_) => return DispatchOutcome::Unknown(UnknownReason::InvalidResponse) };
    if response.task_id != attempt.task_id || response.step_id != attempt.step_id || response.attempt_id != attempt.attempt_id
        || response.worker_instance_id != attempt.worker_instance_id || response.host_session_id != attempt.host_session_id {
        return DispatchOutcome::Unknown(UnknownReason::IdentityMismatch);
    }
    if !response.action_known { #[cfg(debug_assertions)] eprintln!("cua worker failed at {}",response.failure_stage.as_deref().unwrap_or("unknown")); return DispatchOutcome::Unknown(UnknownReason::WorkerFailed); }
    if !response.observe_valid { #[cfg(debug_assertions)] eprintln!("cua worker failed at {}",response.failure_stage.as_deref().unwrap_or("unknown")); return DispatchOutcome::Unknown(UnknownReason::ObserveFailed); }
    let effect=match response.action_effect{Some(effect)=>effect,None=>return DispatchOutcome::Unknown(UnknownReason::InvalidResponse)};
    if response.action_succeeded!=matches!(effect,ActionEffect::Confirmed){return DispatchOutcome::Unknown(UnknownReason::InvalidResponse)}
    let screenshot=match (response.screenshot_path,response.screenshot_mime){
        (Some(path),Some(mime)) if matches!(mime.as_str(),"image/png"|"image/jpeg"|"image/webp")=>{let path=PathBuf::from(path);let valid=path.parent()==Some(evidence)&&std::fs::symlink_metadata(&path).is_ok_and(|value|value.file_type().is_file()&&value.len()<=4*1024*1024);if !valid{return DispatchOutcome::Unknown(UnknownReason::InvalidResponse)}Some((path.to_string_lossy().into_owned(),mime))},
        (None,None)=>None,
        _=>return DispatchOutcome::Unknown(UnknownReason::InvalidResponse),
    };
    let mut indexes=std::collections::HashSet::new();
    let valid_elements=response.elements.len()<=128
        && response.elements.len()<=usize::from(response.element_count)
        && response.elements.iter().all(|element|element.index>0
            && indexes.insert(element.index)
            && element.label.as_deref().is_none_or(|label|!label.is_empty()&&label.chars().count()<=128&&!label.chars().any(char::is_control)));
    let valid_reference=response.observation_ref.as_deref().is_none_or(yonder_application::valid_id);
    let valid_transcript=response.transcript.as_deref().is_none_or(|value|value.len()<=64*1024&&!value.contains('\0'));
    if !valid_elements||!valid_reference||!valid_transcript||((!response.elements.is_empty()||response.transcript.is_some())&&response.observation_ref.is_none()){return DispatchOutcome::Unknown(UnknownReason::InvalidResponse)}
    let observation=ComputerObservation{element_count:response.element_count,screenshot_path:screenshot.as_ref().map(|value|value.0.clone()),screenshot_mime:screenshot.map(|value|value.1),target_visible:response.target_visible,observation_ref:response.observation_ref,transcript:response.transcript,elements:response.elements};
    if matches!(effect,ActionEffect::Partial|ActionEffect::Unverifiable|ActionEffect::SuspectedNoop){#[cfg(debug_assertions)] eprintln!("cua worker unverified at {}",response.failure_stage.as_deref().unwrap_or("action-unconfirmed"));return DispatchOutcome::UnknownObserved { reason:UnknownReason::ObserveFailed, observation }}
    DispatchOutcome::Known { action_succeeded: matches!(effect,ActionEffect::Confirmed), observation:Some(observation) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yonder_application::AttemptPhase;

    struct RecordingActivator(Mutex<Vec<String>>);
    impl WindowActivationPort for RecordingActivator {
        fn activate_window(&self,target:&WindowActivationTarget)->Result<(),UnknownReason>{assert_eq!(target.window_id(),None);self.0.lock().unwrap().push(target.application_id().into());Ok(())}
    }

    #[test]
    fn result_requires_exact_identity_and_post_action_observe() {
        let evidence=std::env::temp_dir();
        let attempt = ExecutionAttempt { task_id: "task".into(), step_id: "step".into(), attempt_id: "attempt".into(), worker_instance_id: "worker".into(), host_session_id: "host".into(), phase: AttemptPhase::Prepared, accepted_sequence: 2 };
        let response = |worker: &str, observe: bool, succeeded:bool, effect:&str| format!(r#"{{"task_id":"task","step_id":"step","attempt_id":"attempt","worker_instance_id":"{worker}","host_session_id":"host","action_known":true,"action_succeeded":{succeeded},"action_effect":"{effect}","observe_valid":{observe}}}"#);
        assert_eq!(classify(&attempt, response("worker", true,true,"confirmed").as_bytes(),&evidence), DispatchOutcome::Known { action_succeeded: true, observation:Some(ComputerObservation{element_count:0,screenshot_path:None,screenshot_mime:None,target_visible:None,observation_ref:None,transcript:None,elements:Vec::new()}) });
        assert_eq!(classify(&attempt, response("old", true,true,"confirmed").as_bytes(),&evidence), DispatchOutcome::Unknown(UnknownReason::IdentityMismatch));
        assert_eq!(classify(&attempt, response("worker", false,true,"confirmed").as_bytes(),&evidence), DispatchOutcome::Unknown(UnknownReason::ObserveFailed));
        assert_eq!(classify(&attempt, response("worker", true,false,"unverifiable").as_bytes(),&evidence), DispatchOutcome::UnknownObserved { reason:UnknownReason::ObserveFailed, observation:ComputerObservation{element_count:0,screenshot_path:None,screenshot_mime:None,target_visible:None,observation_ref:None,transcript:None,elements:Vec::new()} });
        assert_eq!(classify(&attempt, response("worker", true,false,"refused").as_bytes(),&evidence), DispatchOutcome::Known { action_succeeded:false, observation:Some(ComputerObservation{element_count:0,screenshot_path:None,screenshot_mime:None,target_visible:None,observation_ref:None,transcript:None,elements:Vec::new()}) });
        assert_eq!(classify(&attempt, response("worker", true,true,"unverifiable").as_bytes(),&evidence), DispatchOutcome::Unknown(UnknownReason::InvalidResponse));
        assert_eq!(classify(&attempt, b"{}",&evidence), DispatchOutcome::Unknown(UnknownReason::InvalidResponse));
    }

    #[test]
    fn result_keeps_bounded_ephemeral_transcript_and_requires_its_reference() {
        let evidence=std::env::temp_dir();
        let attempt = ExecutionAttempt { task_id: "task".into(), step_id: "step".into(), attempt_id: "attempt".into(), worker_instance_id: "worker".into(), host_session_id: "host".into(), phase: AttemptPhase::Prepared, accepted_sequence: 2 };
        let response=r#"{"task_id":"task","step_id":"step","attempt_id":"attempt","worker_instance_id":"worker","host_session_id":"host","action_known":true,"action_succeeded":true,"action_effect":"confirmed","observe_valid":true,"element_count":2,"observation_ref":"observation_attempt","transcript":"[7] 按钮 切换企业","elements":[{"index":7,"role":"button","label":"按钮 切换企业"}]}"#;
        assert_eq!(classify(&attempt,response.as_bytes(),&evidence),DispatchOutcome::Known{action_succeeded:true,observation:Some(ComputerObservation{
            element_count:2,screenshot_path:None,screenshot_mime:None,target_visible:None,
            observation_ref:Some("observation_attempt".into()),transcript:Some("[7] 按钮 切换企业".into()),
            elements:vec![ObservedElement{index:7,role:yonder_application::ObservedElementRole::Button,label:Some("按钮 切换企业".into())}],
        })});
        let missing_ref=r#"{"task_id":"task","step_id":"step","attempt_id":"attempt","worker_instance_id":"worker","host_session_id":"host","action_known":true,"action_succeeded":true,"action_effect":"confirmed","observe_valid":true,"element_count":1,"transcript":"[7] 按钮 切换企业","elements":[{"index":7,"role":"button"}]}"#;
        assert_eq!(classify(&attempt,missing_ref.as_bytes(),&evidence),DispatchOutcome::Unknown(UnknownReason::InvalidResponse));
    }

    #[test]
    fn sky_app_identity_is_bounded_and_not_a_path() {
        assert!(valid_sky_app_id("com.tencent.WeWorkMac"));
        assert!(!valid_sky_app_id("/Applications/WeCom.app"));
        assert!(!valid_sky_app_id(""));
        assert!(!valid_sky_app_id(&"a".repeat(256)));
    }

    #[test]
    fn activation_uses_only_same_task_cached_driver_identity() {
        let port=RecordingActivator(Mutex::new(vec![]));
        let launched=TargetBinding{task_id:"task-a".into(),application_id:"com.tencent.WeWorkMac".into()};
        assert_eq!(activate_cached_window(&port,Some(&launched),"task-a"),Ok(()));
        assert_eq!(*port.0.lock().unwrap(),vec!["com.tencent.WeWorkMac"]);
        assert_eq!(activate_cached_window(&port,Some(&launched),"task-b"),Err(UnknownReason::InvalidInput));
        assert_eq!(activate_cached_window(&port,None,"task-a"),Err(UnknownReason::InvalidInput));
    }

    #[test]
    fn external_binding_outlives_worker_process_but_not_task_session() {
        let binding=TargetBinding{task_id:"task-a".into(),application_id:"com.example.App".into()};
        let mut runtime=WorkerRuntime{process:None,binding:Some(binding.clone())};
        CuaWorker::stop(&mut runtime.process);
        assert_eq!(runtime.binding,Some(binding));
        runtime.binding=None;
        assert_eq!(runtime.binding,None);
    }

    #[test]
    fn observed_handback_is_distinct_from_unobserved_worker_failure() {
        let observation=ComputerObservation{element_count:1,screenshot_path:None,screenshot_mime:None,target_visible:Some(true),observation_ref:None,transcript:None,elements:Vec::new()};
        assert!(matches!(DispatchOutcome::Unknown(UnknownReason::WorkerFailed),DispatchOutcome::Unknown(_)));
        assert!(!matches!(DispatchOutcome::UnknownObserved{reason:UnknownReason::ObserveFailed,observation},DispatchOutcome::Unknown(_)));
    }

    #[cfg(unix)]
    #[test]
    fn normal_actions_reuse_worker_session() {
        let root=std::env::temp_dir().join(format!("yonda-cua-session-{}",std::process::id()));
        let runtime=root.join("cua_node");
        let package=runtime.join("lib/node_modules/@oai/sky");
        std::fs::create_dir_all(package.join("dist")).unwrap();
        std::fs::create_dir_all(runtime.join("bin")).unwrap();
        let node=runtime.join("bin/node");std::fs::write(&node,b"#!/bin/sh\nexec /bin/sh \"$@\"\n").unwrap();
        use std::os::unix::fs::PermissionsExt;std::fs::set_permissions(&node,std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::write(package.join("package.json"),r#"{"name":"@oai/sky","version":"0.7.1"}"#).unwrap();
        let sdk=package.join("dist/index.js");std::fs::write(&sdk,b"").unwrap();
        let bridge=package.join("Codex Computer Use.app/Contents/SharedSupport/SkyComputerUseClient.app/Contents/MacOS");std::fs::create_dir_all(&bridge).unwrap();std::fs::write(bridge.join("SkyComputerUseClient"),b"").unwrap();
        let script=root.join("worker.sh");
        std::fs::write(&script,b"count=0\nwhile IFS= read -r line; do count=$((count+1)); if [ $count -eq 1 ]; then ok=true; else ok=false; fi; printf '{\"task_id\":\"task\",\"step_id\":\"step\",\"attempt_id\":\"attempt\",\"worker_instance_id\":\"worker\",\"host_session_id\":\"host\",\"action_known\":true,\"action_succeeded\":%s,\"observe_valid\":true}\\n' $ok; done\n").unwrap();
        let evidence=root.join("evidence");std::fs::create_dir(&evidence).unwrap();
        let worker=CuaWorker::new(&script,&sdk,&evidence,Duration::from_secs(2)).unwrap();
        let screenshot=evidence.join("task-attempt.png");std::fs::write(&screenshot,b"png").unwrap();
        worker.end_session();assert!(!screenshot.exists());
        let mut session=worker.start().unwrap();
        session.input.write_all(b"one\n").unwrap();session.input.flush().unwrap();
        assert!(String::from_utf8(session.output.recv_timeout(Duration::from_secs(2)).unwrap().unwrap()).unwrap().contains("\"action_succeeded\":true"));
        session.input.write_all(b"two\n").unwrap();session.input.flush().unwrap();
        assert!(String::from_utf8(session.output.recv_timeout(Duration::from_secs(2)).unwrap().unwrap()).unwrap().contains("\"action_succeeded\":false"));
        let mut slot=Some(session);CuaWorker::stop(&mut slot);drop(worker);std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn sky_constructor_requires_exact_package_identity() {
        let root=std::env::temp_dir().join(format!("yonda-sky-identity-{}",std::process::id()));
        let runtime=root.join("cua_node");
        let package=runtime.join("lib/node_modules/@oai/sky");
        std::fs::create_dir_all(package.join("dist/project/cua/sky_js/src")).unwrap();
        std::fs::create_dir_all(runtime.join("bin")).unwrap();std::fs::write(runtime.join("bin/node"),b"").unwrap();
        let sdk=package.join("dist/project/cua/sky_js/src/index.js");std::fs::write(&sdk,b"").unwrap();
        let bridge=package.join("Codex Computer Use.app/Contents/SharedSupport/SkyComputerUseClient.app/Contents/MacOS");std::fs::create_dir_all(&bridge).unwrap();std::fs::write(bridge.join("SkyComputerUseClient"),b"").unwrap();
        let script=root.join("worker.mjs");std::fs::write(&script,b"").unwrap();
        let evidence=root.join("evidence");std::fs::create_dir(&evidence).unwrap();
        std::fs::write(package.join("package.json"),r#"{"name":"@oai/sky","version":"0.7.1"}"#).unwrap();
        let worker=CuaWorker::new(&script,&sdk,&evidence,Duration::from_secs(2)).unwrap();drop(worker);
        std::fs::write(package.join("package.json"),r#"{"name":"@oai/sky","version":"0.8.0"}"#).unwrap();
        assert!(matches!(CuaWorker::new(&script,&sdk,&evidence,Duration::from_secs(2)),Err(UnknownReason::DependencyUnavailable)));
        std::fs::remove_dir_all(root).unwrap();
    }

}
