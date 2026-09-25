//! 仅在隔离 Tauri 标识的新数据库内生成定位中与定位成功历史事实。
use std::path::Path;
use yonder_adapters::task_store::SqliteTaskStore;
use yonder_application::{
    AttemptPhase, AuthContext, ControlKind, ExecutionAttempt, TaskSource, TaskStore,
    admission::{Admission, Resource, start_attempt},
    agent_registry::AgentRegistry,
    computer_use::{DispatchOutcome, WorkTarget, record_dispatch_outcome},
    register, request_control,
    work_focus::{FocusFailure, FocusOutcome, WorkFocusPort, WorkRef, focus_takeover},
};

struct Focus;

impl WorkFocusPort for Focus {
    fn capture(&mut self, _: &ExecutionAttempt, _: &WorkTarget) -> Result<WorkRef, FocusFailure> {
        unreachable!()
    }

    fn focus(&mut self, _: &WorkRef) -> FocusOutcome {
        FocusOutcome::Focused
    }

    fn release(&mut self, _: &WorkRef) {}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("需要显式指定全新隔离数据库绝对路径")?;
    let path = Path::new(&path);
    if !path.is_absolute()
        || path.exists()
        || path.file_name().and_then(|value| value.to_str()) != Some("tasks.db")
        || path
            .parent()
            .and_then(|value| value.file_name())
            .and_then(|value| value.to_str())
            != Some("com.yonder.focus.fixture")
    {
        return Err("仅允许全新定位夹具目录下的 tasks.db".into());
    }
    std::fs::create_dir_all(path.parent().ok_or("缺少测试目录")?)?;
    let mut store = SqliteTaskStore::open_unencrypted(path)
        .map_err(|error| format!("初始化失败：{error:?}"))?;
    AgentRegistry::register_agent(&mut store, "fixture-agent", 1_000_000_000_000)
        .map_err(|error| format!("测试 Agent 登记失败：{error:?}"))?;
    let task = register(
        &mut store,
        AuthContext::Agent("fixture-agent"),
        "focus-fixture",
        "隔离定位任务",
        Some("原生定位历史验证任务"),
        TaskSource::LocalAgent,
    )
    .map_err(|error| format!("任务登记失败：{error:?}"))?;
    let (task, _) = store
        .declare_step(
            "fixture-agent",
            &task.id,
            task.sequence,
            "step-one",
            "验证接管定位",
        )
        .map_err(|error| format!("步骤声明失败：{error:?}"))?;
    let requested = ExecutionAttempt {
        task_id: task.id.clone(),
        step_id: "step-one".into(),
        attempt_id: "attempt-one".into(),
        worker_instance_id: "fixture-worker".into(),
        host_session_id: "fixture-host".into(),
        phase: AttemptPhase::Prepared,
        accepted_sequence: 0,
    };
    let admission = Admission::new(1).map_err(|error| format!("准入失败：{error:?}"))?;
    let (_, accepted, permit) = start_attempt(
        &mut store,
        &admission,
        &requested,
        task.sequence,
        &[Resource::Desktop],
    )
    .map_err(|error| format!("尝试启动失败：{error:?}"))?;
    let (observed, _) = record_dispatch_outcome(
        &mut store,
        &task.id,
        &accepted.attempt_id,
        DispatchOutcome::Known {
            action_succeeded: true,
            observation: None,
        },
    )
    .map_err(|error| format!("动作结果失败：{error:?}"))?;
    request_control(
        &mut store,
        AuthContext::LocalUser("desktop"),
        &task.id,
        observed.sequence,
        ControlKind::Takeover,
    )
    .map_err(|error| format!("控制登记失败：{error:?}"))?;
    let (paused, _) = permit
        .stop_at_boundary(&mut store, &accepted.attempt_id, ControlKind::Takeover)
        .map_err(|_| "步骤边界停止失败")?;
    let reference = WorkRef {
        work_ref_id: format!("work_{}", accepted.accepted_sequence),
        task_id: task.id.clone(),
        step_id: accepted.step_id.clone(),
        attempt_id: accepted.attempt_id.clone(),
        worker_instance_id: accepted.worker_instance_id.clone(),
        host_session_id: accepted.host_session_id.clone(),
        pid: 1,
        window_id: 1,
        process_start_seconds: 1,
        process_start_microseconds: 1,
    };
    let (focused, control, _) = focus_takeover(&mut store, &mut Focus, &reference)
        .map_err(|error| format!("定位历史提交失败：{error:?}"))?;
    println!(
        "task_id={} stopped_sequence={} locating_sequence={} focused_sequence={} control_id={} database={}",
        task.id,
        paused.sequence,
        focused.sequence - 1,
        focused.sequence,
        control.control_id,
        path.display()
    );
    Ok(())
}
