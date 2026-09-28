//! 正式桌面组合根核心；只供可信本机调用，不开放Agent或外部传输。
pub mod agent_input;
#[cfg(target_os = "macos")]
pub mod local_agent_socket;
pub mod local_agent_stdio;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{Manager, WebviewWindow};
use yonder_adapters::{command::StructuredCommandAdapter, document::OoxmlDocumentAdapter, file::ControlledFileAdapter, task_store::SqliteTaskStore};
#[cfg(target_os = "macos")]
use yonder_adapters::{
    cua::{CuaWorker, MacosFrontmostTarget},
    ego_lite::EgoLiteBridge,
    jev::MacosJevPort,
    work_focus::MacWorkFocus,
};
use yonder_application::{
    ActivityState, AuthContext, ControlKind, TaskStore,
    agent_registry::{AgentRegistration, AgentRegistrationStatus, AgentRegistry},
    admission::Admission,
    browser_use::BrowserUsePort,
    computer_use::{ComputerUsePort, WorkTarget, WorkTargetPort},
    command_approval::{CommandApprovalError, CommandApprovalPreview, CommandApprovalRegistry, CommandApprovalSummary},
    jev_config::JevConfig,
    jev_runtime::{JevDecision, JevDecisionRequest, JevDecisionError},
    work_focus::{FocusFailure, WorkFocusPort, WorkRef, capture_after_observe, focus_takeover},
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct CuaControlStepPresentation {
    pub step_id: String,
    pub label: String,
    pub state: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct CuaControlPresentation {
    pub task_id: String,
    pub current_step: String,
    pub planned_steps: Vec<CuaControlStepPresentation>,
    pub remaining_steps: u16,
    pub plan_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CuaControlState {
    task_id: String,
    takeover_requested: bool,
    presentation: CuaControlPresentation,
    executing_step_id: Option<String>,
}

/// CUA 执行热路径的进程内信号；不持久化，也不等待 TaskHost/SQLite 锁。
#[derive(Clone, Default)]
pub struct CuaControlHub(Arc<Mutex<Option<CuaControlState>>>);

impl CuaControlHub {
    pub fn begin(&self, presentation: CuaControlPresentation) -> bool {
        let Ok(mut state) = self.0.lock() else { return false };
        if let Some(active) = state.as_mut() {
            if active.task_id != presentation.task_id { return false; }
            active.presentation = presentation;
            active.executing_step_id = None;
            return true;
        }
        *state = Some(CuaControlState { task_id: presentation.task_id.clone(), takeover_requested: false, presentation, executing_step_id: None });
        true
    }

    pub fn mark_executing(&self, task_id: &str, step_id: &str) {
        let Ok(mut state) = self.0.lock() else { return };
        let Some(active) = state.as_mut().filter(|active| active.task_id == task_id) else { return };
        active.executing_step_id = Some(step_id.to_owned());
        if let Some(step) = active.presentation.planned_steps.iter().find(|step| step.step_id == step_id) {
            active.presentation.current_step = step.label.clone();
        }
        for step in &mut active.presentation.planned_steps {
            step.state = if step.step_id == step_id { "executing".into() } else { "pending".into() };
        }
        if let Some(index) = active.presentation.planned_steps.iter().position(|step| step.step_id == step_id) {
            for step in &mut active.presentation.planned_steps[..index] { step.state = "completed".into(); }
        }
    }

    pub fn presentation(&self) -> Option<CuaControlPresentation> {
        let state = self.0.lock().ok()?;
        let active = state.as_ref()?;
        let mut presentation = active.presentation.clone();
        if active.executing_step_id.is_none() && presentation.plan_status == "available" {
            presentation.current_step = format!("准备执行：{}", presentation.current_step);
        }
        Some(presentation)
    }

    pub fn request_takeover(&self, task_id: &str) -> bool {
        let Ok(mut state) = self.0.lock() else { return false };
        let Some(active) = state.as_mut().filter(|active| active.task_id == task_id) else { return false };
        active.takeover_requested = true;
        true
    }

    pub fn takeover_requested(&self, task_id: &str) -> bool {
        self.0.lock().is_ok_and(|state| state.as_ref().is_some_and(|active| active.task_id == task_id && active.takeover_requested))
    }

    pub fn take_takeover_requested(&self, task_id: &str) -> bool {
        let Ok(mut state) = self.0.lock() else { return false };
        let Some(active) = state.as_mut().filter(|active| active.task_id == task_id) else { return false };
        std::mem::take(&mut active.takeover_requested)
    }

    pub fn finish(&self, task_id: &str) -> bool {
        let Ok(mut state) = self.0.lock() else { return false };
        if state.as_ref().is_none_or(|active| active.task_id != task_id) { return false; }
        state.take().is_some()
    }
}

#[cfg(target_os = "macos")]
struct CuaControlPort<'a> { inner: &'a CuaWorker, hub: Option<&'a CuaControlHub> }

#[cfg(target_os = "macos")]
impl ComputerUsePort for CuaControlPort<'_> {
    fn dispatch(&self, attempt:&yonder_application::ExecutionAttempt,target:&WorkTarget,action:&yonder_application::computer_use::ComputerAction)->yonder_application::computer_use::DispatchOutcome {
        if let Some(hub)=self.hub { hub.mark_executing(&attempt.task_id,&attempt.step_id); }
        self.inner.dispatch(attempt,target,action)
    }
    fn end_session(&self){self.inner.end_session()}
    fn explicit_takeover_requested(&self,task_id:&str)->bool{self.hub.is_some_and(|hub|hub.takeover_requested(task_id))}
}

/// 与圈选工具条共用 pet 当前显示器、work area 与顶部 16pt 锚点。
pub fn position_window_in_pet_work_area(
    pet: &WebviewWindow,
    window: &WebviewWindow,
    logical_width: f64,
    logical_height: f64,
) -> Result<(), String> {
    let monitor = pet.current_monitor().map_err(|_| "屏幕不可用")?
        .or(pet.primary_monitor().map_err(|_| "屏幕不可用")?).ok_or("屏幕不可用")?;
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let width = (logical_width * scale).round() as u32;
    let height = (logical_height * scale).round() as u32;
    let x = area.position.x + (area.size.width.saturating_sub(width) / 2) as i32;
    let y = area.position.y + (16.0 * scale).round() as i32;
    window.set_size(tauri::PhysicalSize::new(width, height)).map_err(|_| "控制卡尺寸设置失败")?;
    window.set_position(tauri::PhysicalPosition::new(x, y)).map_err(|_| "控制卡定位失败".into())
}

fn show_task_space_near_pet_inner(pet: &WebviewWindow, focus: bool) -> Result<(), String> {
    let app = pet.app_handle();
    let menu = app.get_webview_window("task-space").ok_or("任务菜单不可用")?;
    let monitor = pet.current_monitor().map_err(|_| "屏幕不可用")?
        .or(pet.primary_monitor().map_err(|_| "屏幕不可用")?).ok_or("屏幕不可用")?;
    let area = monitor.work_area();
    let position = pet.outer_position().map_err(|_| "位置不可用")?;
    let pet_size = pet.outer_size().map_err(|_| "尺寸不可用")?;
    let size = menu.outer_size().map_err(|_| "菜单尺寸不可用")?;
    let gap = (8.0 * monitor.scale_factor()) as i32;
    let below = position.y + pet_size.height as i32 + gap;
    let y = if i64::from(below) + i64::from(size.height) <= i64::from(area.position.y) + i64::from(area.size.height) {
        below
    } else { position.y - size.height as i32 - gap };
    let max_x = area.position.x.saturating_add(area.size.width.saturating_sub(size.width) as i32);
    let max_y = area.position.y.saturating_add(area.size.height.saturating_sub(size.height) as i32);
    let target = tauri::PhysicalPosition::new(position.x.clamp(area.position.x, max_x), y.clamp(area.position.y, max_y));
    menu.set_position(target).and_then(|_| menu.show()).map_err(|_| "任务菜单打开失败")?;
    if focus { menu.set_focus().map_err(|_| "任务菜单打开失败")?; }
    let event = if focus {
        "window.dispatchEvent(new Event('yonda-tasks-open'))"
    } else {
        "window.dispatchEvent(new CustomEvent('yonda-tasks-open',{detail:{automatic:true}}))"
    };
    menu.eval(event).map_err(|_| "任务菜单打开失败".into())
}

/// 用户显式打开时保持既有焦点行为。
pub fn show_task_space_near_pet(pet: &WebviewWindow) -> Result<(), String> {
    show_task_space_near_pet_inner(pet, true)
}

/// 自动展示不应覆盖用户正使用的输入或桌面控制卡。
pub fn show_task_space_after_agent_create(pet: &WebviewWindow) -> Result<bool, String> {
    let app = pet.app_handle();
    for label in ["cua-control", "voice-input", "region-preview"] {
        if app.get_webview_window(label).is_some_and(|window| window.is_visible().unwrap_or(false)) {
            return Ok(false);
        }
    }
    // 自动透出不抢走用户当前焦点，否则 task-space 的 blur 收起规则会立刻关闭它。
    show_task_space_near_pet_inner(pet, false)?;
    Ok(true)
}

#[cfg(target_os = "macos")]
fn macos_cua_resource_dir() -> Option<PathBuf> {
    if cfg!(debug_assertions) {
        if let Some(value) = std::env::var_os("YONDER_TEST_CUA_RESOURCE_DIR") {
            let path = PathBuf::from(value);
            let metadata = std::fs::symlink_metadata(&path).ok()?;
            if path.is_absolute() && metadata.is_dir() && !metadata.file_type().is_symlink() {
                return Some(path);
            }
            return None;
        }
    }
    let executable = std::env::current_exe().ok()?;
    Some(executable.parent()?.parent()?.join("Resources/cua"))
}
use yonder_application::{
    file::{FileCreateTargetRequest, FileReadRequest},
    file_authorization::{FileAuthorizationRegistry, FileGrantError, FileGrantPurpose, FileGrantSummary},
};
use uuid::Uuid;

struct FixedTarget(WorkTarget);
impl WorkTargetPort for FixedTarget {
    fn frontmost(&self) -> Result<WorkTarget, yonder_application::computer_use::UnknownReason> {
        Ok(self.0.clone())
    }
}

#[derive(Debug, PartialEq)]
pub enum HostError {
    InvalidDirectory,
    LockUnavailable,
    StorageUnavailable,
    BrowserUnavailable,
    InvalidConfig,
}

#[derive(Debug, PartialEq)]
pub enum ConfirmError {
    InvalidInput,
    PermissionDenied,
    NotFound,
    Conflict,
    QuotaExceeded,
    StorageUnavailable,
}

#[derive(Debug, PartialEq)]
pub enum CommandApprovalHostError { InvalidInput, NotFound, PermissionDenied, Expired, Rejected, Capacity, StorageUnavailable }

impl From<CommandApprovalError> for CommandApprovalHostError {
    fn from(value: CommandApprovalError) -> Self { match value {
        CommandApprovalError::InvalidInput => Self::InvalidInput, CommandApprovalError::NotFound => Self::NotFound,
        CommandApprovalError::PermissionDenied => Self::PermissionDenied, CommandApprovalError::Expired => Self::Expired,
        CommandApprovalError::Rejected => Self::Rejected, CommandApprovalError::Capacity => Self::Capacity,
        CommandApprovalError::Unavailable => Self::StorageUnavailable,
    }}
}

#[derive(Debug, PartialEq)]
pub enum FileGrantHostError {
    InvalidInput,
    NotFound,
    TaskUnavailable,
    TargetUnavailable,
    TargetExists,
    TargetLocked,
    Capacity,
    PermissionDenied,
}

impl FileGrantHostError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::InvalidInput => "文件授权参数无效",
            Self::NotFound => "任务不存在",
            Self::TaskUnavailable => "任务当前不能授予文件权限",
            Self::TargetUnavailable => "无法读取所选文件或目录",
            Self::TargetExists => "新建目标已存在，请选择新的文件名",
            Self::TargetLocked => "所选文件正被其他应用使用",
            Self::Capacity => "文件授权数量已满，请先撤销不再需要的授权",
            Self::PermissionDenied => "当前操作没有文件授权权限",
        }
    }
}

