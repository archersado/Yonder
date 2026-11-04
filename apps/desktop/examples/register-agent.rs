//! 原生 Gateway 验证夹具：通过正式 TaskHost API 在隔离数据库中登记测试 Agent。
use std::path::PathBuf;
use yonder_application::agent_registry::AgentRegistrationStatus;
use yonder_desktop::TaskHost;

fn main() {
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let directory = PathBuf::from(std::env::args_os().nth(1).ok_or("缺少测试目录")?);
        let agent_id = std::env::args().nth(2).ok_or("缺少Agent ID")?;
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
            .try_into()?;
        let mut host = TaskHost::open(&directory).map_err(|_| "测试宿主打开失败")?;
        match std::env::args().nth(3).as_deref() {
            Some("disable") => {
                host.set_agent_status(&agent_id, AgentRegistrationStatus::Disabled, now_ms)
                    .map_err(|_| "测试Agent停用失败")?;
            }
            None => {
                host.register_agent(&agent_id, now_ms)
                    .map_err(|_| "测试Agent登记失败")?;
            }
            Some(_) => return Err("未知操作".into()),
        }
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
