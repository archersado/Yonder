//! 仅在隔离 Tauri 标识的新数据库内生成可信云端创建来源事实。
use std::path::Path;
use yonder_adapters::task_store::SqliteTaskStore;
use yonder_application::{AuthContext, TaskSource, agent_registry::AgentRegistry, register};

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
            != Some("com.yonder.creation.fixture")
    {
        return Err("仅允许全新创建来源夹具目录下的 tasks.db".into());
    }
    std::fs::create_dir_all(path.parent().ok_or("缺少测试目录")?)?;
    let mut store = SqliteTaskStore::open_unencrypted(path)
        .map_err(|error| format!("初始化失败：{error:?}"))?;
    AgentRegistry::register_agent(&mut store, "fixture-agent", 1_000_000_000_000)
        .map_err(|error| format!("测试 Agent 登记失败：{error:?}"))?;
    let task = register(
        &mut store,
        AuthContext::Agent("fixture-agent"),
        "creation-fixture",
        "隔离创建来源任务",
        Some("原生创建来源验证任务"),
        TaskSource::CloudAgent,
    )
    .map_err(|error| format!("任务登记失败：{error:?}"))?;
    println!(
        "task_id={} sequence={} source=cloud-agent owner=fixture-agent database={}",
        task.id,
        task.sequence,
        path.display()
    );
    Ok(())
}