fn file_grant_host_error(error: FileGrantError) -> FileGrantHostError {
    match error {
        FileGrantError::InvalidInput => FileGrantHostError::InvalidInput,
        FileGrantError::PermissionDenied => FileGrantHostError::PermissionDenied,
        FileGrantError::NotFound => FileGrantHostError::NotFound,
        FileGrantError::Capacity => FileGrantHostError::Capacity,
        FileGrantError::File(yonder_application::file::FileError::AlreadyExists) => {
            FileGrantHostError::TargetExists
        }
        FileGrantError::File(yonder_application::file::FileError::HostLocked) => {
            FileGrantHostError::TargetLocked
        }
        FileGrantError::File(_) | FileGrantError::Expired | FileGrantError::Unavailable => {
            FileGrantHostError::TargetUnavailable
        }
    }
}

impl ConfirmError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::InvalidInput => "确认参数无效",
            Self::PermissionDenied => "仅本机用户可确认结果",
            Self::NotFound => "任务不存在",
            Self::Conflict => "结果已更新或已有不同确认，请刷新后重试",
            Self::QuotaExceeded => "审计容量不足，无法新增确认",
            Self::StorageUnavailable => "任务存储不可用",
        }
    }
}

pub fn emit_pet_presentation(
    window: &WebviewWindow,
    has_tasks: bool,
    state: &str,
    step_label: Option<&str>,
) {
    let state = match state {
        "idle" | "listening" | "thinking" | "executing" | "waiting_for_user" | "paused"
        | "recording" | "success" | "failed" | "unknown" => state,
        _ => "unknown",
    };
    let detail = serde_json::json!({
        "hasTasks": has_tasks,
        "state": state,
        "stepLabel": if state == "executing" { step_label } else { None },
    });
    let _ = window.eval(&format!(
        "window.dispatchEvent(new CustomEvent('yonda-presentation',{{detail:{detail}}}))"
    ));
}

pub fn emit_pet_terminal_presentation(
    window: &WebviewWindow,
    has_tasks: bool,
    state: &str,
    event_id: &str,
    resume_has_tasks: bool,
    resume_state: &str,
    resume_step_label: Option<&str>,
) {
    if !matches!(state, "success" | "failed")
        || !matches!(
            resume_state,
            "idle"
                | "listening"
                | "thinking"
                | "executing"
                | "waiting_for_user"
                | "paused"
                | "recording"
                | "success"
                | "failed"
                | "unknown"
        )
    {
        return;
    }
    let detail = serde_json::json!({"hasTasks":has_tasks,"state":state,"stepLabel":null,"eventId":event_id,"resumeHasTasks":resume_has_tasks,"resumeState":resume_state,"resumeStepLabel":if resume_state=="executing"{resume_step_label}else{None}});
    let _ = window.eval(&format!(
        "window.dispatchEvent(new CustomEvent('yonda-presentation',{{detail:{detail}}}))"
    ));
}

pub fn emit_pet_agent_connection(window: &WebviewWindow, connected: bool) {
    let _ = window.eval(&format!("window.dispatchEvent(new CustomEvent('yonda-agent-connection',{{detail:{{connected:{connected}}}}}))"));
}

pub struct TaskHost {
    store: SqliteTaskStore,
    admission: Admission,
    files: ControlledFileAdapter,
    documents: OoxmlDocumentAdapter,
    commands: StructuredCommandAdapter,
    file_grants: FileAuthorizationRegistry,
    command_approvals: CommandApprovalRegistry,
    listening_pending: bool,
    listening_until: Option<Instant>,
    task_space_open_pending: bool,
    #[cfg(target_os = "macos")]
    browser: Option<EgoLiteBridge>,
    #[cfg(target_os = "macos")]
    computer: Option<CuaWorker>,
    #[cfg(target_os = "macos")]
    cua_control: Option<CuaControlHub>,
    #[cfg(target_os = "macos")]
    jev: Option<MacosJevPort>,
    #[cfg(target_os = "macos")]
    targets: MacosFrontmostTarget,
    #[cfg(target_os = "macos")]
    focus: MacWorkFocus,
    #[cfg(target_os = "macos")]
    work_refs: HashMap<String, WorkRef>,
    host_session_id: String,
    // 最后释放锁，确保任务库与准入先销毁；不删除文件，崩溃由OS释放锁。
    _lock: File,
}

