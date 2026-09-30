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
    pub action_kind: yonder_protocol::CuaActionKind,
    pub target_ref: String,
    pub preconditions: Vec<yonder_protocol::CuaObserveConditionParams>,
    pub expected_observe: Vec<yonder_protocol::CuaObserveConditionParams>,
    pub confirmation_ref: Option<String>,
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

pub fn submit(
    store: &mut impl crate::TaskStore,
    auth: crate::AuthContext<'_>,
    params: &yonder_protocol::PlanSubmitParams,
) -> Result<crate::Task, crate::Error> {
    if !matches!(auth, crate::AuthContext::Agent(_)) || auth.agent_id() != params.agent_id {
        return Err(crate::Error::PermissionDenied);
    }
    let fragment = PlanFragment {
        plan_id: params.plan_id.clone(), plan_version: params.plan_version, task_id: params.task_id.clone(),
        expected_sequence: yonder_protocol::sequence(&params.expected_sequence).map_err(|_| crate::Error::InvalidInput)?,
        deadline_ms: params.deadline, token_budget: params.token_budget,
        slots: params.slots.iter().map(|slot| PlanSlot { step_id:slot.step_id.clone(), label:slot.label.clone(), candidates:slot.candidates.iter().map(|candidate| CandidateAction { candidate_id:candidate.candidate_id.clone(), tool_name:candidate.tool_name.clone(), arguments_json:serde_json::to_string(&candidate.arguments).expect("protocol JSON value serializes"), action_kind:candidate.action_kind, target_ref:candidate.target_ref.clone(), preconditions:candidate.preconditions.clone(), expected_observe:candidate.expected_observe.clone(), confirmation_ref:candidate.confirmation_ref.clone() }).collect() }).collect(),
    };
    validate(&fragment)?;
    store.submit_plan_fragment(auth.agent_id(), &fragment)
}

