//! 正式桌面组合根核心；只供可信本机调用，不开放Agent或外部传输。
pub mod local_agent_stdio;
pub mod agent_input;
#[cfg(target_os = "macos")]
pub mod local_agent_socket;
use std::{collections::HashMap, fs::{File, OpenOptions}, path::Path, time::{Duration, Instant}};
use tauri::WebviewWindow;
use yonder_adapters::task_store::SqliteTaskStore;
use yonder_application::{AuthContext, ActivityState, ControlKind, TaskStore, admission::Admission, browser_use::BrowserUsePort, computer_use::{ComputerUsePort,WorkTarget,WorkTargetPort}, work_focus::{FocusFailure,WorkFocusPort,WorkRef,capture_after_observe,focus_takeover}};
#[cfg(target_os = "macos")]
use yonder_adapters::{cua::{CuaWorker,MacosFrontmostTarget},ego_lite::EgoLiteBridge,work_focus::MacWorkFocus};

struct FixedTarget(WorkTarget);
impl WorkTargetPort for FixedTarget { fn frontmost(&self)->Result<WorkTarget,yonder_application::computer_use::UnknownReason>{Ok(self.0.clone())} }

#[derive(Debug, PartialEq)]
pub enum HostError { InvalidDirectory, LockUnavailable, StorageUnavailable, BrowserUnavailable }

pub fn emit_pet_presentation(window: &WebviewWindow, has_tasks: bool, state: &str) {
    let state = match state {
        "idle" | "listening" | "thinking" | "executing" | "waiting_for_user" | "paused" | "recording" | "success" | "failed" | "unknown" => state,
        _ => "unknown",
    };
    let _ = window.eval(&format!("window.dispatchEvent(new CustomEvent('yonda-presentation',{{detail:{{hasTasks:{has_tasks},state:'{state}'}}}}))"));
}

pub fn emit_pet_terminal_presentation(window:&WebviewWindow,has_tasks:bool,state:&str,event_id:&str,resume_has_tasks:bool,resume_state:&str){
    if !matches!(state,"success"|"failed")||!matches!(resume_state,"idle"|"listening"|"thinking"|"executing"|"waiting_for_user"|"paused"|"recording"|"success"|"failed"|"unknown"){return}
    let detail=serde_json::json!({"hasTasks":has_tasks,"state":state,"eventId":event_id,"resumeHasTasks":resume_has_tasks,"resumeState":resume_state});
    let _=window.eval(&format!("window.dispatchEvent(new CustomEvent('yonda-presentation',{{detail:{detail}}}))"));
}

pub fn emit_pet_agent_connection(window: &WebviewWindow, connected: bool) {
    let _ = window.eval(&format!("window.dispatchEvent(new CustomEvent('yonda-agent-connection',{{detail:{{connected:{connected}}}}}))"));
}

pub struct TaskHost {
    store: SqliteTaskStore,
    admission: Admission,
    listening_pending: bool,
    listening_until: Option<Instant>,
    #[cfg(target_os = "macos")]
    browser: Option<EgoLiteBridge>,
    #[cfg(target_os = "macos")]
    computer: Option<CuaWorker>,
    #[cfg(target_os = "macos")]
    targets: MacosFrontmostTarget,
    #[cfg(target_os = "macos")]
    focus: MacWorkFocus,
    #[cfg(target_os = "macos")]
    work_refs: HashMap<String,WorkRef>,
    host_session_id: String,
    // 最后释放锁，确保任务库与准入先销毁；不删除文件，崩溃由OS释放锁。
    _lock: File,
}