impl TaskHost {
    /// 调用方必须传系统应用数据目录；不接收外部请求或UI提供的路径。
    pub fn open(directory: &Path) -> Result<Self, HostError> {
        if !directory.is_absolute() {
            return Err(HostError::InvalidDirectory);
        }
        std::fs::create_dir_all(directory).map_err(|_| HostError::StorageUnavailable)?;
        for name in ["host.lock", "tasks.db"] {
            match std::fs::symlink_metadata(directory.join(name)) {
                Ok(metadata) if !metadata.is_file() => return Err(HostError::InvalidDirectory),
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                    return Err(HostError::StorageUnavailable);
                }
                _ => {}
            }
        }
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(directory.join("host.lock"))
            .map_err(|_| HostError::LockUnavailable)?;
        lock.try_lock().map_err(|_| HostError::LockUnavailable)?;
        let mut store = SqliteTaskStore::open_unencrypted(&directory.join("tasks.db"))
            .map_err(|_| HostError::StorageUnavailable)?;
        while yonder_application::recover_running(&mut store, 100)
            .map_err(|_| HostError::StorageUnavailable)?
            != 0
        {}
        // 当前只读宿主尚无执行器；后续执行必须共用此实例。
        let admission = Admission::new(4).map_err(|_| HostError::StorageUnavailable)?;
        #[cfg(target_os = "macos")]
        let browser = std::env::var_os("HOME").and_then(|home| {
            EgoLiteBridge::new(
                &Path::new(&home).join(".local/bin/ego-browser"),
                Duration::from_secs(30),
            )
            .ok()
        });
        #[cfg(target_os = "macos")]
        let computer = {
            use std::os::unix::fs::PermissionsExt;
            let evidence = directory.join("observations");
            std::fs::create_dir_all(&evidence).map_err(|_| HostError::StorageUnavailable)?;
            std::fs::set_permissions(&evidence, std::fs::Permissions::from_mode(0o700))
                .map_err(|_| HostError::StorageUnavailable)?;
            macos_cua_resource_dir().and_then(|root| {
                CuaWorker::new(
                    &root.join("node"),
                    &root.join("cua_worker.mjs"),
                    &root.join("node_modules/@trycua/cua-driver/dist/index.js"),
                    &evidence,
                    Duration::from_secs(30),
                )
                .ok()
            })
        };
        #[cfg(target_os = "macos")]
        let jev = macos_cua_resource_dir().and_then(|root| {
            MacosJevPort::new(
                &root.join("node"),
                &root.join("jev_worker.mjs"),
                &root.join("node_modules/@typesafe-ai/sdk/dist/index.mjs"),
                "Yonder",
                "jev",
                Duration::from_secs(30),
            )
            .ok()
        });
        Ok(Self {
            store,
            admission,
            files: ControlledFileAdapter::default(),
            documents: OoxmlDocumentAdapter,
            commands: StructuredCommandAdapter,
            file_grants: FileAuthorizationRegistry::default(),
            command_approvals: CommandApprovalRegistry::default(),
            listening_pending: false,
            listening_until: None,
            task_space_open_pending: false,
            #[cfg(target_os = "macos")]
            browser,
            #[cfg(target_os = "macos")]
            computer,
            #[cfg(target_os = "macos")]
            cua_control: None,
            #[cfg(target_os = "macos")]
            jev,
            #[cfg(target_os = "macos")]
            targets: MacosFrontmostTarget,
            #[cfg(target_os = "macos")]
            focus: MacWorkFocus::new(),
            #[cfg(target_os = "macos")]
            work_refs: HashMap::new(),
            host_session_id: format!("host_{}", std::process::id()),
            _lock: lock,
        })
    }

    pub fn desktop_control_active(&self) -> Result<bool, HostError> {
        self.admission
            .has_resource(yonder_application::admission::Resource::Desktop)
            .map_err(|_| HostError::StorageUnavailable)
    }

    #[cfg(target_os = "macos")]
    pub fn set_cua_control_hub(&mut self, hub:CuaControlHub){self.cua_control=Some(hub)}

    pub fn register_agent(
        &mut self,
        agent_id: &str,
        now_ms: u64,
    ) -> Result<AgentRegistration, HostError> {
        AgentRegistry::register_agent(&mut self.store, agent_id, now_ms)
            .map_err(|_| HostError::StorageUnavailable)
    }

    /// 私有本机认证组合根只确保缺失身份存在；不能重新启用用户禁用/撤权的 Agent。
    pub fn ensure_local_agent(&mut self, agent_id: &str, now_ms: u64) -> Result<AgentRegistration, HostError> {
        AgentRegistry::ensure_agent(&mut self.store, agent_id, now_ms).map_err(|_| HostError::StorageUnavailable)
    }

    pub fn list_agents(&mut self) -> Result<Vec<AgentRegistration>, HostError> {
        AgentRegistry::list_agents(&mut self.store).map_err(|_| HostError::StorageUnavailable)
    }

    pub fn set_agent_status(
        &mut self,
        agent_id: &str,
        status: AgentRegistrationStatus,
        now_ms: u64,
    ) -> Result<AgentRegistration, HostError> {
        if status != AgentRegistrationStatus::Enabled {
            self.file_grants
                .revoke_owner(AuthContext::LocalUser("desktop"), agent_id)
                .map_err(|_| HostError::StorageUnavailable)?;
            self.command_approvals
                .revoke_owner(AuthContext::LocalUser("desktop"), agent_id)
                .map_err(|_| HostError::StorageUnavailable)?;
        }
        AgentRegistry::set_agent_status(&mut self.store, agent_id, status, now_ms)
            .map_err(|_| HostError::StorageUnavailable)
    }

    /// 原生选择器的结果只在此处转成受控引用，路径不会返回给 WebView 或 Agent。
    pub fn issue_file_grant(
        &mut self,
        task_id: &str,
        purpose: FileGrantPurpose,
        selected_path: &Path,
        now_ms: u64,
    ) -> Result<FileGrantSummary, FileGrantHostError> {
        if !yonder_application::valid_id(task_id) || !selected_path.is_absolute() {
            return Err(FileGrantHostError::InvalidInput);
        }
        let path = selected_path
            .to_str()
            .filter(|value| !value.is_empty())
            .ok_or(FileGrantHostError::InvalidInput)?;
        let authorized_root = selected_path
            .parent()
            .and_then(|parent| parent.to_str())
            .filter(|value| !value.is_empty())
            .ok_or(FileGrantHostError::InvalidInput)?;
        let task = self.store.get(task_id).map_err(|error| match error {
            yonder_application::Error::NotFound => FileGrantHostError::NotFound,
            _ => FileGrantHostError::TaskUnavailable,
        })?;
        if matches!(
            task.status,
            yonder_application::Status::Completed
                | yonder_application::Status::Failed
                | yonder_application::Status::Cancelled
        ) {
            return Err(FileGrantHostError::TaskUnavailable);
        }
        let grant_id = format!("file_grant_{}", Uuid::new_v4().simple());
        let expires_at_ms = now_ms
            .checked_add(yonder_application::file_authorization::MAX_FILE_GRANT_LIFETIME_MS)
            .ok_or(FileGrantHostError::InvalidInput)?;
        let grant = match purpose {
            FileGrantPurpose::CreateNew => self.file_grants.issue_create_target(
                &self.files,
                AuthContext::LocalUser("desktop"),
                &task,
                &grant_id,
                &FileCreateTargetRequest {
                    path: path.into(),
                    authorized_root: authorized_root.into(),
                },
                now_ms,
                expires_at_ms,
            ),
            FileGrantPurpose::Read | FileGrantPurpose::Replace | FileGrantPurpose::Trash => {
                self.file_grants.issue_existing(
                    &self.files,
                    AuthContext::LocalUser("desktop"),
                    &task,
                    &grant_id,
                    purpose,
                    &FileReadRequest {
                        path: path.into(),
                        authorized_root: authorized_root.into(),
                    },
                    now_ms,
                    expires_at_ms,
                )
            }
        }
        .map_err(file_grant_host_error)?;
        Ok(FileGrantSummary {
            grant_id: grant.grant_id,
            purpose: grant.purpose,
            expires_at_ms: grant.expires_at_ms,
        })
    }

    pub fn list_file_grants(
        &mut self,
        task_id: &str,
        now_ms: u64,
    ) -> Result<Vec<FileGrantSummary>, FileGrantHostError> {
        let task = self.store.get(task_id).map_err(|error| match error {
            yonder_application::Error::NotFound => FileGrantHostError::NotFound,
            _ => FileGrantHostError::TaskUnavailable,
        })?;
        self.file_grants
            .list_for_local(AuthContext::LocalUser("desktop"), &task, now_ms)
            .map_err(file_grant_host_error)
    }

    pub fn revoke_file_grant(
        &mut self,
        task_id: &str,
        grant_id: &str,
    ) -> Result<(), FileGrantHostError> {
        let task = self.store.get(task_id).map_err(|error| match error {
            yonder_application::Error::NotFound => FileGrantHostError::NotFound,
            _ => FileGrantHostError::TaskUnavailable,
        })?;
        self.file_grants
            .revoke(AuthContext::LocalUser("desktop"), &task, grant_id)
            .map_err(file_grant_host_error)
    }

    pub fn pause_desktop_for_user(&mut self) -> Result<bool, HostError> {
        let Some(task_id) = self.admission.task_holding(yonder_application::admission::Resource::Desktop).map_err(|_| HostError::StorageUnavailable)? else { return Ok(false); };
        let task = self.store.get(&task_id).map_err(|_| HostError::StorageUnavailable)?;
        let (_, control) = yonder_application::request_control(&mut self.store, AuthContext::LocalUser("desktop"), &task_id, task.sequence, ControlKind::Pause).map_err(|_| HostError::StorageUnavailable)?;
        if control.phase == yonder_application::ControlPhase::Pending {
            let attempt = self.store.get_attempt(&task_id).map_err(|_| HostError::StorageUnavailable)?.ok_or(HostError::StorageUnavailable)?;
            yonder_application::stop_at_boundary(&mut self.store, &task_id, &attempt.attempt_id, ControlKind::Pause).map_err(|_| HostError::StorageUnavailable)?;
        }
        self.admission.release_task_after_stop(&task_id).map_err(|_| HostError::StorageUnavailable)?;
        Ok(true)
    }

    /// 未来GUI命令还须校验本地task-space窗口；不能对Agent暴露本机权限。
    pub fn query(&mut self, request: &[u8], now_ms: u64) -> Result<Vec<u8>, HostError> {
        let response = yonder_application::query::handle_encoded_current(
            &mut self.store,
            AuthContext::LocalUser("desktop"),
            request,
            now_ms,
        )
        .map_err(|_| HostError::StorageUnavailable)?;
        #[cfg(target_os = "macos")]
        if let Some((task_id, kind)) = yonder_application::gateway::local_control_request(request) {
            if yonder_application::gateway::response_is_stopped_control(&response) {
                if self
                    .admission
                    .holds(&task_id)
                    .map_err(|_| HostError::StorageUnavailable)?
                {
                    self.admission
                        .release_task_after_stop(&task_id)
                        .map_err(|_| HostError::StorageUnavailable)?;
                }
                if kind == ControlKind::Takeover {
                    self.admission
                        .set_desktop_taken_over(true)
                        .map_err(|_| HostError::StorageUnavailable)?;
                    if let Some(reference) = self.work_refs.get(&task_id).cloned() {
                        focus_takeover(&mut self.store, &mut self.focus, &reference)
                            .map_err(|_| HostError::StorageUnavailable)?;
                    } else {
                        let current = self
                            .store
                            .get_control(&task_id)
                            .map_err(|_| HostError::StorageUnavailable)?
                            .ok_or(HostError::StorageUnavailable)?;
                        self.store
                            .begin_focus(&task_id, &current.control_id)
                            .map_err(|_| HostError::StorageUnavailable)?;
                        self.store
                            .finish_focus(
                                &task_id,
                                &current.control_id,
                                Some(FocusFailure::ReferenceUnavailable),
                            )
                            .map_err(|_| HostError::StorageUnavailable)?;
                    }
                }
                return yonder_application::query::handle_encoded_current(
                    &mut self.store,
                    AuthContext::LocalUser("desktop"),
                    request,
                    now_ms,
                )
                .map_err(|_| HostError::StorageUnavailable);
            }
        }
        Ok(response)
    }

    /// 仅供打包Task Space专用命令；调用路径本身是显式本地用户意图。
    pub fn user_takeover(
        &mut self,
        task_id: &str,
        expected: u64,
        now_ms: u64,
    ) -> Result<Vec<u8>, HostError> {
        let request =
            yonder_application::gateway::local_takeover_request(task_id, expected, now_ms)
                .map_err(|_| HostError::StorageUnavailable)?;
        self.query(&request, now_ms)
    }

    /// 控制卡已由宿主绑定当前 task；sequence 必须在持有事实源锁后重新读取。
    pub fn user_takeover_current(&mut self, task_id: &str, now_ms: u64) -> Result<Vec<u8>, HostError> {
        let sequence = self.store.get(task_id).map_err(|_| HostError::StorageUnavailable)?.sequence;
        self.user_takeover(task_id, sequence, now_ms)
    }

    /// Task Space 的显式本机用户确认入口；Agent 不能调用。
    pub fn confirm_result(
        &mut self,
        task_id: &str,
        expected_sequence: u64,
        confirmation_id: &str,
        comment: Option<&str>,
    ) -> Result<yonder_application::Task, ConfirmError> {
        yonder_application::confirm_result(
            &mut self.store,
            AuthContext::LocalUser("desktop"),
            task_id,
            expected_sequence,
            confirmation_id,
            comment,
        )
        .map_err(|error| match error {
            yonder_application::Error::InvalidInput => ConfirmError::InvalidInput,
            yonder_application::Error::PermissionDenied => ConfirmError::PermissionDenied,
            yonder_application::Error::NotFound => ConfirmError::NotFound,
            yonder_application::Error::Conflict => ConfirmError::Conflict,
            yonder_application::Error::QuotaExceeded => ConfirmError::QuotaExceeded,
            _ => ConfirmError::StorageUnavailable,
        })
    }

    /// 仅由打包 Task Space 的本机用户入口调用；预览完整命令不会经 Gateway 返回。
    pub fn list_command_approvals(&mut self, task_id: &str, now_ms: u64) -> Result<Vec<CommandApprovalSummary>, CommandApprovalHostError> {
        let task=self.store.get(task_id).map_err(|_|CommandApprovalHostError::NotFound)?;
        self.command_approvals.list_for_local(AuthContext::LocalUser("desktop"),&task,now_ms).map_err(Into::into)
    }

    /// 仅由打包 Task Space 的本机用户入口调用；预览完整命令不会经 Gateway 返回。
    pub fn preview_command_approval(&mut self, task_id: &str, command_id: &str, now_ms: u64) -> Result<CommandApprovalPreview, CommandApprovalHostError> {
        let task=self.store.get(task_id).map_err(|_|CommandApprovalHostError::NotFound)?;
        self.command_approvals.preview_for_local(AuthContext::LocalUser("desktop"),&task,command_id,now_ms).map_err(Into::into)
    }

    /// 仅由打包 Task Space 的显式“批准执行”按钮调用；Agent 没有等价入口。
    pub fn approve_command(&mut self, task_id: &str, command_id: &str, now_ms: u64) -> Result<CommandApprovalSummary, CommandApprovalHostError> {
        let task=self.store.get(task_id).map_err(|_|CommandApprovalHostError::NotFound)?;
        self.command_approvals.approve(AuthContext::LocalUser("desktop"),&task,command_id,now_ms).map_err(Into::into)
    }

    pub fn reject_command(&mut self, task_id: &str, command_id: &str, now_ms: u64) -> Result<(), CommandApprovalHostError> {
        let task=self.store.get(task_id).map_err(|_|CommandApprovalHostError::NotFound)?;
        self.command_approvals.reject(AuthContext::LocalUser("desktop"),&task,command_id,now_ms).map_err(Into::into)
    }

    /// Task Space中的显式用户操作；引用、序号和所有权均从可信存储复核。
    pub fn user_browser_handoff(&mut self, task_id: &str, expected: u64) -> Result<(), HostError> {
        if !yonder_application::valid_id(task_id) {
            return Err(HostError::BrowserUnavailable);
        }
        let reference = self
            .store
            .get_browser_reference(task_id)
            .map_err(|_| HostError::StorageUnavailable)?
            .ok_or(HostError::BrowserUnavailable)?;
        if reference.finished || reference.ownership != "agent" {
            return Err(HostError::BrowserUnavailable);
        }
        #[cfg(target_os = "macos")]
        {
            let browser = self.browser.as_ref().ok_or(HostError::BrowserUnavailable)?;
            yonder_application::browser_use::execute_agent_action(
                &mut self.store,
                &self.admission,
                browser,
                AuthContext::LocalUser("desktop"),
                task_id,
                expected,
                "hand-off",
                &self.host_session_id,
            )
            .map_err(|_| HostError::BrowserUnavailable)?;
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = expected;
            Err(HostError::BrowserUnavailable)
        }
    }

    /// 仅供已认证组合根提供会话；不是GUI或外部Agent传输入口。
    pub fn query_session(
        &mut self,
        session: &mut yonder_application::gateway::GatewaySession<'_>,
        request: &[u8],
        now_ms: u64,
    ) -> Result<Vec<u8>, HostError> {
        #[cfg(target_os = "macos")]
        let browser = self
            .browser
            .as_ref()
            .map(|port| port as &dyn BrowserUsePort);
        #[cfg(target_os = "macos")]
        let controlled_computer=self.computer.as_ref().map(|inner|CuaControlPort{inner,hub:self.cua_control.as_ref()});
        #[cfg(target_os = "macos")]
        let computer = controlled_computer.as_ref().map(|port|port as &dyn ComputerUsePort);
        #[cfg(target_os = "macos")]
        let computer_task = yonder_application::gateway::computer_request_task(request);
        #[cfg(target_os = "macos")]
        let captured_target = if computer_task.is_some() {
            self.targets.frontmost().ok()
        } else {
            None
        };
        #[cfg(target_os = "macos")]
        let fixed_target = captured_target.clone().map(FixedTarget);
        #[cfg(target_os = "macos")]
        let targets = fixed_target
            .as_ref()
            .map(|value| value as &dyn WorkTargetPort)
            .or(Some(&self.targets as &dyn WorkTargetPort));
        #[cfg(target_os = "macos")]
        let computer_permission_required = !self.targets.available();
        #[cfg(not(target_os = "macos"))]
        let browser: Option<&dyn BrowserUsePort> = None;
        #[cfg(not(target_os = "macos"))]
        let computer: Option<&dyn ComputerUsePort> = None;
        #[cfg(not(target_os = "macos"))]
        let targets: Option<&dyn WorkTargetPort> = None;
        #[cfg(not(target_os = "macos"))]
        let computer_permission_required = false;
        #[cfg(target_os = "macos")]
        let commands = Some(&self.commands as &dyn yonder_application::command::CommandPort);
        #[cfg(target_os = "macos")]
        let jev_config = self.store.get_jev_config().map_err(|_| HostError::StorageUnavailable)?.unwrap_or_default();
        #[cfg(target_os = "macos")]
        let jev = self.jev.as_ref().map(|port| port as &dyn yonder_application::jev_runtime::JevDecisionPort);
        #[cfg(not(target_os = "macos"))]
        let commands: Option<&dyn yonder_application::command::CommandPort> = None;
        #[cfg(not(target_os = "macos"))]
        let jev_config: Option<&yonder_application::jev_config::JevConfig> = None;
        #[cfg(not(target_os = "macos"))]
        let jev: Option<&dyn yonder_application::jev_runtime::JevDecisionPort> = None;
        let (response, accepted_create) = session
            .handle_encoded_with_runtimes_and_file_grants(
                &mut self.store,
                &self.admission,
                Some(&self.file_grants),
                Some(&self.command_approvals),
                Some(&self.files),
                Some(&self.documents),
                commands,
                #[cfg(target_os = "macos")]
                Some(&jev_config),
                #[cfg(not(target_os = "macos"))]
                jev_config,
                jev,
                browser,
                computer,
                targets,
                computer_permission_required,
                &self.host_session_id,
                request,
                now_ms,
            )
            .map_err(|_| HostError::StorageUnavailable)?;
        #[cfg(target_os = "macos")]
        if let Some(target) = captured_target {
            if yonder_application::gateway::response_is_computer_success(&response) {
                if let Some(attempt) = self
                    .store
                    .get_attempt(computer_task.as_deref().unwrap_or(""))
                    .map_err(|_| HostError::StorageUnavailable)?
                {
                    if let Some(previous) = self.work_refs.remove(&attempt.task_id) {
                        self.focus.release(&previous);
                    }
                    if let Ok(reference) = capture_after_observe(&mut self.focus, &attempt, &target)
                    {
                        self.work_refs.insert(attempt.task_id.clone(), reference);
                    }
                }
            }
        }
        if accepted_create {
            self.listening_pending = true;
            self.listening_until = None;
            self.task_space_open_pending = true;
        }
        Ok(response)
    }

    /// 仅供已认证 Local Socket 在成功新建后消费一次展示请求。
    pub fn take_task_space_open_pending(&mut self) -> bool {
        std::mem::take(&mut self.task_space_open_pending)
    }

    fn running_step_label(&mut self, task_id: &str) -> Option<String> {
        self.store
            .get_presentation(task_id)
            .ok()
            .filter(|(task, _)| task.status == yonder_application::Status::Running)
            .and_then(|(_, presentation)| presentation.current_step.map(|step| step.label))
    }

    /// 当前请求已经由协议与连接身份校验；只为它的真实可执行任务解析步骤。
    pub fn execution_step_label(
        &mut self,
        agent_id: &str,
        hint: &yonder_application::gateway::ExecutionPresentationHint,
    ) -> Option<String> {
        use yonder_application::gateway::ExecutionPresentationHint;
        let task_id = match hint {
            ExecutionPresentationHint::StoredStep { task_id }
            | ExecutionPresentationHint::DeclaredStep { task_id, .. }
            | ExecutionPresentationHint::PlanSlot { task_id, .. } => task_id,
        };
        let task = self.store.get(task_id).ok()?;
        if task.owner_agent_id != agent_id
            || !matches!(
                task.status,
                yonder_application::Status::Created | yonder_application::Status::Running
            )
        {
            return None;
        }
        match hint {
            ExecutionPresentationHint::StoredStep { .. } => self
                .store
                .get_presentation(task_id)
                .ok()?
                .1
                .current_step
                .map(|step| step.label),
            ExecutionPresentationHint::DeclaredStep { label, .. } => Some(label.clone()),
            ExecutionPresentationHint::PlanSlot {
                plan_id,
                plan_version,
                ..
            } => {
                let stored = self
                    .store
                    .get_plan_fragment(task_id, plan_id, *plan_version)
                    .ok()??;
                stored
                    .fragment
                    .slots
                    .get(usize::from(stored.current_slot))
                    .map(|slot| slot.label.clone())
            }
        }
    }

    /// 只投影已校验执行请求对应的步骤标签；不把动作参数或观察数据交给 UI。
    pub fn cua_control_presentation(
        &mut self,
        agent_id: &str,
        hint: &yonder_application::gateway::ExecutionPresentationHint,
    ) -> Option<CuaControlPresentation> {
        use yonder_application::gateway::ExecutionPresentationHint;
        let task_id = match hint {
            ExecutionPresentationHint::StoredStep { task_id }
            | ExecutionPresentationHint::DeclaredStep { task_id, .. }
            | ExecutionPresentationHint::PlanSlot { task_id, .. } => task_id,
        };
        let task = self.store.get(task_id).ok()?;
        if task.owner_agent_id != agent_id || !matches!(task.status, yonder_application::Status::Created | yonder_application::Status::Running) {
            return None;
        }
        match hint {
            ExecutionPresentationHint::PlanSlot { plan_id, plan_version, .. } => {
                let stored = self.store.get_plan_fragment(task_id, plan_id, *plan_version).ok()??;
                let current = usize::from(stored.current_slot);
                let start = current.saturating_sub(1);
                let end = (start + 4).min(stored.fragment.slots.len());
                let planned_steps = stored.fragment.slots[start..end].iter().enumerate().map(|(offset, slot)| {
                    let index = start + offset;
                    CuaControlStepPresentation {
                        step_id: slot.step_id.clone(), label: slot.label.clone(),
                        state: if index < current { "completed".into() } else { "pending".into() },
                    }
                }).collect::<Vec<_>>();
                let current_step = stored.fragment.slots.get(current).map(|slot| slot.label.clone())
                    .unwrap_or_else(|| "计划步骤已完成".into());
                Some(CuaControlPresentation {
                    task_id: task_id.clone(), current_step, planned_steps,
                    remaining_steps: u16::try_from(stored.fragment.slots.len().saturating_sub(end)).unwrap_or(u16::MAX),
                    plan_status: "available".into(),
                })
            }
            ExecutionPresentationHint::DeclaredStep { label, .. } => Some(CuaControlPresentation {
                task_id: task_id.clone(), current_step: label.clone(), planned_steps: vec![], remaining_steps: 0, plan_status: "none".into(),
            }),
            ExecutionPresentationHint::StoredStep { .. } => Some(CuaControlPresentation {
                task_id: task_id.clone(), current_step: self.running_step_label(task_id).unwrap_or_else(|| "正在执行当前步骤".into()),
                planned_steps: vec![], remaining_steps: 0, plan_status: "none".into(),
            }),
        }
    }

    /// 本地桌宠派生展示；执行优先，否则以首个未结束任务为代表。非执行/隐藏许可。
    pub fn presentation(&mut self) -> Result<(bool, &'static str, Option<String>), HostError> {
        let tasks = self
            .store
            .list(None, None, false, 1)
            .map_err(|_| HostError::StorageUnavailable)?;
        let mut step_label = None;
        let state = match self.activity() {
            ActivityState::Busy => {
                self.listening_pending = false;
                self.listening_until = None;
                step_label = self
                    .store
                    .list_running(None, None, 1)
                    .ok()
                    .and_then(|running| running.first().map(|task| task.id.clone()))
                    .and_then(|task_id| self.running_step_label(&task_id));
                "executing"
            }
            ActivityState::Unknown => return Err(HostError::StorageUnavailable),
            ActivityState::NoKnownWork => match tasks.first().map(|task| task.status) {
                _ if self.listening_pending => {
                    self.listening_pending = false;
                    self.listening_until = Some(Instant::now() + Duration::from_millis(1600));
                    "listening"
                }
                _ if self
                    .listening_until
                    .is_some_and(|until| Instant::now() < until) =>
                {
                    "listening"
                }
                Some(yonder_application::Status::WaitingForUser) => "waiting_for_user",
                Some(
                    yonder_application::Status::Paused | yonder_application::Status::Interrupted,
                ) => "paused",
                _ => "idle",
            },
        };
        Ok((!tasks.is_empty(), state, step_label))
    }

    /// 是观察而非隐藏许可；正式收起仍需预约协调。
    pub fn activity(&mut self) -> ActivityState {
        yonder_application::activity_state(&mut self.store, Some(&self.admission))
    }

    pub fn jev_config(&mut self) -> Result<JevConfig, HostError> {
        self.store
            .get_jev_config()
            .map_err(|_| HostError::StorageUnavailable)
            .map(|value| value.unwrap_or_default())
    }

    pub fn save_jev_config(&mut self, config: JevConfig) -> Result<JevConfig, HostError> {
        config
            .validate()
            .map_err(|_| HostError::InvalidConfig)?;
        self.store
            .save_jev_config(&config)
            .map_err(|_| HostError::StorageUnavailable)
    }

    #[cfg(target_os = "macos")]
    pub fn jev_decision(
        &mut self,
        request: JevDecisionRequest,
    ) -> Result<JevDecision, JevDecisionError> {
        let config = self
            .jev_config()
            .map_err(|_| JevDecisionError::InvalidConfig)?;
        let Some(port) = self.jev.as_ref() else {
            return Err(JevDecisionError::DependencyUnavailable);
        };
        yonder_application::jev_runtime::decide(&config, port, &request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cua_control_hub_only_accepts_the_active_task_once() {
        let hub=CuaControlHub::default();
        let presentation=|task_id:&str| CuaControlPresentation { task_id:task_id.into(), current_step:"步骤一".into(), planned_steps:vec![CuaControlStepPresentation { step_id:"step-1".into(), label:"步骤一".into(), state:"pending".into() }], remaining_steps:0, plan_status:"available".into() };
        assert!(!hub.request_takeover("task-a"));
        assert!(hub.begin(presentation("task-a")));
        assert!(!hub.request_takeover("task-b"));
        assert!(hub.request_takeover("task-a"));
        assert!(hub.take_takeover_requested("task-a"));
        assert!(hub.presentation().is_some());
        assert!(!hub.take_takeover_requested("task-a"));
        assert!(hub.finish("task-a"));
        assert!(!hub.finish("task-a"));
        assert!(hub.begin(presentation("task-b")));
        assert!(hub.finish("task-b"));
    }

    #[test]
    fn cua_control_hub_survives_step_boundaries_until_the_task_finishes() {
        let hub=CuaControlHub::default();
        let presentation=|label:&str| CuaControlPresentation { task_id:"task-a".into(), current_step:label.into(), planned_steps:vec![], remaining_steps:0, plan_status:"none".into() };
        assert!(hub.begin(presentation("打开企业微信")));
        hub.mark_executing("task-a","launch");
        assert_eq!(hub.presentation().unwrap().current_step,"打开企业微信");

        // 同一任务的下一次 RPC 只刷新投影，不能把任务级控制状态当作上一步一起清理。
        assert!(hub.begin(presentation("定位会话")));
        assert_eq!(hub.presentation().unwrap().current_step,"定位会话");
        assert!(!hub.take_takeover_requested("task-a"));
        assert!(hub.presentation().is_some());

        assert!(hub.finish("task-a"));
        assert!(hub.presentation().is_none());
    }

    #[test]
    fn cua_control_hub_projects_the_real_dispatch_step_without_action_arguments() {
        let hub=CuaControlHub::default();
        assert!(hub.begin(CuaControlPresentation {
            task_id:"task-a".into(), current_step:"打开设置".into(), remaining_steps:2, plan_status:"available".into(),
            planned_steps:vec![
                CuaControlStepPresentation { step_id:"open".into(), label:"打开设置".into(), state:"pending".into() },
                CuaControlStepPresentation { step_id:"save".into(), label:"保存更改".into(), state:"pending".into() },
            ],
        }));
        hub.mark_executing("task-a","save");
        let value=serde_json::to_value(hub.presentation().unwrap()).unwrap();
        assert_eq!(value["current_step"],"保存更改");
        assert_eq!(value["planned_steps"][0]["state"],"completed");
        assert_eq!(value["planned_steps"][1]["state"],"executing");
        assert!(value.to_string().contains("打开设置"));
        assert!(!value.to_string().contains("arguments"));
    }
    use yonder_application::{Action, create, transition};

    #[test]
    fn lock_child_probe() {
        if let Some(directory) = std::env::var_os("YONDA_TEST_HOST_DIRECTORY") {
            assert_eq!(
                TaskHost::open(Path::new(&directory)).err(),
                Some(HostError::LockUnavailable)
            );
        }
    }

    #[test]
    fn failed_start_preserves_database_and_releases_host_lock() {
        let directory = std::env::temp_dir().join(format!(
            "yonda-host-failed-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let database = directory.join("tasks.db");
        std::fs::write(&database, b"invalid database").unwrap();
        assert_eq!(
            TaskHost::open(&directory).err(),
            Some(HostError::StorageUnavailable)
        );
        assert_eq!(std::fs::read(&database).unwrap(), b"invalid database");
        // 仅测试清理本次生成的非法样本；产品不会删除或回退旧库。
        std::fs::remove_file(&database).unwrap();
        drop(TaskHost::open(&directory).unwrap());
        for name in ["tasks.db", "host.lock"] {
            std::fs::remove_file(directory.join(name)).unwrap();
        }
        std::fs::remove_dir(directory.join("observations")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn region_entry_pauses_only_confirmed_desktop_boundaries() {
        use yonder_application::{AttemptPhase, ExecutionAttempt, admission::{Resource,start_attempt}, computer_use::{DispatchOutcome,UnknownReason,record_dispatch_outcome}};
        for (id,outcome,advance,expected) in [
            ("stopped",Some(DispatchOutcome::Known{action_succeeded:true,observation:None}),true,Ok(true)),
            ("observed",Some(DispatchOutcome::Known{action_succeeded:true,observation:None}),false,Ok(true)),
            ("prepared",None,false,Err(HostError::StorageUnavailable)),
            ("unknown",Some(DispatchOutcome::Unknown(UnknownReason::ObserveFailed)),false,Err(HostError::StorageUnavailable)),
        ]{
            let directory = std::env::temp_dir().join(format!("yonda-region-pause-{}-{}-{}",id,std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            std::fs::create_dir(&directory).unwrap();
            let mut host=TaskHost::open(&directory).unwrap();
            let task=host.store.register("agent",id,"test",Some(id),yonder_application::TaskSource::LocalAgent).unwrap();
            let (task,step)=host.store.declare_step("agent",&task.id,task.sequence,"step","test").unwrap();
            let attempt=ExecutionAttempt{task_id:task.id.clone(),step_id:step.step_id,attempt_id:format!("attempt_{}",task.sequence+1),worker_instance_id:"worker".into(),host_session_id:"host".into(),phase:AttemptPhase::Prepared,accepted_sequence:0};
            let (_,attempt,permit)=start_attempt(&mut host.store,&host.admission,&attempt,task.sequence,&[Resource::Desktop]).unwrap(); drop(permit);
            let task=if let Some(outcome)=outcome{record_dispatch_outcome(&mut host.store,&attempt.task_id,&attempt.attempt_id,outcome).unwrap().0}else{host.store.get(&task.id).unwrap()};
            if advance { yonder_application::advance_after_observe(&mut host.store,&task.id,&attempt.attempt_id).unwrap(); }
            assert_eq!(host.pause_desktop_for_user(),expected);
            assert_eq!(host.admission.holds_resource(&task.id,Resource::Desktop).unwrap(),expected.is_err());
            assert_eq!(host.store.get(&task.id).unwrap().status,if expected.is_err(){yonder_application::Status::Running}else{yonder_application::Status::Paused});
            drop(host);
            for name in ["tasks.db","host.lock"]{std::fs::remove_file(directory.join(name)).unwrap();}
            std::fs::remove_dir(directory.join("observations")).unwrap();std::fs::remove_dir(directory).unwrap();
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_grants_are_task_bound_and_agent_revocation_clears_them() {
        let directory = std::env::temp_dir().join(format!(
            "yonda-file-grant-host-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let selected = directory.join("selected.docx");
        std::fs::write(&selected, b"not a real document").unwrap();
        let mut host = TaskHost::open(&directory).unwrap();
        let now = 1_000_000_000_000;
        host.register_agent("agent-a", now).unwrap();
        let task = host
            .store
            .register(
                "agent-a",
                "file-grant-test",
                "授权测试",
                Some("测试"),
                yonder_application::TaskSource::LocalAgent,
            )
            .unwrap();
        let granted = host
            .issue_file_grant(&task.id, FileGrantPurpose::Read, &selected, now + 10)
            .unwrap();
        let command=host.command_approvals.propose(AuthContext::Agent("agent-a"),&task,yonder_application::command::CommandRequest{program:"/usr/bin/printf".into(),args:vec!["safe".into()],cwd:directory.to_str().unwrap().into(),env:std::collections::BTreeMap::new(),timeout_ms:1000},now+10).unwrap();
        host.approve_command(&task.id,&command.command_id,now+11).unwrap();
        assert!(granted.grant_id.starts_with("file_grant_"));
        assert_eq!(
            host.list_file_grants(&task.id, now + 11).unwrap().len(),
            1,
            "本机视图只列出安全摘要"
        );
        let mut session = yonder_application::gateway::GatewaySession::new(
            AuthContext::Agent("agent-a"),
            yonder_application::gateway::Platform::Macos,
        );
        let hello = format!(r#"{{"jsonrpc":"2.0","id":"hello","method":"gateway.hello","params":{{"agent_id":"agent-a","capability":"task.read","deadline":{},"protocol_version":{{"major":1,"minor":27}}}}}}"#, now + 2_000);
        let response = String::from_utf8(host.query_session(&mut session, hello.as_bytes(), now + 100).unwrap()).unwrap();
        assert!(response.contains("file.grant.read"));
        let query = format!(
            r#"{{"jsonrpc":"2.0","id":"grants","method":"task.file.grants","params":{{"agent_id":"agent-a","capability":"file.grant.read","deadline":{},"task_id":"{}"}}}}"#,
            now + 2_000, task.id
        );
        let response = String::from_utf8(host.query_session(&mut session, query.as_bytes(), now + 101).unwrap()).unwrap();
        assert!(response.contains(&granted.grant_id));
        assert!(!response.contains("selected.docx") && !response.contains("authorized_root"));
        host.set_agent_status("agent-a", AgentRegistrationStatus::Disabled, now + 12)
            .unwrap();
        assert!(host.list_file_grants(&task.id, now + 13).unwrap().is_empty());
        assert!(host.list_command_approvals(&task.id,now+13).unwrap().is_empty());
        assert!(String::from_utf8(host.query_session(&mut session, query.as_bytes(), now + 102).unwrap())
            .unwrap()
            .contains("-32003"));
        drop(host);
        for name in ["selected.docx", "tasks.db", "host.lock"] {
            std::fs::remove_file(directory.join(name)).unwrap();
        }
        std::fs::remove_dir(directory.join("observations")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn file_execute_consumes_replace_grant_without_returning_a_path() {
        let directory = std::env::temp_dir().join(format!("yonda-file-exec-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let selected = directory.join("secret.txt"); std::fs::write(&selected, b"first").unwrap();
        let mut host = TaskHost::open(&directory).unwrap(); let now = 1_000_000_000_000;
        host.register_agent("agent-a", now).unwrap();
        let task = host.store.register("agent-a", "file-exec-test", "文件执行测试", Some("测试"), yonder_application::TaskSource::LocalAgent).unwrap();
        let (task, _) = host.store.declare_step("agent-a", &task.id, task.sequence, "edit", "替换文本").unwrap();
        let grant = host.issue_file_grant(&task.id, FileGrantPurpose::Replace, &selected, now + 10).unwrap();
        let mut session = yonder_application::gateway::GatewaySession::new(AuthContext::Agent("agent-a"), yonder_application::gateway::Platform::Macos);
        let hello = format!(r#"{{"jsonrpc":"2.0","id":"hello","method":"gateway.hello","params":{{"agent_id":"agent-a","capability":"task.read","deadline":{},"protocol_version":{{"major":1,"minor":28}}}}}}"#, now + 2_000);
        assert!(String::from_utf8(host.query_session(&mut session, hello.as_bytes(), now + 20).unwrap()).unwrap().contains("file.execute"));
        let execute = format!(r#"{{"jsonrpc":"2.0","id":"execute","method":"task.file.execute","params":{{"agent_id":"agent-a","capability":"file.execute","deadline":{},"task_id":"{}","expected_sequence":"{}","grant_id":"{}","operation":"replace","data_base64":"c2Vjb25k"}}}}"#, now + 2_000, task.id, task.sequence, grant.grant_id);
        let response = String::from_utf8(host.query_session(&mut session, execute.as_bytes(), now + 21).unwrap()).unwrap();
        assert!(response.contains("bytes_written") && !response.contains("secret.txt") && !response.contains(directory.to_str().unwrap()));
        assert_eq!(std::fs::read(&selected).unwrap(), b"second");
        assert!(host.list_file_grants(&task.id, now + 22).unwrap().is_empty(), "一次性替换授权必须已消费");
        drop(host);
        for name in ["secret.txt", "tasks.db", "host.lock"] { std::fs::remove_file(directory.join(name)).unwrap(); }
        std::fs::remove_dir(directory.join("observations")).unwrap(); std::fs::remove_dir(directory).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn command_gateway_requires_local_approval_and_executes_reference_once() {
        let directory=std::env::temp_dir().join(format!("yonda-command-exec-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let mut host=TaskHost::open(&directory).unwrap(); let now=1_000_000_000_000;
        host.register_agent("agent-a",now).unwrap();
        let task=host.store.register("agent-a","command-gateway","命令执行测试",Some("测试"),yonder_application::TaskSource::LocalAgent).unwrap();
        let (task,_)=host.store.declare_step("agent-a",&task.id,task.sequence,"run-command","执行命令").unwrap();
        let mut session=yonder_application::gateway::GatewaySession::new(AuthContext::Agent("agent-a"),yonder_application::gateway::Platform::Macos);
        let hello=format!(r#"{{"jsonrpc":"2.0","id":"hello","method":"gateway.hello","params":{{"agent_id":"agent-a","capability":"task.read","deadline":{},"protocol_version":{{"major":1,"minor":30}}}}}}"#,now+2_000);
        let response=String::from_utf8(host.query_session(&mut session,hello.as_bytes(),now+10).unwrap()).unwrap();
        assert!(response.contains("command.propose")&&response.contains("command.execute"));
        let propose=format!(r#"{{"jsonrpc":"2.0","id":"propose","method":"task.command.propose","params":{{"agent_id":"agent-a","capability":"command.propose","deadline":{},"task_id":"{}","program":"/usr/bin/printf","args":["%s","gateway-ok"],"cwd":"{}","env":{{}},"timeout_ms":1000}}}}"#,now+2_000,task.id,directory.to_str().unwrap());
        let response:serde_json::Value=serde_json::from_slice(&host.query_session(&mut session,propose.as_bytes(),now+11).unwrap()).unwrap();
        let command_id=response["result"]["approval"]["command_id"].as_str().expect("命令提议响应").to_owned();
        assert_eq!(host.store.get(&task.id).unwrap().status,yonder_application::Status::Created);
        assert!(host.store.get_attempt(&task.id).unwrap().is_none());
        host.approve_command(&task.id,&command_id,now+12).unwrap();
        let execute=format!(r#"{{"jsonrpc":"2.0","id":"execute","method":"task.command.execute","params":{{"agent_id":"agent-a","capability":"command.execute","deadline":{},"task_id":"{}","expected_sequence":"{}","command_id":"{}"}}}}"#,now+2_000,task.id,task.sequence,command_id);
        let response_bytes=host.query_session(&mut session,execute.as_bytes(),now+13).unwrap();
        let response_text=String::from_utf8(response_bytes.clone()).unwrap();
        assert!(!response_text.contains("/usr/bin/printf")&&!response_text.contains(directory.to_str().unwrap())&&!response_text.contains("gateway-ok"));
        let response:serde_json::Value=serde_json::from_slice(&response_bytes).unwrap();
        let execution=&response["result"]["execution"];
        assert_eq!(execution["outcome"],"exited");
        assert_eq!(execution["exit_code"],0);
        assert_eq!(execution["stdout_base64"],"Z2F0ZXdheS1vaw==");
        assert_eq!(execution["attempt_result"]["action_succeeded"],true);
        assert!(host.list_command_approvals(&task.id,now+14).unwrap().is_empty());
        let replay=String::from_utf8(host.query_session(&mut session,execute.as_bytes(),now+15).unwrap()).unwrap();
        assert!(replay.contains("-32011")||replay.contains("-32012"));
        assert!(!replay.contains("gateway-ok")&&!replay.contains("/usr/bin/printf"));
        drop(host);
        for name in ["tasks.db","host.lock"]{std::fs::remove_file(directory.join(name)).unwrap();}
        std::fs::remove_dir(directory.join("observations")).unwrap(); std::fs::remove_dir(directory).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn document_execute_uses_dual_grants_and_default_save_as_without_paths() {
        let directory = std::env::temp_dir().join(format!("yonda-document-exec-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        let source = directory.join("source.docx");
        let output = directory.join("output.docx");
        std::fs::write(&source, include_bytes!("../../../spikes/ooxml-adapter-comparison/fixtures/synthetic/sample.docx")).unwrap();
        let mut host = TaskHost::open(&directory).unwrap(); let now = 1_000_000_000_000;
        host.register_agent("agent-a", now).unwrap();
        let task = host.store.register("agent-a", "document-exec-test", "文档执行测试", Some("测试"), yonder_application::TaskSource::LocalAgent).unwrap();
        let (task, _) = host.store.declare_step("agent-a", &task.id, task.sequence, "edit", "替换文档文本").unwrap();
        let expected_hash = yonder_application::file::read(&host.files, &FileReadRequest { path: source.to_str().unwrap().into(), authorized_root: directory.to_str().unwrap().into() }).unwrap().sha256;
        let source_grant = host.issue_file_grant(&task.id, FileGrantPurpose::Read, &source, now + 10).unwrap();
        let output_grant = host.issue_file_grant(&task.id, FileGrantPurpose::CreateNew, &output, now + 10).unwrap();
        let mut session = yonder_application::gateway::GatewaySession::new(AuthContext::Agent("agent-a"), yonder_application::gateway::Platform::Macos);
        let hello = format!(r#"{{"jsonrpc":"2.0","id":"hello","method":"gateway.hello","params":{{"agent_id":"agent-a","capability":"task.read","deadline":{},"protocol_version":{{"major":1,"minor":29}}}}}}"#, now + 2_000);
        assert!(String::from_utf8(host.query_session(&mut session, hello.as_bytes(), now + 20).unwrap()).unwrap().contains("document.execute"));
        let execute = format!(r#"{{"jsonrpc":"2.0","id":"execute","method":"task.document.execute","params":{{"agent_id":"agent-a","capability":"document.execute","deadline":{},"task_id":"{}","expected_sequence":"{}","source_grant_id":"{}","output_grant_id":"{}","expected_hash":"{}","before":"YONDER_DOCX_BEFORE","after":"YONDER_DOCX_AFTER"}}}}"#, now + 2_000, task.id, task.sequence, source_grant.grant_id, output_grant.grant_id, expected_hash);
        let response = String::from_utf8(host.query_session(&mut session, execute.as_bytes(), now + 21).unwrap()).unwrap();
        assert!(response.contains("document-execution") && response.contains("bytes_written"));
        assert!(!response.contains("source.docx") && !response.contains("output.docx") && !response.contains(directory.to_str().unwrap()));
        assert!(output.is_file() && std::fs::read(&source).unwrap() == include_bytes!("../../../spikes/ooxml-adapter-comparison/fixtures/synthetic/sample.docx"));
        assert!(host.list_file_grants(&task.id, now + 22).unwrap().iter().all(|grant| grant.grant_id != output_grant.grant_id), "输出新建授权必须已消费");
        drop(host);
        for name in ["source.docx", "output.docx", "tasks.db", "host.lock"] { std::fs::remove_file(directory.join(name)).unwrap(); }
        std::fs::remove_dir(directory.join("observations")).unwrap(); std::fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn host_locks_recovers_and_queries_real_tasks_without_trusting_request_identity() {
        assert_eq!(
            TaskHost::open(Path::new("relative")).err(),
            Some(HostError::InvalidDirectory)
        );
        let directory = std::env::temp_dir().join(format!(
            "yonda-host-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let mut original = SqliteTaskStore::open_unencrypted(&directory.join("tasks.db")).unwrap();
        assert_eq!(
            create(&mut original, "manual", AuthContext::LocalUser("desktop")),
            Err(yonder_application::Error::PermissionDenied)
        );
        assert_eq!(
            yonder_application::get(&mut original, "manual"),
            Err(yonder_application::Error::NotFound)
        );
        for (id, agent) in [("task-a", "agent-a"), ("task-b", "agent-b")] {
            create(&mut original, id, AuthContext::Agent(agent)).unwrap();
            transition(&mut original, id, 1, Action::Start).unwrap();
        }
        drop(original);
        let mut host = TaskHost::open(&directory).unwrap();
        assert_eq!(
            TaskHost::open(&directory).err(),
            Some(HostError::LockUnavailable)
        );
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "tests::lock_child_probe"])
            .env("YONDA_TEST_HOST_DIRECTORY", &directory)
            .output()
            .unwrap();
        assert!(child.status.success(), "跨进程锁验证失败：{:?}", child);
        let request = br#"{"jsonrpc":"2.0","id":"r1","method":"task.list","params":{"agent_id":"desktop","capability":"task.read","deadline":2000,"limit":100}}"#;
        let response = String::from_utf8(host.query(request, 1000).unwrap()).unwrap();
        for field in ["task-a", "task-b", "agent-a", "agent-b"] {
            assert!(response.contains(field));
        }
        assert_eq!(response.matches("interrupted").count(), 2);
        assert_eq!(response.matches("\"sequence\":\"3\"").count(), 2);
        let timeline = host
            .store
            .register(
                "agent-a",
                "timeline",
                "时间线",
                Some("验证"),
                yonder_application::TaskSource::LocalAgent,
            )
            .unwrap();
        host.store
            .declare_step(
                "agent-a",
                &timeline.id,
                timeline.sequence,
                "inspect",
                "检查时间线",
            )
            .unwrap();
        let events = format!(
            r#"{{"jsonrpc":"2.0","id":"e","method":"task.events","params":{{"agent_id":"desktop","capability":"task.read","deadline":2000,"task_id":"{}","after_sequence":"0","limit":100}}}}"#,
            timeline.id
        );
        let events = String::from_utf8(host.query(events.as_bytes(), 1000).unwrap()).unwrap();
        assert!(events.contains("step_declaration") && events.contains("检查时间线"));
        transition(&mut host.store, &timeline.id, 2, Action::Cancel).unwrap();
        use yonder_application::gateway::{GatewaySession, Platform};
        for agent in ["agent-a", "agent-b"] {
            AgentRegistry::register_agent(&mut host.store, agent, 1_000_000_000_000).unwrap();
        }
        for (agent, own, other) in [
            ("agent-a", "task-a", "task-b"),
            ("agent-b", "task-b", "task-a"),
        ] {
            let mut session = GatewaySession::new(AuthContext::Agent(agent), Platform::Macos);
            let query = String::from_utf8(request.to_vec())
                .unwrap()
                .replace("desktop", agent);
            let read = |host: &mut TaskHost, session: &mut GatewaySession<'_>, bytes: &[u8]| {
                String::from_utf8(host.query_session(session, bytes, 1000).unwrap()).unwrap()
            };
            assert!(read(&mut host, &mut session, query.as_bytes()).contains("-32002"));
            let hello = format!(
                r#"{{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{{"agent_id":"{agent}","capability":"task.read","deadline":2000,"protocol_version":{{"major":1,"minor":0}}}}}}"#
            );
            assert!(read(&mut host, &mut session, hello.as_bytes()).contains("hello"));
            let response = read(&mut host, &mut session, query.as_bytes());
            assert!(response.contains(own));
            assert!(!response.contains(other));
            let get = format!(
                r#"{{"jsonrpc":"2.0","id":"g","method":"task.get","params":{{"agent_id":"{agent}","capability":"task.read","deadline":2000,"task_id":"{other}"}}}}"#
            );
            assert!(read(&mut host, &mut session, get.as_bytes()).contains("-32004"));
            let mut fresh = GatewaySession::new(AuthContext::Agent(agent), Platform::Macos);
            assert!(read(&mut host, &mut fresh, query.as_bytes()).contains("-32002"));
        }
        assert_eq!(host.activity(), ActivityState::NoKnownWork);
        assert_eq!(host.presentation().unwrap(), (true, "paused", None));
        transition(&mut host.store, "task-a", 3, Action::Resume).unwrap();
        assert_eq!(host.presentation().unwrap(), (true, "executing", None));
        transition(&mut host.store, "task-a", 4, Action::WaitForUser).unwrap();
        assert_eq!(host.presentation().unwrap(), (true, "waiting_for_user", None));
        transition(&mut host.store, "task-a", 5, Action::Resume).unwrap();
        transition(&mut host.store, "task-a", 6, Action::Pause).unwrap();
        assert_eq!(host.presentation().unwrap(), (true, "paused", None));
        transition(&mut host.store, "task-a", 7, Action::Resume).unwrap();
        transition(&mut host.store, "task-a", 8, Action::Complete).unwrap();
        transition(&mut host.store, "task-b", 3, Action::Cancel).unwrap();
        assert_eq!(host.presentation().unwrap(), (false, "idle", None));

        let forged = String::from_utf8(request.to_vec())
            .unwrap()
            .replace("desktop", "agent-a");
        let denied = String::from_utf8(host.query(forged.as_bytes(), 1000).unwrap()).unwrap();
        assert!(denied.contains("-32003"));
        assert!(!denied.contains("task-a"));
        drop(host);
        let mut reopened = TaskHost::open(&directory).unwrap();
        assert_eq!(reopened.presentation().unwrap(), (false, "idle", None));
        let mut session = GatewaySession::new(AuthContext::Agent("agent-c"), Platform::Macos);
        let create = br#"{"jsonrpc":"2.0","id":"c0","method":"task.create","params":{"agent_id":"agent-c","capability":"task.create","deadline":2000,"idempotency_key":"listen","description":"request","name":"request test"}}"#;
        assert!(
            String::from_utf8(reopened.query_session(&mut session, create, 1000).unwrap())
                .unwrap()
                .contains("-32003")
        );
        AgentRegistry::register_agent(&mut reopened.store, "agent-c", 1_000_000_000_000).unwrap();
        assert!(
            String::from_utf8(reopened.query_session(&mut session, create, 1000).unwrap())
                .unwrap()
                .contains("-32002")
        );
        assert_eq!(reopened.presentation().unwrap(), (false, "idle", None));
        let hello = br#"{"jsonrpc":"2.0","id":"h","method":"gateway.hello","params":{"agent_id":"agent-c","capability":"task.read","deadline":2000,"protocol_version":{"major":1,"minor":3}}}"#;
        reopened.query_session(&mut session, hello, 1000).unwrap();
        assert!(
            String::from_utf8(reopened.query_session(&mut session, create, 1000).unwrap())
                .unwrap()
                .contains("created")
        );
        assert_eq!(reopened.presentation().unwrap(), (true, "listening", None));
        std::thread::sleep(Duration::from_millis(1650));
        assert_ne!(reopened.presentation().unwrap().1, "listening");
        reopened.query_session(&mut session, create, 1000).unwrap();
        assert_eq!(reopened.presentation().unwrap(), (true, "idle", None));
        let created = reopened
            .store
            .list(None, None, false, 100)
            .unwrap()
            .into_iter()
            .find(|task| task.owner_agent_id == "agent-c")
            .unwrap();
        let (created, _) = reopened
            .store
            .declare_step(
                "agent-c",
                &created.id,
                created.sequence,
                "open-settings",
                "打开设置面板",
            )
            .unwrap();
        assert_eq!(
            reopened.execution_step_label(
                "agent-c",
                &yonder_application::gateway::ExecutionPresentationHint::StoredStep {
                    task_id: created.id.clone(),
                },
            ),
            Some("打开设置面板".into())
        );
        assert_eq!(
            reopened.execution_step_label(
                "agent-c",
                &yonder_application::gateway::ExecutionPresentationHint::DeclaredStep {
                    task_id: created.id.clone(),
                    label: "点击保存".into(),
                },
            ),
            Some("点击保存".into())
        );
        transition(
            &mut reopened.store,
            &created.id,
            created.sequence,
            Action::Start,
        )
        .unwrap();
        assert_eq!(
            reopened.presentation().unwrap(),
            (true, "executing", Some("打开设置面板".into()))
        );
        drop(reopened);
        for name in ["tasks.db", "host.lock"] {
            std::fs::remove_file(directory.join(name)).unwrap();
        }
        std::fs::remove_dir(directory.join("observations")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
}