/// 可信 Gateway 会话执行一个槽位。连续执行必须由 [`execute_available`] 调用，
/// 以保证每轮都重新读取计划、任务序列和控制事实。
pub fn execute_one(
    store: &mut impl crate::TaskStore, admission: &crate::admission::Admission,
    computer: &(impl crate::computer_use::ComputerUsePort + ?Sized), targets: &(impl crate::computer_use::WorkTargetPort + ?Sized),
    config: &crate::jev_config::JevConfig, jev: &(impl JevDecisionPort + ?Sized), intents: Option<&crate::cua_intent::CuaIntentRegistry>,
    auth: crate::AuthContext<'_>, task_id: &str, plan_id: &str, plan_version: u64, expected: u64, now_ms: u64, host_session_id: &str,
) -> Result<(crate::Task, &'static str, Option<crate::computer_use::ComputerObservation>), crate::Error> {
    if !matches!(auth, crate::AuthContext::Agent(_)) { return Err(crate::Error::PermissionDenied); }
    if computer.explicit_takeover_requested(task_id) { return Ok((store.get(task_id)?, "takeover-requested", None)); }
    let stored=store.get_plan_fragment(task_id,plan_id,plan_version)?.ok_or(crate::Error::NotFound)?;
    if stored.owner_agent_id != auth.agent_id() || stored.fragment.deadline_ms <= now_ms || expected != store.get(task_id)?.sequence { return Err(crate::Error::Conflict); }
    let index=usize::from(stored.current_slot);
    if index >= stored.fragment.slots.len() { return Ok((store.get(task_id)?, "fragment-complete", None)); }
    let slot=&stored.fragment.slots[index];
    computer.project_decision(task_id,&slot.step_id,&slot.label,&format!("正在评估 {} 个受支持候选",slot.candidates.len()));
    let choice=select(config,jev,&stored.fragment,index).map_err(|_|crate::Error::StopRequired)?;
    let Selection::Dispatch(action)=choice else {
        computer.project_decision(task_id,&slot.step_id,&slot.label,"已交回慢脑重新观察或规划");
        let task=store.hand_back_plan_fragment(task_id,plan_id,plan_version,expected,"需要慢脑重新 Observe 或规划")?;
        return Ok((task, "handback", None));
    };
    let decision = if slot.candidates.len()==1 {
        format!("慢脑单候选直接授权：{}",action_kind_label(action.action_kind))
    } else {
        format!("Jev 已从 {} 个候选中选择：{}",slot.candidates.len(),action_kind_label(action.action_kind))
    };
    computer.project_decision(task_id,&slot.step_id,&slot.label,&decision);
    let current_task=store.get(task_id)?;
    let mut arguments:serde_json::Value=serde_json::from_str(&action.arguments_json).map_err(|_|crate::Error::InvalidInput)?;
    let fields=arguments.as_object_mut().ok_or(crate::Error::InvalidInput)?;
    let semantic=match action.action_kind {
        yonder_protocol::CuaActionKind::FocusTargetSearch=>Some(("focus-target-search",Some(crate::cua_intent::CuaIntentText::Target))),
        yonder_protocol::CuaActionKind::EnterTargetQuery=>Some(("enter-target-query",Some(crate::cua_intent::CuaIntentText::Target))),
        yonder_protocol::CuaActionKind::ActivateTarget=>Some(("activate-target",Some(crate::cua_intent::CuaIntentText::Target))),
        yonder_protocol::CuaActionKind::FocusMessageComposer=>Some(("focus-message-composer",None)),
        yonder_protocol::CuaActionKind::DraftMessageRef=>Some(("draft-message-ref",Some(crate::cua_intent::CuaIntentText::Message))),
        yonder_protocol::CuaActionKind::SendMessage=>Some(("send-message",None)),
        _=>None,
    };
    if let Some((kind,text))=semantic {
        let registry=intents.ok_or(crate::Error::StorageUnavailable)?;
        fields.insert("_yonder_action_kind".into(),serde_json::Value::String(kind.into()));
        if let Some(text)=text {
            let private=registry.resolve_text(auth,&current_task,&action.target_ref,text,now_ms).map_err(cua_intent_error)?;
            fields.insert("_yonder_private_text".into(),serde_json::Value::String(private));
        }
    }
    // 到达发送槽位才开放顶部本机确认。批准在 Driver 派发前消费；后续任何
    // unknown/超时都不会恢复引用，因此不会隐式重试副作用。
    if matches!(action.action_kind, yonder_protocol::CuaActionKind::SendMessage) {
        let registry=intents.ok_or(crate::Error::StorageUnavailable)?;
        let confirmation=action.confirmation_ref.as_deref().ok_or(crate::Error::InvalidInput)?;
        let state=registry.arm(auth,&current_task,&action.target_ref,confirmation,now_ms).map_err(cua_intent_error)?.state;
        if state != crate::cua_intent::CuaIntentState::Approved {
            computer.project_decision(task_id,&slot.step_id,&slot.label,"等待用户在顶部浮窗确认发送");
            return Ok((current_task,"awaiting-confirmation",None));
        }
        registry.consume_send(auth,&current_task,&action.target_ref,confirmation,now_ms).map_err(cua_intent_error)?;
    }
    let runtime_arguments=serde_json::to_string(&arguments).map_err(|_|crate::Error::InvalidInput)?;
    let (task,result,observation)=crate::computer_use::execute_agent_step(store,admission,computer,targets,auth,task_id,expected,&slot.step_id,&slot.label,&action.tool_name,&runtime_arguments,host_session_id)?;
    if !matches!(result.conclusion,crate::AttemptConclusion::Observed { action_succeeded:true }) {
        computer.project_step_unverified(task_id,&slot.step_id);
        // user-input 会把任务原子地转为 interrupted，并已写入事件/Outbox；此时
        // 不得再把片段交回写成第二次状态迁移，否则会掩盖中断事实并触发 CAS 冲突。
        if !matches!(task.status, crate::Status::Created | crate::Status::Running) {
            return Ok((task, "handback", observation));
        }
        match store.hand_back_plan_fragment(task_id,plan_id,plan_version,task.sequence,"需要慢脑重新 Observe 或规划") {
            Ok(task) => return Ok((task,"handback",observation)),
            // 用户输入可在 unknown 结果落库后、交回事件写入前原子地中断任务。
            // 此时保留已写入的 interrupted 事实，不能用过期 CAS 再写第二个交回。
            Err(crate::Error::Conflict) => {
                let current=store.get(task_id)?;
                if !matches!(current.status, crate::Status::Created | crate::Status::Running) {
                    return Ok((current,"handback",observation));
                }
                return Err(crate::Error::Conflict);
            }
            Err(error) => return Err(error),
        }
    }
    let task=store.advance_plan_fragment(task_id,plan_id,plan_version,stored.current_slot,task.sequence)?;
    computer.project_step_completed(task_id,&slot.step_id);
    Ok((task,"advanced",observation))
}

