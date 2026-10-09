//! EX-S5 已验证执行经验：只保存可跨任务复用的非敏感策略元数据。
use crate::{Error, plan_fragment::{CandidateAction, StoredPlanFragment}};

pub const MAX_RECALLED_EXPERIENCES: usize = 3;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionExperience {
    pub platform: String,
    pub driver: String,
    pub application_id: String,
    pub action_kind: String,
    pub control_role: String,
    pub interaction: String,
    pub precondition: String,
    pub success_condition: String,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_verified_at_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExperienceQuery {
    pub platform: String,
    pub driver: String,
    pub application_id: String,
    pub action_kind: Option<String>,
    pub control_role: Option<String>,
}

pub trait ExecutionExperienceStore {
    fn record_verified_experience(&mut self, value: &ExecutionExperience) -> Result<(), Error>;
    fn recall_experiences(&mut self, query: &ExperienceQuery, limit: usize) -> Result<Vec<ExecutionExperience>, Error>;
}

fn action_kind(value: yonder_protocol::CuaActionKind) -> &'static str {
    use yonder_protocol::CuaActionKind::*;
    match value {
        LaunchApplication => "launch-application", BringToFront => "bring-to-front",
        FocusControl => "focus-control", InputText => "input-text", ActivateControl => "activate-control",
        FocusTargetSearch => "focus-target-search", EnterTargetQuery => "enter-target-query",
        ActivateTarget => "activate-target", FocusMessageComposer => "focus-message-composer",
        DraftMessageRef => "draft-message-ref", ResolveConversation => "resolve-conversation",
        DraftMessage => "draft-message", SendMessage => "send-message",
    }
}

fn fact(value: yonder_protocol::CuaObserveFact) -> &'static str {
    use yonder_protocol::CuaObserveFact::*;
    match value { ActionSucceeded=>"action-succeeded",ObserveValid=>"observe-valid",TargetVisible=>"target-visible",ApplicationReady=>"application-ready",TargetResolved=>"target-resolved",ComposerReady=>"composer-ready",DeliveryConfirmed=>"delivery-confirmed" }
}

fn conditions(values:&[yonder_protocol::CuaObserveConditionParams])->String{
    values.iter().map(|value|format!("{}={}",fact(value.fact),value.expected)).collect::<Vec<_>>().join(",")
}

fn valid_application_id(value:&str)->bool{
    (3..=128).contains(&value.len()) && value.contains('.') && value.bytes().all(|byte|byte.is_ascii_alphanumeric()||matches!(byte,b'.'|b'-'|b'_'))
}

fn application_id(fragment:&StoredPlanFragment)->Option<String>{
    fragment.fragment.slots.iter().flat_map(|slot|slot.candidates.iter()).find_map(|candidate|{
        let arguments:serde_json::Value=serde_json::from_str(&candidate.arguments_json).ok()?;
        let value=arguments.get("bundle_id")?.as_str()?;
        valid_application_id(value).then(||value.to_owned())
    })
}

fn safe_strategy(candidate:&CandidateAction)->Option<(&'static str,&'static str)>{
    use yonder_protocol::CuaActionKind::*;
    let arguments:serde_json::Value=serde_json::from_str(&candidate.arguments_json).ok()?;
    let object=arguments.as_object()?;
    match candidate.action_kind {
        LaunchApplication=>Some(("application","native-launch")),
        BringToFront=>Some(("window","verified-window-activation")),
        FocusControl if candidate.tool_name=="click"=>Some(("control","ax-focus")),
        ActivateControl if candidate.tool_name=="click"=>{
            if object.contains_key("x")||object.contains_key("y") {
                Some(("control",if object.get("click_count").and_then(serde_json::Value::as_u64)==Some(2){"visual-double-click"}else{"visual-click"}))
            } else if object.contains_key("observed_element_index") { Some(("control","fresh-ax-element-click")) }
            else { Some(("control","semantic-ax-click")) }
        }
        ActivateControl if candidate.tool_name=="press_key"=>Some(("control","bounded-key-activation")),
        _=>None,
    }
}