impl TaskHost {
    /// 调用方必须传系统应用数据目录；不接收外部请求或UI提供的路径。
    pub fn open(directory: &Path) -> Result<Self, HostError> {
        if !directory.is_absolute() { return Err(HostError::InvalidDirectory); }
        std::fs::create_dir_all(directory).map_err(|_| HostError::StorageUnavailable)?;
        for name in ["host.lock", "tasks.db"] {
            match std::fs::symlink_metadata(directory.join(name)) {
                Ok(metadata) if !metadata.is_file() => return Err(HostError::InvalidDirectory),
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => return Err(HostError::StorageUnavailable),
                _ => {}
            }
        }
        let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false)
            .open(directory.join("host.lock")).map_err(|_| HostError::LockUnavailable)?;
        lock.try_lock().map_err(|_| HostError::LockUnavailable)?;
        let mut store = SqliteTaskStore::open_unencrypted(&directory.join("tasks.db"))
            .map_err(|_| HostError::StorageUnavailable)?;
        while yonder_application::recover_running(&mut store, 100)
            .map_err(|_| HostError::StorageUnavailable)? != 0 {}
        // 当前只读宿主尚无执行器；后续执行必须共用此实例。
        let admission = Admission::new(4).map_err(|_| HostError::StorageUnavailable)?;
        #[cfg(target_os = "macos")]
        let browser = std::env::var_os("HOME").and_then(|home| EgoLiteBridge::new(&Path::new(&home).join(".local/bin/ego-browser"), Duration::from_secs(30)).ok());
        #[cfg(target_os = "macos")]
        let computer={
            use std::os::unix::fs::PermissionsExt;
            let evidence=directory.join("observations");
            std::fs::create_dir_all(&evidence).map_err(|_|HostError::StorageUnavailable)?;
            std::fs::set_permissions(&evidence,std::fs::Permissions::from_mode(0o700)).map_err(|_|HostError::StorageUnavailable)?;
            std::env::current_exe().ok().and_then(|exe|{let root=exe.parent()?.parent()?.join("Resources/cua");CuaWorker::new(&root.join("node"),&root.join("cua_worker.mjs"),&root.join("node_modules/@trycua/cua-driver/dist/index.js"),&evidence,Duration::from_secs(30)).ok()})
        };
        Ok(Self { store, admission, listening_pending: false, listening_until: None, #[cfg(target_os = "macos")] browser, #[cfg(target_os = "macos")] computer, #[cfg(target_os = "macos")] targets:MacosFrontmostTarget, #[cfg(target_os = "macos")] focus:MacWorkFocus::new(), #[cfg(target_os = "macos")] work_refs:HashMap::new(), host_session_id: format!("host_{}",std::process::id()), _lock: lock })
    }

    pub fn desktop_control_active(&self) -> Result<bool, HostError> {
        self.admission.has_resource(yonder_application::admission::Resource::Desktop)
            .map_err(|_| HostError::StorageUnavailable)
    }

    /// 未来GUI命令还须校验本地task-space窗口；不能对Agent暴露本机权限。
    pub fn query(&mut self, request: &[u8], now_ms: u64) -> Result<Vec<u8>, HostError> {
        let response=yonder_application::query::handle_encoded_current(&mut self.store, AuthContext::LocalUser("desktop"), request, now_ms).map_err(|_| HostError::StorageUnavailable)?;
        #[cfg(target_os="macos")]
        if let Some((task_id,kind))=yonder_application::gateway::local_control_request(request){
            if yonder_application::gateway::response_is_stopped_control(&response){
                    if self.admission.holds(&task_id).map_err(|_|HostError::StorageUnavailable)?{self.admission.release_task_after_stop(&task_id).map_err(|_|HostError::StorageUnavailable)?;}
                    if kind==ControlKind::Takeover{
                        self.admission.set_desktop_taken_over(true).map_err(|_|HostError::StorageUnavailable)?;
                        if let Some(reference)=self.work_refs.get(&task_id).cloned(){focus_takeover(&mut self.store,&mut self.focus,&reference).map_err(|_|HostError::StorageUnavailable)?;}
                        else{
                            let current=self.store.get_control(&task_id).map_err(|_|HostError::StorageUnavailable)?.ok_or(HostError::StorageUnavailable)?;
                            self.store.begin_focus(&task_id,&current.control_id).map_err(|_|HostError::StorageUnavailable)?;
                            self.store.finish_focus(&task_id,&current.control_id,Some(FocusFailure::ReferenceUnavailable)).map_err(|_|HostError::StorageUnavailable)?;
                        }
                    }
                    return yonder_application::query::handle_encoded_current(&mut self.store,AuthContext::LocalUser("desktop"),request,now_ms).map_err(|_|HostError::StorageUnavailable);
            }
        }
        Ok(response)
    }

    /// 仅供打包Task Space专用命令；调用路径本身是显式本地用户意图。
    pub fn user_takeover(&mut self,task_id:&str,expected:u64,now_ms:u64)->Result<Vec<u8>,HostError>{
        let request=yonder_application::gateway::local_takeover_request(task_id,expected,now_ms).map_err(|_|HostError::StorageUnavailable)?;
        self.query(&request,now_ms)
    }

    /// Task Space中的显式用户操作；引用、序号和所有权均从可信存储复核。
    pub fn user_browser_handoff(&mut self,task_id:&str,expected:u64)->Result<(),HostError>{
        if !yonder_application::valid_id(task_id){return Err(HostError::BrowserUnavailable)}
        let reference=self.store.get_browser_reference(task_id).map_err(|_|HostError::StorageUnavailable)?.ok_or(HostError::BrowserUnavailable)?;
        if reference.finished||reference.ownership!="agent"{return Err(HostError::BrowserUnavailable)}
        #[cfg(target_os="macos")]
        {
            let browser=self.browser.as_ref().ok_or(HostError::BrowserUnavailable)?;
            yonder_application::browser_use::execute_agent_action(&mut self.store,&self.admission,browser,AuthContext::LocalUser("desktop"),task_id,expected,"hand-off",&self.host_session_id).map_err(|_|HostError::BrowserUnavailable)?;
            Ok(())
        }
        #[cfg(not(target_os="macos"))]
        { let _=expected; Err(HostError::BrowserUnavailable) }
    }

    /// 仅供已认证组合根提供会话；不是GUI或外部Agent传输入口。
    pub fn query_session(&mut self, session: &mut yonder_application::gateway::GatewaySession<'_>, request: &[u8], now_ms: u64) -> Result<Vec<u8>, HostError> {
        #[cfg(target_os = "macos")]
        let browser = self.browser.as_ref().map(|port|port as &dyn BrowserUsePort);
        #[cfg(target_os = "macos")]
        let computer=self.computer.as_ref().map(|port|port as &dyn ComputerUsePort);
        #[cfg(target_os = "macos")]
        let computer_task=yonder_application::gateway::computer_request_task(request);
        #[cfg(target_os = "macos")]
        let captured_target=if computer_task.is_some(){self.targets.frontmost().ok()}else{None};
        #[cfg(target_os = "macos")]
        let fixed_target=captured_target.clone().map(FixedTarget);
        #[cfg(target_os = "macos")]
        let targets=fixed_target.as_ref().map(|value|value as &dyn WorkTargetPort).or(Some(&self.targets as &dyn WorkTargetPort));
        #[cfg(target_os = "macos")]
        let computer_permission_required=!self.targets.available();
        #[cfg(not(target_os = "macos"))]
        let browser: Option<&dyn BrowserUsePort> = None;
        #[cfg(not(target_os = "macos"))]
        let computer:Option<&dyn ComputerUsePort>=None;
        #[cfg(not(target_os = "macos"))]
        let targets:Option<&dyn WorkTargetPort>=None;
        #[cfg(not(target_os = "macos"))]
        let computer_permission_required=false;
        let (response, accepted_create) = session.handle_encoded_with_runtimes(&mut self.store, &self.admission, browser, computer, targets, computer_permission_required, &self.host_session_id, request, now_ms)
            .map_err(|_| HostError::StorageUnavailable)?;
        #[cfg(target_os="macos")]
        if let Some(target)=captured_target{
            if yonder_application::gateway::response_is_computer_success(&response){
                if let Some(attempt)=self.store.get_attempt(computer_task.as_deref().unwrap_or("")).map_err(|_|HostError::StorageUnavailable)?{
                    if let Some(previous)=self.work_refs.remove(&attempt.task_id){self.focus.release(&previous);}
                    if let Ok(reference)=capture_after_observe(&mut self.focus,&attempt,&target){self.work_refs.insert(attempt.task_id.clone(),reference);}
                }
            }
        }
        if accepted_create { self.listening_pending = true; self.listening_until = None; }
        Ok(response)
    }

    /// 本地桌宠派生展示；执行优先，否则以首个未结束任务为代表。非执行/隐藏许可。
    pub fn presentation(&mut self) -> Result<(bool, &'static str), HostError> {
        let tasks = self.store.list(None, None, false, 1).map_err(|_| HostError::StorageUnavailable)?;
        let state = match self.activity() {
            ActivityState::Busy => { self.listening_pending = false; self.listening_until = None; "executing" },
            ActivityState::Unknown => return Err(HostError::StorageUnavailable),
            ActivityState::NoKnownWork => match tasks.first().map(|task| task.status) {
                _ if self.listening_pending => {
                    self.listening_pending = false;
                    self.listening_until = Some(Instant::now() + Duration::from_millis(1600));
                    "listening"
                },
                _ if self.listening_until.is_some_and(|until| Instant::now() < until) => "listening",
                Some(yonder_application::Status::WaitingForUser) => "waiting_for_user",
                Some(yonder_application::Status::Paused | yonder_application::Status::Interrupted) => "paused",
                _ => "idle",
            },
        };
        Ok((!tasks.is_empty(), state))
    }

    /// 是观察而非隐藏许可；正式收起仍需预约协调。
    pub fn activity(&mut self) -> ActivityState {
        yonder_application::activity_state(&mut self.store, Some(&self.admission))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yonder_application::{Action, create, transition};

    #[test]
    fn lock_child_probe() {
        if let Some(directory) = std::env::var_os("YONDA_TEST_HOST_DIRECTORY") {
            assert_eq!(TaskHost::open(Path::new(&directory)).err(), Some(HostError::LockUnavailable));
        }
    }

    #[test]
    fn failed_start_preserves_database_and_releases_host_lock() {
        let directory = std::env::temp_dir().join(format!("yonda-host-failed-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let database = directory.join("tasks.db");
        std::fs::write(&database, b"invalid database").unwrap();
        assert_eq!(TaskHost::open(&directory).err(), Some(HostError::StorageUnavailable));
        assert_eq!(std::fs::read(&database).unwrap(), b"invalid database");
        // 仅测试清理本次生成的非法样本；产品不会删除或回退旧库。
        std::fs::remove_file(&database).unwrap();
        drop(TaskHost::open(&directory).unwrap());
        for name in ["tasks.db", "host.lock"] { std::fs::remove_file(directory.join(name)).unwrap(); }
        std::fs::remove_dir(directory.join("observations")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn host_locks_recovers_and_queries_real_tasks_without_trusting_request_identity() {
        assert_eq!(TaskHost::open(Path::new("relative")).err(), Some(HostError::InvalidDirectory));
        let directory = std::env::temp_dir().join(format!("yonda-host-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let mut original = SqliteTaskStore::open_unencrypted(&directory.join("tasks.db")).unwrap();
        assert_eq!(create(&mut original, "manual", AuthContext::LocalUser("desktop")), Err(yonder_application::Error::PermissionDenied));
        assert_eq!(yonder_application::get(&mut original, "manual"), Err(yonder_application::Error::NotFound));
        for (id, agent) in [("task-a", "agent-a"), ("task-b", "agent-b")] {
            create(&mut original, id, AuthContext::Agent(agent)).unwrap();
            transition(&mut original, id, 1, Action::Start).unwrap();
        }
        drop(original);
        let mut host = TaskHost::open(&directory).unwrap();
        assert_eq!(TaskHost::open(&directory).err(), Some(HostError::LockUnavailable));
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "tests::lock_child_probe"])
            .env("YONDA_TEST_HOST_DIRECTORY", &directory).output().unwrap();
        assert!(child.status.success(), "跨进程锁验证失败：{:?}", child);
        let request = br#"{"jsonrpc":"2.0","id":"r1","method":"task.list","params":{"agent_id":"desktop","capability":"task.read","deadline":2000,"limit":100}}"#;
        let response = String::from_utf8(host.query(request, 1000).unwrap()).unwrap();
        for field in ["task-a", "task-b", "agent-a", "agent-b"] { assert!(response.contains(field)); }
        assert_eq!(response.matches("interrupted").count(), 2);
        assert_eq!(response.matches("\"sequence\":\"3\"").count(), 2);
        let timeline = host.store.register("agent-a", "timeline", "时间线", Some("验证")).unwrap();
        host.store.declare_step("agent-a", &timeline.id, timeline.sequence, "inspect", "检查时间线").unwrap();
        let events = format!(r#"{{"jsonrpc":"2.0","id":"e","method":"task.events","params":{{"agent_id":"desktop","capability":"task.read","deadline":2000,"task_id":"{}","after_sequence":"0","limit":100}}}}"#, timeline.id);
        let events = String::from_utf8(host.query(events.as_bytes(), 1000).unwrap()).unwrap();
        assert!(events.contains("step_declaration") && events.contains("检查时间线"));
        transition(&mut host.store, &timeline.id, 2, Action::Cancel).unwrap();
        use yonder_application::gateway::{GatewaySession, Platform};
        for (agent, own, other) in [("agent-a", "task-a", "task-b"), ("agent-b", "task-b", "task-a")] {
            let mut session = GatewaySession::new(AuthContext::Agent(agent), Platform::Macos);
            let query = String::from_utf8(request.to_vec()).unwrap().replace("desktop", agent);
            let read = |host: &mut TaskHost, session: &mut GatewaySession<'_>, bytes: &[u8]| String::from_utf8(host.query_session(session, bytes, 1000).unwrap()).unwrap();
            assert!(read(&mut host, &mut session, query.as_bytes()).contains("-32002"));
            let hello = format!(r#"{{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{{"agent_id":"{agent}","capability":"task.read","deadline":2000,"protocol_version":{{"major":1,"minor":0}}}}}}"#);
            assert!(read(&mut host, &mut session, hello.as_bytes()).contains("hello"));
            let response = read(&mut host, &mut session, query.as_bytes());
            assert!(response.contains(own)); assert!(!response.contains(other));
            let get = format!(r#"{{"jsonrpc":"2.0","id":"g","method":"task.get","params":{{"agent_id":"{agent}","capability":"task.read","deadline":2000,"task_id":"{other}"}}}}"#);
            assert!(read(&mut host, &mut session, get.as_bytes()).contains("-32004"));
            let mut fresh = GatewaySession::new(AuthContext::Agent(agent), Platform::Macos);
            assert!(read(&mut host, &mut fresh, query.as_bytes()).contains("-32002"));
        }
        assert_eq!(host.activity(), ActivityState::NoKnownWork);
        assert_eq!(host.presentation().unwrap(), (true, "paused"));
        transition(&mut host.store, "task-a", 3, Action::Resume).unwrap();
        assert_eq!(host.presentation().unwrap(), (true, "executing"));
        transition(&mut host.store, "task-a", 4, Action::WaitForUser).unwrap();
        assert_eq!(host.presentation().unwrap(), (true, "waiting_for_user"));
        transition(&mut host.store, "task-a", 5, Action::Resume).unwrap();
        transition(&mut host.store, "task-a", 6, Action::Pause).unwrap();
        assert_eq!(host.presentation().unwrap(), (true, "paused"));
        transition(&mut host.store, "task-a", 7, Action::Resume).unwrap();
        transition(&mut host.store, "task-a", 8, Action::Complete).unwrap();
        transition(&mut host.store, "task-b", 3, Action::Cancel).unwrap();
        assert_eq!(host.presentation().unwrap(), (false, "idle"));

        let forged = String::from_utf8(request.to_vec()).unwrap().replace("desktop", "agent-a");
        let denied = String::from_utf8(host.query(forged.as_bytes(), 1000).unwrap()).unwrap();
        assert!(denied.contains("-32003"));
        assert!(!denied.contains("task-a"));
        drop(host);
        let mut reopened = TaskHost::open(&directory).unwrap();
        assert_eq!(reopened.presentation().unwrap(), (false, "idle"));
        let mut session = GatewaySession::new(AuthContext::Agent("agent-c"), Platform::Macos);
        let create = br#"{"jsonrpc":"2.0","id":"c0","method":"task.create","params":{"agent_id":"agent-c","capability":"task.create","deadline":2000,"idempotency_key":"listen","description":"request","name":"request test"}}"#;
        assert!(String::from_utf8(reopened.query_session(&mut session, create, 1000).unwrap()).unwrap().contains("-32002"));
        assert_eq!(reopened.presentation().unwrap(), (false, "idle"));
        let hello = br#"{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{"agent_id":"agent-c","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":3}}}"#;
        reopened.query_session(&mut session, hello, 1000).unwrap();
        assert!(String::from_utf8(reopened.query_session(&mut session, create, 1000).unwrap()).unwrap().contains("created"));
        assert_eq!(reopened.presentation().unwrap(), (true, "listening"));
        std::thread::sleep(Duration::from_millis(1650));
        assert_ne!(reopened.presentation().unwrap().1, "listening");
        reopened.query_session(&mut session, create, 1000).unwrap();
        let created = reopened.store.list(None, None, false, 100).unwrap().into_iter().find(|task| task.owner_agent_id == "agent-c").unwrap();
        transition(&mut reopened.store, &created.id, created.sequence, Action::Start).unwrap();
        assert_eq!(reopened.presentation().unwrap(), (true, "executing"));
        drop(reopened);
        for name in ["tasks.db", "host.lock"] { std::fs::remove_file(directory.join(name)).unwrap(); }
        std::fs::remove_dir(directory.join("observations")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
