use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRegistrationStatus {
    Enabled,
    Disabled,
    Revoked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentRegistration {
    pub agent_id: String,
    pub status: AgentRegistrationStatus,
    pub registered_at: u64,
    pub last_seen_at: Option<u64>,
    pub updated_at: u64,
}

pub trait AgentRegistry {
    /// 仅可信本地组合根可调用；已禁用/撤权记录绝不被此方法复活。
    fn ensure_agent(&mut self, _agent_id: &str, _now_ms: u64) -> Result<AgentRegistration, crate::Error> {
        Err(crate::Error::StorageUnavailable)
    }
    fn register_agent(
        &mut self,
        agent_id: &str,
        now_ms: u64,
    ) -> Result<AgentRegistration, crate::Error>;
    fn list_agents(&mut self) -> Result<Vec<AgentRegistration>, crate::Error>;
    fn set_agent_status(
        &mut self,
        agent_id: &str,
        status: AgentRegistrationStatus,
        now_ms: u64,
    ) -> Result<AgentRegistration, crate::Error>;
    fn authorize_agent_write(&mut self, agent_id: &str, now_ms: u64) -> Result<(), crate::Error>;
}
