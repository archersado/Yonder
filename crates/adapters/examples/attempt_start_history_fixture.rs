//! 仅在隔离 Tauri 标识的新数据库内生成可信尝试开始与结果事实。
use std::path::Path;
use yonder_adapters::task_store::SqliteTaskStore;
use yonder_application::{
    AttemptConclusion, AttemptPhase, AuthContext, ExecutionAttempt, TaskSource, TaskStore,
    agent_registry::AgentRegistry, register,
};

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
            != Some("com.yonder.attempt-start.fixture")
    {
        return Err("仅允许全新尝试开始夹具目录下的 tasks.db".into());
    }
    std::fs::create_dir_all(path.parent().ok_or("缺少测试目录")?)?;
    let mut store = SqliteTaskStore::open_unencrypted(path)
        .map_err(|error| format!("初始化失败：{error:?}"))?;
    AgentRegistry::register_agent(&mut store, "fixture-agent", 1_000_000_000_000)
        .map_err(|error| format!("测试 Agent 登记失败：{error:?}"))?;
    let task = register(
        &mut store,
        AuthContext::Agent("fixture-agent"),
        "attempt-start-fixture",
        "隔离尝试开始任务",
        Some("原生尝试开始验证任务"),
        TaskSource::LocalAgent,
    )
    .map_err(|error| format!("任务登记失败：{error:?}"))?;
    let (task, _) = store
        .declare_step(
            "fixture-agent",
            &task.id,
            task.sequence,
            "step-one",
            "打开文档",
        )
        .map_err(|error| format!("步骤声明失败：{error:?}"))?;
    let (_, attempt) = store
        .prepare_attempt(
            &ExecutionAttempt {
                task_id: task.id.clone(),
                step_id: "step-one".into(),
                attempt_id: "attempt-one".into(),
                worker_instance_id: "worker-one".into(),
                host_session_id: "host-one".into(),
                phase: AttemptPhase::Prepared,
                accepted_sequence: 0,
            },
            task.sequence,
        )
        .map_err(|error| format!("尝试准备失败：{error:?}"))?;
    let (_, result) = store
        .record_attempt_result(
            &attempt,
            attempt.accepted_sequence,
            AttemptConclusion::Observed {
                action_succeeded: true,
            },
        )
        .map_err(|error| format!("尝试结果提交失败：{error:?}"))?;
    println!(
        "task_id={} start_sequence={} result_sequence={} database={}",
        task.id,
        attempt.accepted_sequence,
        result.result_sequence,
        path.display()
    );
    Ok(())
}