fn action_kind_label(kind:yonder_protocol::CuaActionKind)->&'static str{match kind{
    yonder_protocol::CuaActionKind::LaunchApplication=>"打开应用",
    yonder_protocol::CuaActionKind::BringToFront=>"前置目标窗口",
    yonder_protocol::CuaActionKind::FocusTargetSearch=>"聚焦会话搜索",
    yonder_protocol::CuaActionKind::EnterTargetQuery=>"输入会话目标",
    yonder_protocol::CuaActionKind::ActivateTarget=>"打开目标会话",
    yonder_protocol::CuaActionKind::FocusMessageComposer=>"聚焦消息输入框",
    yonder_protocol::CuaActionKind::DraftMessageRef=>"填写受保护消息草稿",
    yonder_protocol::CuaActionKind::ResolveConversation=>"定位目标会话",
    yonder_protocol::CuaActionKind::DraftMessage=>"填写消息草稿",
    yonder_protocol::CuaActionKind::SendMessage=>"发送消息",
}}

/// 在一次 Gateway 调用生命周期内同步消费剩余槽位。每轮仍只产生一个副作用，
/// 且 `execute_one` 已在返回前完成 Observe 与 attempt 停止边界；下一轮因此会
/// 重新经过任务 CAS、控制检查、桌面租约与目标解析。函数返回后不留下后台执行。
pub fn execute_available(
    store: &mut impl crate::TaskStore, admission: &crate::admission::Admission,
    computer: &(impl crate::computer_use::ComputerUsePort + ?Sized), targets: &(impl crate::computer_use::WorkTargetPort + ?Sized),
    config: &crate::jev_config::JevConfig, jev: &(impl JevDecisionPort + ?Sized), intents: Option<&crate::cua_intent::CuaIntentRegistry>,
    auth: crate::AuthContext<'_>, task_id: &str, plan_id: &str, plan_version: u64, expected: u64, now_ms: u64, host_session_id: &str,
) -> Result<(crate::Task, &'static str, Option<crate::computer_use::ComputerObservation>), crate::Error> {
    let initial = store.get_plan_fragment(task_id, plan_id, plan_version)?.ok_or(crate::Error::NotFound)?;
    if initial.owner_agent_id != auth.agent_id()
        || initial.fragment.deadline_ms <= now_ms
        || expected != store.get(task_id)?.sequence
    {
        return Err(crate::Error::Conflict);
    }
    let remaining = initial.fragment.slots.len().saturating_sub(usize::from(initial.current_slot));
    if remaining == 0 {
        return Ok((store.get(task_id)?, "fragment-complete", None));
    }
    let allowed = remaining.min(usize::try_from(config.step_limit).unwrap_or(usize::MAX));
    let mut sequence = expected;
    for _ in 0..allowed {
        if computer.explicit_takeover_requested(task_id) {
            return Ok((store.get(task_id)?, "takeover-requested", None));
        }
        let (task, disposition, observation) = execute_one(
            store, admission, computer, targets, config, jev, intents, auth, task_id, plan_id,
            plan_version, sequence, now_ms, host_session_id,
        )?;
        sequence = task.sequence;
        if disposition != "advanced" {
            return Ok((task, disposition, observation));
        }
        if computer.explicit_takeover_requested(task_id) {
            return Ok((task, "takeover-requested", None));
        }
        let current = store.get_plan_fragment(task_id, plan_id, plan_version)?.ok_or(crate::Error::NotFound)?;
        if usize::from(current.current_slot) >= current.fragment.slots.len() {
            return Ok((task, "fragment-complete", observation));
        }
    }
    let task = store.hand_back_plan_fragment(
        task_id, plan_id, plan_version, sequence, "计划片段步数预算已耗尽",
    )?;
    Ok((task, "handback", None))
}

