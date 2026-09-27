//! EX-S2 受限计划片段的 Application 规则。
//!
//! 此模块只接受已由 Gateway 鉴权的不可变 CUA 候选；它不解析外部 JSON、
//! 不构造 Agent 身份，也不派发 Driver。实际副作用仍必须经 computer_use
//! 的统一启动、桌面租约和 Observe 链路。
use crate::{jev_config::JevCapability, jev_runtime::{self, JevCandidate, JevDecision, JevDecisionError, JevDecisionPort, JevDecisionRequest}, valid_id};

pub const MAX_SLOTS: usize = 10;
pub const MAX_TOKEN_BUDGET: u32 = 10_000;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CandidateAction {
    pub candidate_id: String,
    pub tool_name: String,
    pub arguments_json: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PlanSlot {
    pub step_id: String,
    pub label: String,
    pub candidates: Vec<CandidateAction>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct PlanFragment {
    pub plan_id: String,
    pub plan_version: u64,
    pub task_id: String,
    pub expected_sequence: u64,
    pub deadline_ms: u64,
    pub token_budget: u32,
    pub slots: Vec<PlanSlot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredPlanFragment {
    pub owner_agent_id: String,
    pub accepted_sequence: u64,
    pub current_slot: u16,
    pub fragment: PlanFragment,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Selection {
    Dispatch(CandidateAction),
    HandBack { reason: &'static str },
}

pub fn validate(fragment: &PlanFragment) -> Result<(), crate::Error> {
    if !valid_id(&fragment.plan_id) || !valid_id(&fragment.task_id)
        || fragment.plan_version == 0 || fragment.expected_sequence == 0
        || fragment.deadline_ms == 0 || fragment.token_budget == 0
        || fragment.token_budget > MAX_TOKEN_BUDGET || fragment.slots.is_empty()
        || fragment.slots.len() > MAX_SLOTS { return Err(crate::Error::InvalidInput); }
    let mut steps = std::collections::HashSet::new();
    for slot in &fragment.slots {
        if !valid_id(&slot.step_id) || !steps.insert(slot.step_id.as_str())
            || !crate::valid_step_label(&slot.label) || !(1..=9).contains(&slot.candidates.len()) { return Err(crate::Error::InvalidInput); }
        let mut candidates = std::collections::HashSet::new();
        for candidate in &slot.candidates {
            if !valid_candidate(candidate) || !candidates.insert(candidate.candidate_id.as_str()) { return Err(crate::Error::InvalidInput); }
        }
    }
    Ok(())
}

fn valid_candidate(candidate: &CandidateAction) -> bool {
    valid_id(&candidate.candidate_id)
        && yonder_protocol::valid_sdk_tool_name(&candidate.tool_name)
        && !candidate.arguments_json.is_empty()
        && candidate.arguments_json.len() <= 16 * 1024
        && serde_json::from_str::<serde_json::Value>(&candidate.arguments_json)
            .is_ok_and(|value| value.is_object() && yonder_protocol::safe_sdk_arguments(&value))
}

/// 从指定槽位作一次有界选择。调用方必须在持久化读取、CAS、控制检查及
/// 当前 Observe 检查后调用；返回 HandBack 不会产生副作用。
pub fn select(
    config: &crate::jev_config::JevConfig,
    port: &impl JevDecisionPort,
    fragment: &PlanFragment,
    slot_index: usize,
) -> Result<Selection, JevDecisionError> {
    validate(fragment).map_err(|_| JevDecisionError::InvalidInput)?;
    let slot = fragment.slots.get(slot_index).ok_or(JevDecisionError::InvalidInput)?;
    let mut candidates = slot.candidates.iter().map(|candidate| JevCandidate {
        id: candidate.candidate_id.clone(), dispatchable: true, parameter_complete: true,
    }).collect::<Vec<_>>();
    candidates.push(JevCandidate { id: jev_runtime::HAND_BACK.into(), dispatchable: true, parameter_complete: true });
    match jev_runtime::decide(config, port, &JevDecisionRequest { task_id: fragment.task_id.clone(), step_id: slot.step_id.clone(), capability: JevCapability::Cua, candidates })? {
        JevDecision::Dispatch { candidate_id } => Ok(Selection::Dispatch(slot.candidates.iter().find(|candidate| candidate.candidate_id == candidate_id).expect("Jev decision only returns submitted candidate").clone())),
        JevDecision::HandBack { reason } => Ok(Selection::HandBack { reason }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jev_config::{JEV_REMOTE_ENDPOINT, JevConfig, JevServiceMode};
    use crate::jev_runtime::JevModelChoice;
    struct Fake;
    impl JevDecisionPort for Fake { fn choose(&self, _: &JevConfig, _: &JevDecisionRequest) -> Result<JevModelChoice, JevDecisionError> { Ok(JevModelChoice { candidate_id: "click".into(), confidence: 0.9 }) } }
    fn fragment() -> PlanFragment { PlanFragment { plan_id:"plan-1".into(), plan_version:1, task_id:"task-1".into(), expected_sequence:1, deadline_ms:1, token_budget:1, slots:vec![PlanSlot { step_id:"step-1".into(), label:"点击继续".into(), candidates:vec![CandidateAction { candidate_id:"click".into(), tool_name:"computer_click".into(), arguments_json:"{}".into() }] }] } }
    fn config() -> JevConfig { JevConfig { enabled:true, service_mode:JevServiceMode::Remote, endpoint:JEV_REMOTE_ENDPOINT.into(), step_limit:10, time_limit_ms:60_000, token_limit:10_000, capabilities:vec![JevCapability::Cua] } }
    #[test] fn selects_only_submitted_candidate() { assert_eq!(select(&config(), &Fake, &fragment(), 0).unwrap(), Selection::Dispatch(fragment().slots[0].candidates[0].clone())); }
    #[test] fn rejects_free_form_or_oversized_fragment() { let mut value=fragment(); value.slots[0].candidates[0].tool_name="Shell".into(); assert_eq!(validate(&value),Err(crate::Error::InvalidInput)); value=fragment(); value.slots=vec![]; assert_eq!(validate(&value),Err(crate::Error::InvalidInput)); }
}