/// 只有归属慢脑已经核验整个目标 achieved 后才可调用；坐标、标签、目标引用和正文不会进入返回值。
pub fn verified_experiences(fragment:&StoredPlanFragment,platform:&str,driver:&str,verified_at_ms:u64)->Vec<ExecutionExperience>{
    let Some(application_id)=application_id(fragment) else{return Vec::new();};
    fragment.fragment.slots.iter().filter_map(|slot|{
        let candidate=(slot.candidates.len()==1).then(||&slot.candidates[0])?;
        let (control_role,interaction)=safe_strategy(candidate)?;
        Some(ExecutionExperience{platform:platform.into(),driver:driver.into(),application_id:application_id.clone(),action_kind:action_kind(candidate.action_kind).into(),control_role:control_role.into(),interaction:interaction.into(),precondition:conditions(&candidate.preconditions),success_condition:conditions(&candidate.expected_observe),success_count:1,failure_count:0,last_verified_at_ms:verified_at_ms})
    }).collect()
}

pub fn recall_query(fragment:&StoredPlanFragment,platform:&str,driver:&str)->Option<ExperienceQuery>{
    let application_id=application_id(fragment)?;
    let candidate=fragment.fragment.slots.get(usize::from(fragment.current_slot)).and_then(|slot|(slot.candidates.len()==1).then(||&slot.candidates[0]));
    let strategy=candidate.and_then(safe_strategy);
    Some(ExperienceQuery{platform:platform.into(),driver:driver.into(),application_id,action_kind:candidate.and_then(|value|strategy.map(|_|action_kind(value.action_kind).into())),control_role:strategy.map(|value|value.0.into())})
}

pub fn replan_hint(value:&ExecutionExperience)->String{
    format!("策略={}；适用前置={}；成功条件={}；历史验证={}次",value.interaction,if value.precondition.is_empty(){"无额外条件"}else{&value.precondition},if value.success_condition.is_empty(){"必须重新 Observe"}else{&value.success_condition},value.success_count)
}

#[cfg(test)]
mod tests{
    use super::*;use crate::plan_fragment::{PlanFragment,PlanSlot};
    fn candidate(kind:yonder_protocol::CuaActionKind,arguments:&str)->CandidateAction{CandidateAction{candidate_id:"c1".into(),tool_name:"click".into(),arguments_json:arguments.into(),action_kind:kind,target_ref:"opaque".into(),preconditions:vec![yonder_protocol::CuaObserveConditionParams{fact:yonder_protocol::CuaObserveFact::ObserveValid,expected:true}],expected_observe:vec![yonder_protocol::CuaObserveConditionParams{fact:yonder_protocol::CuaObserveFact::ActionSucceeded,expected:true}],confirmation_ref:None}}
    fn fragment(last:CandidateAction)->StoredPlanFragment{StoredPlanFragment{owner_agent_id:"a1".into(),accepted_sequence:2,current_slot:1,fragment:PlanFragment{plan_id:"p1".into(),plan_version:1,task_id:"t1".into(),expected_sequence:1,deadline_ms:10,token_budget:10,slots:vec![PlanSlot{step_id:"launch".into(),label:"打开".into(),candidates:vec![CandidateAction{tool_name:"launch_app".into(),action_kind:yonder_protocol::CuaActionKind::LaunchApplication,arguments_json:r#"{"bundle_id":"com.example.Music"}"#.into(),..candidate(yonder_protocol::CuaActionKind::LaunchApplication,"{}")}]},PlanSlot{step_id:"play".into(),label:"播放".into(),candidates:vec![last]}]}}}
    #[test]fn strips_visual_coordinates_but_keeps_strategy(){let stored=fragment(candidate(yonder_protocol::CuaActionKind::ActivateControl,r#"{"x":700,"y":780,"click_count":2,"observation_ref":"obs_1"}"#));let values=verified_experiences(&stored,"macos","sky",100);let play=values.iter().find(|value|value.action_kind=="activate-control").unwrap();assert_eq!(play.interaction,"visual-double-click");let hint=replan_hint(play);assert!(!hint.contains("700")&&!hint.contains("780")&&!hint.contains("obs_1"));}
    #[test]fn sensitive_action_is_not_promoted(){let stored=fragment(candidate(yonder_protocol::CuaActionKind::SendMessage,"{}"));assert_eq!(verified_experiences(&stored,"macos","sky",100).len(),1);}
    #[test]fn query_targets_current_safe_action(){let stored=fragment(candidate(yonder_protocol::CuaActionKind::ActivateControl,"{}"));let query=recall_query(&stored,"macos","sky").unwrap();assert_eq!(query.application_id,"com.example.Music");assert_eq!(query.action_kind.as_deref(),Some("activate-control"));}
}