fn cua_intent_error(error:crate::cua_intent::CuaIntentError)->crate::Error{match error{
    crate::cua_intent::CuaIntentError::InvalidInput=>crate::Error::InvalidInput,
    crate::cua_intent::CuaIntentError::PermissionDenied=>crate::Error::PermissionDenied,
    crate::cua_intent::CuaIntentError::NotFound|crate::cua_intent::CuaIntentError::Expired=>crate::Error::NotFound,
    crate::cua_intent::CuaIntentError::Rejected=>crate::Error::StopRequired,
    crate::cua_intent::CuaIntentError::Capacity|crate::cua_intent::CuaIntentError::Unavailable=>crate::Error::StorageUnavailable,
}}

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
        && valid_id(&candidate.target_ref)
        && !candidate.preconditions.is_empty()
        && !candidate.expected_observe.is_empty()
        && candidate.preconditions.len() <= 4
        && candidate.expected_observe.len() <= 4
        && valid_observe_conditions(&candidate.preconditions)
        && valid_observe_conditions(&candidate.expected_observe)
        && yonder_protocol::tool_matches_action_kind(candidate.action_kind, &candidate.tool_name)
        && match candidate.action_kind {
            yonder_protocol::CuaActionKind::SendMessage => candidate.confirmation_ref.as_deref().is_some_and(valid_id) && semantic_arguments(candidate.action_kind,&candidate.tool_name,&candidate.arguments_json),
            yonder_protocol::CuaActionKind::FocusTargetSearch | yonder_protocol::CuaActionKind::EnterTargetQuery | yonder_protocol::CuaActionKind::ActivateTarget | yonder_protocol::CuaActionKind::FocusMessageComposer | yonder_protocol::CuaActionKind::DraftMessageRef => candidate.confirmation_ref.is_none() && semantic_arguments(candidate.action_kind,&candidate.tool_name,&candidate.arguments_json),
            _ => candidate.confirmation_ref.as_deref().is_none_or(valid_id),
        }
}

fn semantic_arguments(kind:yonder_protocol::CuaActionKind,tool_name:&str,value:&str)->bool{serde_json::from_str::<serde_json::Value>(value).is_ok_and(|value|value.as_object().is_some_and(|arguments|arguments.is_empty()||(arguments.len()==2&&arguments.get("x").and_then(serde_json::Value::as_f64).is_some_and(f64::is_finite)&&arguments.get("y").and_then(serde_json::Value::as_f64).is_some_and(f64::is_finite)&&((tool_name=="click"&&matches!(kind,yonder_protocol::CuaActionKind::FocusTargetSearch|yonder_protocol::CuaActionKind::ActivateTarget|yonder_protocol::CuaActionKind::FocusMessageComposer|yonder_protocol::CuaActionKind::SendMessage))||(tool_name=="type_text"&&matches!(kind,yonder_protocol::CuaActionKind::EnterTargetQuery|yonder_protocol::CuaActionKind::DraftMessageRef))))))}

fn valid_observe_conditions(conditions: &[yonder_protocol::CuaObserveConditionParams]) -> bool {
    let mut facts = std::collections::HashSet::new();
    conditions.iter().all(|condition| facts.insert(condition.fact as u8))
}

