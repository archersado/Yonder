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
