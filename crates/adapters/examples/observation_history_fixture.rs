//! 仅用于隔离 macOS 原生 E2E：在全新数据库内生成正式 TaskHost 可读取的 Observe 历史。
use std::path::Path;
use yonder_adapters::task_store::SqliteTaskStore;
use yonder_application::{
    AttemptPhase, AuthContext, ExecutionAttempt, TaskObservationResult, TaskSource, TaskStore,
    admission::{Admission, Resource, start_attempt},
    advance_after_observe,
    agent_registry::AgentRegistry,
    computer_use::{DispatchOutcome, UnknownReason, record_dispatch_outcome},
    prepare_next_attempt, record_observation, register,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let register_existing = args
        .first()
        .is_some_and(|value| value == "--register-existing");
    let path = args
        .get(if register_existing { 1 } else { 0 })
        .ok_or("需要显式指定测试数据库绝对路径")?;
    let path = Path::new(&path);
    if !path.is_absolute()
        || path.exists() != register_existing
        || path.file_name().and_then(|name| name.to_str()) != Some("tasks.db")
        || path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            != Some("com.yonder.observation.fixture")
    {
        return Err("仅允许隔离标识目录下的指定状态 tasks.db".into());
    }
    let parent = path.parent().ok_or("缺少测试目录")?;
    std::fs::create_dir_all(parent)?;
    let mut store = SqliteTaskStore::open_unencrypted(path)
        .map_err(|error| format!("初始化失败：{error:?}"))?;
    AgentRegistry::register_agent(&mut store, "fixture-agent", 1_000_000_000_000)
        .map_err(|error| format!("测试 Agent 登记失败：{error:?}"))?;
    if register_existing {
        println!("fixture-agent registered database={}", path.display());
        return Ok(());
    }
    let task = register(
        &mut store,
        AuthContext::Agent("fixture-agent"),
        "observe-fixture",
        "隔离观察任务",
        Some("原生 Observe 验证任务"),
        TaskSource::LocalAgent,
    )
    .map_err(|error| format!("登记失败：{error:?}"))?;
    let (task, _) = store
        .declare_step(
            "fixture-agent",
            &task.id,
            task.sequence,
            "step-one",
            "打开目标",
        )
        .map_err(|error| format!("步骤一失败：{error:?}"))?;
    let attempt = |step: &str, id: &str| ExecutionAttempt {
        task_id: task.id.clone(),
        step_id: step.into(),
        attempt_id: id.into(),
        worker_instance_id: "fixture-worker".into(),
        host_session_id: "fixture-host".into(),
        phase: AttemptPhase::Prepared,
        accepted_sequence: 0,
    };
    let admission = Admission::new(1).map_err(|error| format!("准入失败：{error:?}"))?;
    let (_, first, _) = start_attempt(
        &mut store,
        &admission,
        &attempt("step-one", "attempt-one"),
        task.sequence,
        &[Resource::Desktop],
    )
    .map_err(|error| format!("尝试一失败：{error:?}"))?;
    let (first_result, _) = record_dispatch_outcome(
        &mut store,
        &task.id,
        &first.attempt_id,
        DispatchOutcome::Known {
            action_succeeded: true,
            observation: None,
        },
    )
    .map_err(|error| format!("结果一失败：{error:?}"))?;
    record_observation(
        &mut store,
        &task.id,
        &first.attempt_id,
        first_result.sequence,
        TaskObservationResult::Matched,
        "目标已打开",
    )
    .map_err(|error| format!("观察一失败：{error:?}"))?;
    let (advanced, _) = advance_after_observe(&mut store, &task.id, &first.attempt_id)
        .map_err(|error| format!("边界推进失败：{error:?}"))?;
    let (declared, _) = store
        .declare_step(
            "fixture-agent",
            &task.id,
            advanced.sequence,
            "step-two",
            "核实目标",
        )
        .map_err(|error| format!("步骤二失败：{error:?}"))?;
    let (_, second) = prepare_next_attempt(
        &mut store,
        &attempt("step-two", "attempt-two"),
        declared.sequence,
    )
    .map_err(|error| format!("尝试二失败：{error:?}"))?;
    let (second_result, _) = record_dispatch_outcome(
        &mut store,
        &task.id,
        &second.attempt_id,
        DispatchOutcome::Unknown(UnknownReason::TimedOut),
    )
    .map_err(|error| format!("结果二失败：{error:?}"))?;
    let observed = record_observation(
        &mut store,
        &task.id,
        &second.attempt_id,
        second_result.sequence,
        TaskObservationResult::Unknown,
        "核实超时，结果未知",
    )
    .map_err(|error| format!("观察二失败：{error:?}"))?;
    println!(
        "task_id={} sequence={} database={}",
        task.id,
        observed.sequence,
        path.display()
    );
    Ok(())
}