/// 从指定槽位作一次有界选择。调用方必须在持久化读取、CAS、控制检查及
/// 当前 Observe 检查后调用；返回 HandBack 不会产生副作用。
pub fn select(
    config: &crate::jev_config::JevConfig,
    port: &(impl JevDecisionPort + ?Sized),
    fragment: &PlanFragment,
    slot_index: usize,
) -> Result<Selection, JevDecisionError> {
    validate(fragment).map_err(|_| JevDecisionError::InvalidInput)?;
    let slot = fragment.slots.get(slot_index).ok_or(JevDecisionError::InvalidInput)?;
    // 慢脑只给出一个已验证候选时，它已经是确定性执行授权；再让 Jev 在
    // “执行/交回”之间二次规划只会增加时延并制造无理由交回。
    if slot.candidates.len() == 1 {
        return Ok(Selection::Dispatch(slot.candidates[0].clone()));
    }
    let mut candidates = slot.candidates.iter().map(|candidate| JevCandidate {
        id: candidate.candidate_id.clone(), dispatchable: true, parameter_complete: true,
        action_kind: format!("{:?}", candidate.action_kind), target_ref: candidate.target_ref.clone(),
        preconditions: candidate.preconditions.iter().map(|condition| format!("{:?}={}", condition.fact, condition.expected)).collect(),
        expected_observe: candidate.expected_observe.iter().map(|condition| format!("{:?}={}", condition.fact, condition.expected)).collect(),
    }).collect::<Vec<_>>();
    candidates.push(JevCandidate { id: jev_runtime::HAND_BACK.into(), dispatchable: true, parameter_complete: true, action_kind:"handback".into(), target_ref:"none".into(), preconditions:vec![], expected_observe:vec![] });
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
    fn candidate() -> CandidateAction { CandidateAction { candidate_id:"click".into(), tool_name:"bring_to_front".into(), arguments_json:"{}".into(), action_kind:yonder_protocol::CuaActionKind::BringToFront, target_ref:"wecom-app".into(), preconditions:vec![yonder_protocol::CuaObserveConditionParams { fact:yonder_protocol::CuaObserveFact::ApplicationReady, expected:true }], expected_observe:vec![yonder_protocol::CuaObserveConditionParams { fact:yonder_protocol::CuaObserveFact::TargetResolved, expected:true }], confirmation_ref:None } }
    fn fragment() -> PlanFragment { PlanFragment { plan_id:"plan-1".into(), plan_version:1, task_id:"task-1".into(), expected_sequence:1, deadline_ms:1, token_budget:1, slots:vec![PlanSlot { step_id:"step-1".into(), label:"点击继续".into(), candidates:vec![candidate()] }] } }
    fn config() -> JevConfig { JevConfig { enabled:true, service_mode:JevServiceMode::Remote, endpoint:JEV_REMOTE_ENDPOINT.into(), step_limit:10, time_limit_ms:60_000, token_limit:10_000, capabilities:vec![JevCapability::Cua] } }
    #[test] fn selects_only_submitted_candidate() { assert_eq!(select(&config(), &Fake, &fragment(), 0).unwrap(), Selection::Dispatch(fragment().slots[0].candidates[0].clone())); }
    struct MustNotChoose;
    impl JevDecisionPort for MustNotChoose { fn choose(&self, _: &JevConfig, _: &JevDecisionRequest) -> Result<JevModelChoice, JevDecisionError> { panic!("单一路径不得请求 Jev") } }
    #[test] fn dispatches_a_single_verified_slow_brain_step_without_jev() { assert_eq!(select(&config(), &MustNotChoose, &fragment(), 0).unwrap(), Selection::Dispatch(candidate())); }
    #[test] fn rejects_free_form_or_oversized_fragment() { let mut value=fragment(); value.slots[0].candidates[0].tool_name="Shell".into(); assert_eq!(validate(&value),Err(crate::Error::InvalidInput)); value=fragment(); value.slots=vec![]; assert_eq!(validate(&value),Err(crate::Error::InvalidInput)); }
    #[test] fn send_candidate_requires_opaque_confirmation_and_no_body() { let mut value=fragment(); value.slots[0].candidates[0].action_kind=yonder_protocol::CuaActionKind::SendMessage; value.slots[0].candidates[0].tool_name="press_key".into(); value.slots[0].candidates[0].confirmation_ref=Some("confirm-1".into()); assert_eq!(validate(&value),Ok(())); value.slots[0].candidates[0].arguments_json=r#"{\"text\":\"hi\"}"#.into(); assert_eq!(validate(&value),Err(crate::Error::InvalidInput)); }
    #[test] fn coordinates_are_limited_to_semantic_clicks_and_protected_text() { let mut value=fragment(); { let candidate=&mut value.slots[0].candidates[0];candidate.action_kind=yonder_protocol::CuaActionKind::EnterTargetQuery;candidate.tool_name="type_text".into();candidate.target_ref="intent-1".into();candidate.arguments_json=r#"{"x":10,"y":20}"#.into(); } assert_eq!(validate(&value),Ok(()));value.slots[0].candidates[0].arguments_json=r#"{"x":10,"y":20,"delivery_mode":"foreground"}"#.into();assert_eq!(validate(&value),Err(crate::Error::InvalidInput));{ let candidate=&mut value.slots[0].candidates[0];candidate.action_kind=yonder_protocol::CuaActionKind::FocusTargetSearch;candidate.tool_name="click".into();candidate.arguments_json=r#"{"x":10,"y":20}"#.into(); } assert_eq!(validate(&value),Ok(())); }
}
