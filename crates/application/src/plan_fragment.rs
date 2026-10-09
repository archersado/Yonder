//! EX-S2 受限计划片段的 Application 规则。
//!
//! 此模块只接受已由 Gateway 鉴权的不可变 CUA 候选；它不解析外部 JSON、
//! 不构造 Agent 身份，也不派发 Driver。实际副作用仍必须经 computer_use
//! 的统一启动、桌面租约和 Observe 链路。
use crate::{jev_config::JevCapability, jev_runtime::{self, JevCandidate, JevDecision, JevDecisionError, JevDecisionPort, JevDecisionRequest, JevStepDecision, JevStepVerificationRequest}, valid_id};

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
    Dispatch { action: CandidateAction, via_jev: bool },
    HandBack { reason: &'static str },
}

/// 当前 Gateway 调用内由 Driver 结果直接取得的结构事实；不解释业务寓意，
/// 不包含界面原文、坐标或正文。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ObservedFacts { action_succeeded:Option<bool>, observe_valid:Option<bool>, target_visible:Option<bool> }
impl ObservedFacts {
    fn get(self,fact:yonder_protocol::CuaObserveFact)->Option<bool>{match fact{
        yonder_protocol::CuaObserveFact::ActionSucceeded=>self.action_succeeded,
        yonder_protocol::CuaObserveFact::ObserveValid=>self.observe_valid,
        yonder_protocol::CuaObserveFact::TargetVisible=>self.target_visible,
        yonder_protocol::CuaObserveFact::ApplicationReady|yonder_protocol::CuaObserveFact::TargetResolved|yonder_protocol::CuaObserveFact::ComposerReady|yonder_protocol::CuaObserveFact::DeliveryConfirmed=>None,
    }}
    fn from_driver(conclusion:&crate::AttemptConclusion,observation:Option<&crate::computer_use::ComputerObservation>)->Self{
        match conclusion {
            crate::AttemptConclusion::Observed{action_succeeded}=>Self{action_succeeded:Some(*action_succeeded),observe_valid:Some(true),target_visible:observation.and_then(|value|value.target_visible)},
            crate::AttemptConclusion::Unknown{..}=>Self{action_succeeded:None,observe_valid:Some(false),target_visible:observation.and_then(|value|value.target_visible)},
        }
    }
    fn satisfies(self,conditions:&[yonder_protocol::CuaObserveConditionParams])->bool{conditions.iter().all(|condition|self.get(condition.fact)==Some(condition.expected))}
    fn labels(self)->Vec<String>{[("ActionSucceeded",self.action_succeeded),("ObserveValid",self.observe_valid),("TargetVisible",self.target_visible)].into_iter().filter_map(|(name,value)|value.map(|value|format!("{name}={value}"))).collect()}
}

/// 只在一次 Gateway 执行调用中存活的新鲜 Observe；临时句柄不进入持久化。
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ObservedContext { facts:ObservedFacts, observation_ref:Option<String>, elements:Vec<yonder_protocol::ObservedElement> }

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
fn execute_one(
    store: &mut impl crate::TaskStore, admission: &crate::admission::Admission,
    runtime:Option<&crate::execution_runtime::ExecutionRuntimeHandle>,runtime_plan:Option<&StoredPlanFragment>,
    computer: &(impl crate::computer_use::ComputerUsePort + ?Sized), targets: &(impl crate::computer_use::WorkTargetPort + ?Sized),
    config: &crate::jev_config::JevConfig, jev: &(impl JevDecisionPort + ?Sized), intents: Option<&crate::cua_intent::CuaIntentRegistry>,
    auth: crate::AuthContext<'_>, task_id: &str, plan_id: &str, plan_version: u64, expected: u64, now_ms: u64, host_session_id: &str, observed_context:&mut Option<ObservedContext>,
) -> Result<(crate::Task, &'static str, Option<crate::computer_use::ComputerObservation>), crate::Error> {
    if !matches!(auth, crate::AuthContext::Agent(_)) { return Err(crate::Error::PermissionDenied); }
    let stored=runtime_plan.cloned().map_or_else(||store.get_plan_fragment(task_id,plan_id,plan_version),|value|Ok(Some(value)))?.ok_or(crate::Error::NotFound)?;
    let persisted_task=store.get(task_id)?;
    let current_task=if let Some(runtime)=runtime{let snapshot=runtime.snapshot(task_id).map_err(runtime_error)?;let cursor=snapshot.active_plan.as_ref().ok_or(crate::Error::Conflict)?;if cursor.plan_id!=plan_id||cursor.plan_version!=plan_version{return Err(crate::Error::Conflict);}crate::Task{status:runtime_status(&snapshot.phase),sequence:snapshot.sequence,..persisted_task}}else{persisted_task};
    if computer.explicit_takeover_requested(task_id) { return Ok((current_task, "takeover-requested", None)); }
    if stored.owner_agent_id != auth.agent_id() || stored.fragment.deadline_ms <= now_ms || expected != current_task.sequence { return Err(crate::Error::Conflict); }
    let current_slot=runtime.and_then(|runtime|runtime.snapshot(task_id).ok()).and_then(|snapshot|snapshot.active_plan.map(|cursor|cursor.current_slot)).unwrap_or(stored.current_slot);
    let index=usize::from(current_slot);
    if index >= stored.fragment.slots.len() { return Ok((store.get(task_id)?, "awaiting-goal-verification", None)); }
    let slot=&stored.fragment.slots[index];
    let observed_element_count=if slot.candidates.len()==1&&is_observed_element_template(&slot.candidates[0]){observed_context.as_ref().map(|value|value.elements.iter().filter(|element|actionable_element(element.role)).take(jev_runtime::MAX_CANDIDATES).count())}else{None};
    let choice_count=observed_element_count.unwrap_or(slot.candidates.len());
    let evaluating=observed_element_count.map_or_else(||format!("正在评估 {choice_count} 个受支持候选"),|_|format!("正在依据最新 Observe 评估 {choice_count} 个控件候选"));
    computer.project_decision(task_id,&slot.step_id,&slot.label,&evaluating);
    let task_goal=current_task.name.as_deref();
    let choice=match select(config,jev,&stored.fragment,index,observed_context.as_ref(),task_goal){
        Ok(choice)=>choice,
        Err(_)=>{
            computer.project_decision(task_id,&slot.step_id,&slot.label,"快脑决策不可用，已交回慢脑");
            let task=hand_back(store,runtime,&current_task,task_id,plan_id,plan_version,expected,"快脑决策不可用，需要慢脑重新 Observe 或规划")?;
            return Ok((task,"handback",None));
        }
    };
    let Selection::Dispatch { action,via_jev }=choice else {
        let Selection::HandBack { reason }=choice else { unreachable!() };
        let (projection,handoff)=match reason{
            "low-confidence"=>("快脑置信度不足，已交回慢脑","快脑置信度不足，需要慢脑重新 Observe 或规划"),
            "preconditions-not-observed"=>("前置 Observe 不足，已交回慢脑","需要慢脑重新 Observe 或规划"),
            _=>("快脑未返回可派发候选，已交回慢脑","快脑未返回可派发候选，需要慢脑重新 Observe 或规划"),
        };
        computer.project_decision(task_id,&slot.step_id,&slot.label,projection);
        let task=hand_back(store,runtime,&current_task,task_id,plan_id,plan_version,expected,handoff)?;
        return Ok((task, "handback", None));
    };
    let decision = if via_jev {
        format!("Jev 已从 {choice_count} 个候选中选择：{}",action_kind_label(action.action_kind))
    } else {
        format!("唯一可执行候选直接授权：{}",action_kind_label(action.action_kind))
    };
    computer.project_decision(task_id,&slot.step_id,&slot.label,&decision);
    let mut arguments:serde_json::Value=serde_json::from_str(&action.arguments_json).map_err(|_|crate::Error::InvalidInput)?;
    let fields=arguments.as_object_mut().ok_or(crate::Error::InvalidInput)?;
    let semantic=match action.action_kind {
        yonder_protocol::CuaActionKind::FocusControl=>Some(("focus-control",None)),
        yonder_protocol::CuaActionKind::InputText=>Some(("input-text",None)),
        yonder_protocol::CuaActionKind::ActivateControl=>Some(("activate-control",None)),
        yonder_protocol::CuaActionKind::FocusTargetSearch=>Some(("focus-target-search",Some(crate::cua_intent::CuaIntentText::Target))),
        yonder_protocol::CuaActionKind::EnterTargetQuery=>Some(("enter-target-query",Some(crate::cua_intent::CuaIntentText::Target))),
        yonder_protocol::CuaActionKind::ActivateTarget=>Some(("activate-target",Some(crate::cua_intent::CuaIntentText::Target))),
        yonder_protocol::CuaActionKind::FocusMessageComposer=>Some(("focus-message-composer",None)),
        yonder_protocol::CuaActionKind::DraftMessageRef=>Some(("draft-message-ref",Some(crate::cua_intent::CuaIntentText::Message))),
        yonder_protocol::CuaActionKind::SendMessage=>Some(("send-message",Some(crate::cua_intent::CuaIntentText::Message))),
        _=>None,
    };
    if let Some((kind,text))=semantic {
        fields.insert("_yonder_action_kind".into(),serde_json::Value::String(kind.into()));
        if let Some(text)=text {
            let registry=intents.ok_or(crate::Error::StorageUnavailable)?;
            let private=registry.resolve_text(auth,&current_task,&action.target_ref,text,now_ms).map_err(cua_intent_error)?;
            fields.insert("_yonder_private_text".into(),serde_json::Value::String(private));
        }
    }
    // 到达发送槽位才开放顶部本机确认。批准在 Driver 派发前消费；后续任何
    // unknown/超时都不会恢复引用，因此不会隐式重试副作用。
    if matches!(action.action_kind, yonder_protocol::CuaActionKind::SendMessage) {
        let registry=intents.ok_or(crate::Error::StorageUnavailable)?;
        let target=registry.resolve_text(auth,&current_task,&action.target_ref,crate::cua_intent::CuaIntentText::Target,now_ms).map_err(cua_intent_error)?;
        fields.insert("_yonder_private_target".into(),serde_json::Value::String(target));
        let confirmation=action.confirmation_ref.as_deref().ok_or(crate::Error::InvalidInput)?;
        let state=registry.arm(auth,&current_task,&action.target_ref,confirmation,now_ms).map_err(cua_intent_error)?.state;
        if state != crate::cua_intent::CuaIntentState::Approved {
            computer.project_decision(task_id,&slot.step_id,&slot.label,"等待用户在顶部浮窗确认发送");
            return Ok((current_task,"awaiting-confirmation",None));
        }
        registry.consume_send(auth,&current_task,&action.target_ref,confirmation,now_ms).map_err(cua_intent_error)?;
    }
    let runtime_arguments=serde_json::to_string(&arguments).map_err(|_|crate::Error::InvalidInput)?;
    let (task,result,observation)=if let Some(runtime)=runtime{crate::computer_use::execute_agent_step_runtime(store,runtime,admission,computer,targets,auth,task_id,expected,&slot.step_id,&slot.label,&action.tool_name,&runtime_arguments,host_session_id)?}else{crate::computer_use::execute_agent_step(store,admission,computer,targets,auth,task_id,expected,&slot.step_id,&slot.label,&action.tool_name,&runtime_arguments,host_session_id)?};
    if !matches!(result.conclusion,crate::AttemptConclusion::Observed { action_succeeded:true }) {
        computer.project_step_unverified(task_id,&slot.step_id);
        // user-input 会把任务原子地转为 interrupted，并已写入事件/Outbox；此时
        // 不得再把片段交回写成第二次状态迁移，否则会掩盖中断事实并触发 CAS 冲突。
        if !matches!(task.status, crate::Status::Created | crate::Status::Running) {
            return Ok((task, "handback", observation));
        }
        match hand_back(store,runtime,&task,task_id,plan_id,plan_version,task.sequence,"需要慢脑重新 Observe 或规划") {
            Ok(task) => return Ok((task,"handback",observation)),
            // 用户输入可在 unknown 结果落库后、交回事件写入前原子地中断任务。
            // 此时保留已写入的 interrupted 事实，不能用过期 CAS 再写第二个交回。
            Err(crate::Error::Conflict) => {
                let current=runtime.map_or_else(||store.get(task_id),|runtime|runtime.snapshot(task_id).map(|snapshot|crate::Task{status:runtime_status(&snapshot.phase),sequence:snapshot.sequence,..task.clone()}).map_err(runtime_error))?;
                if !matches!(current.status, crate::Status::Created | crate::Status::Running) {
                    return Ok((current,"handback",observation));
                }
                return Err(crate::Error::Conflict);
            }
            Err(error) => return Err(error),
        }
    }
    let fresh_facts=ObservedFacts::from_driver(&result.conclusion,observation.as_ref());
    if !fresh_facts.satisfies(&action.expected_observe) {
        computer.project_step_unverified(task_id,&slot.step_id);
        let task=hand_back(store,runtime,&task,task_id,plan_id,plan_version,task.sequence,"Observe 未满足计划后置条件，需要慢脑重新规划")?;
        return Ok((task,"handback",observation));
    }
    if matches!(action.action_kind,yonder_protocol::CuaActionKind::ActivateControl) {
        let summary=step_observation_summary(&slot.label,observation.as_ref());
        computer.project_decision(task_id,&slot.step_id,&slot.label,"快脑正在核验当前步骤");
        let request=JevStepVerificationRequest{task_id:task_id.into(),step_id:slot.step_id.clone(),step_label:slot.label.clone(),capability:JevCapability::Cua,action_kind:format!("{:?}",action.action_kind),observed_facts:fresh_facts.labels(),expected_observe:action.expected_observe.iter().map(|condition|format!("{:?}={}",condition.fact,condition.expected)).collect(),observation_summary:summary};
        match jev_runtime::verify_step(config,jev,&request) {
            Ok(JevStepDecision::Complete)=>computer.project_decision(task_id,&slot.step_id,&slot.label,"快脑已确认当前步骤完成"),
            Ok(JevStepDecision::Incomplete)=>{
                computer.project_step_unverified(task_id,&slot.step_id);
                computer.project_decision(task_id,&slot.step_id,&slot.label,"快脑确认当前步骤未完成，已交回慢脑");
                let task=hand_back(store,runtime,&task,task_id,plan_id,plan_version,task.sequence,"快脑确认当前步骤未完成，需要慢脑重新规划")?;
                return Ok((task,"handback",observation));
            }
            Ok(JevStepDecision::HandBack)|Err(_)=>{
                computer.project_step_unverified(task_id,&slot.step_id);
                computer.project_decision(task_id,&slot.step_id,&slot.label,"快脑无法确认当前步骤完成，已交回慢脑");
                let task=hand_back(store,runtime,&task,task_id,plan_id,plan_version,task.sequence,"快脑无法确认当前步骤完成，需要慢脑重新规划")?;
                return Ok((task,"handback",observation));
            }
        }
    }
    let (observation_ref,elements)=observation.as_ref().map(|value|(value.observation_ref.clone(),value.elements.clone())).unwrap_or_default();
    *observed_context=Some(ObservedContext{facts:fresh_facts,observation_ref,elements});
    let task=if let Some(runtime)=runtime{let snapshot=runtime.apply(crate::execution_runtime::RuntimeCommand::AdvancePlanSlot{task_id:task_id.into(),plan_id:plan_id.into(),plan_version,expected_slot:current_slot,step_id:slot.step_id.clone()}).map_err(runtime_error)?;crate::Task{status:runtime_status(&snapshot.phase),sequence:snapshot.sequence,..task}}else{store.advance_plan_fragment(task_id,plan_id,plan_version,stored.current_slot,task.sequence)?};
    computer.project_step_completed(task_id,&slot.step_id);
    Ok((task,"advanced",observation))
}

fn runtime_status(phase:&crate::execution_runtime::RuntimePhase)->crate::Status{match phase{crate::execution_runtime::RuntimePhase::Completed=>crate::Status::Completed,crate::execution_runtime::RuntimePhase::Failed=>crate::Status::Failed,crate::execution_runtime::RuntimePhase::Cancelled=>crate::Status::Cancelled,crate::execution_runtime::RuntimePhase::Interrupted=>crate::Status::Interrupted,crate::execution_runtime::RuntimePhase::Paused=>crate::Status::Paused,_=>crate::Status::Running}}
fn runtime_error(error:crate::execution_runtime::RuntimeError)->crate::Error{match error{crate::execution_runtime::RuntimeError::InvalidInput=>crate::Error::InvalidInput,crate::execution_runtime::RuntimeError::NotFound=>crate::Error::NotFound,crate::execution_runtime::RuntimeError::Conflict=>crate::Error::Conflict,crate::execution_runtime::RuntimeError::Backpressure=>crate::Error::StopRequired,crate::execution_runtime::RuntimeError::Unavailable=>crate::Error::StorageUnavailable}}
fn hand_back(store:&mut impl crate::TaskStore,runtime:Option<&crate::execution_runtime::ExecutionRuntimeHandle>,task:&crate::Task,task_id:&str,plan_id:&str,plan_version:u64,expected:u64,reason:&str)->Result<crate::Task,crate::Error>{if let Some(runtime)=runtime{let current=runtime.snapshot(task_id).map_err(runtime_error)?;if current.sequence!=expected{return Err(crate::Error::Conflict);}let cursor=current.active_plan.as_ref().ok_or(crate::Error::Conflict)?;if cursor.plan_id!=plan_id||cursor.plan_version!=plan_version{return Err(crate::Error::Conflict);}let snapshot=runtime.apply(crate::execution_runtime::RuntimeCommand::HandBack{task_id:task_id.into(),reason:reason.into()}).map_err(runtime_error)?;Ok(crate::Task{status:crate::Status::Running,sequence:snapshot.sequence,..task.clone()})}else{store.hand_back_plan_fragment(task_id,plan_id,plan_version,expected,reason)}}

fn step_observation_summary(step_label:&str,observation:Option<&crate::computer_use::ComputerObservation>)->Vec<String>{
    const SENSITIVE:[&str;8]=["password","secure text","密码","口令","验证码","消息正文","收件人","联系人"];
    const SEMANTIC:[&str;10]=["播放","暂停","歌曲名","选中","已选择","button","selected","playing","paused","track"];
    let Some(observation)=observation else{return vec!["Driver 未返回可转移的界面文本".into()]};
    let mut needles=step_label.to_lowercase().split(|character:char|!character.is_alphanumeric()).filter(|token|token.chars().count()>=3).map(str::to_owned).collect::<Vec<_>>();
    needles.extend(SEMANTIC.into_iter().map(str::to_owned));
    let mut result=Vec::new();
    if let Some(transcript)=observation.transcript.as_deref(){for raw in transcript.lines(){let line=raw.trim();let lower=line.to_lowercase();if line.is_empty()||SENSITIVE.iter().any(|value|lower.contains(value))||!needles.iter().any(|value|lower.contains(value)){continue;}let bounded=line.chars().take(256).collect::<String>();if !result.contains(&bounded){result.push(bounded);}if result.len()==32{break;}}}
    if result.is_empty(){result.push(format!("结构观察：元素数量 {}，未提取到相关语义文本",observation.element_count));}
    result
}

fn action_kind_label(kind:yonder_protocol::CuaActionKind)->&'static str{match kind{
    yonder_protocol::CuaActionKind::LaunchApplication=>"打开应用",
    yonder_protocol::CuaActionKind::BringToFront=>"前置目标窗口",
    yonder_protocol::CuaActionKind::FocusControl=>"聚焦控件",
    yonder_protocol::CuaActionKind::InputText=>"输入文本",
    yonder_protocol::CuaActionKind::ActivateControl=>"激活控件",
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
        return Ok((store.get(task_id)?, "awaiting-goal-verification", None));
    }
    let allowed = remaining.min(usize::try_from(config.step_limit).unwrap_or(usize::MAX));
    let mut sequence = expected;
    let mut observed_context=None;
    for _ in 0..allowed {
        if computer.explicit_takeover_requested(task_id) {
            return Ok((store.get(task_id)?, "takeover-requested", None));
        }
        let (task, disposition, observation) = execute_one(
            store, admission, None, None, computer, targets, config, jev, intents, auth, task_id, plan_id,
            plan_version, sequence, now_ms, host_session_id,&mut observed_context,
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
            return Ok((task, "awaiting-goal-verification", observation));
        }
    }
    let task = store.hand_back_plan_fragment(
        task_id, plan_id, plan_version, sequence, "计划片段步数预算已耗尽",
    )?;
    Ok((task, "handback", None))
}

/// 由单一内存运行时连续消费计划槽位；SQLite 仅在入口读取不可变片段，
/// Driver 结果、片段游标与交回先形成连续事件，再异步投影。
pub fn execute_available_runtime(
    store:&mut impl crate::TaskStore,runtime:&crate::execution_runtime::ExecutionRuntimeHandle,admission:&crate::admission::Admission,
    computer:&(impl crate::computer_use::ComputerUsePort+?Sized),targets:&(impl crate::computer_use::WorkTargetPort+?Sized),config:&crate::jev_config::JevConfig,jev:&(impl JevDecisionPort+?Sized),intents:Option<&crate::cua_intent::CuaIntentRegistry>,
    auth:crate::AuthContext<'_>,task_id:&str,plan_id:&str,plan_version:u64,expected:u64,now_ms:u64,host_session_id:&str,
)->Result<(crate::Task,&'static str,Option<crate::computer_use::ComputerObservation>),crate::Error>{
    let stored=store.get_plan_fragment(task_id,plan_id,plan_version)?.ok_or(crate::Error::NotFound)?;let persisted=store.get(task_id)?;
    if stored.owner_agent_id!=auth.agent_id()||stored.fragment.deadline_ms<=now_ms{return Err(crate::Error::Conflict);}
    let (initial,newly_activated)=match runtime.snapshot(task_id){Ok(snapshot)=>(snapshot,false),Err(crate::execution_runtime::RuntimeError::NotFound)=>(runtime.apply(crate::execution_runtime::RuntimeCommand::Activate{task_id:task_id.into(),checkpoint:expected}).map_err(runtime_error)?,true),Err(error)=>return Err(runtime_error(error))};
    match initial.active_plan.as_ref(){
        Some(cursor) if cursor.plan_id==plan_id&&cursor.plan_version==plan_version=>if initial.sequence!=expected{return Err(crate::Error::Conflict)},
        Some(_)=>if initial.phase!=crate::execution_runtime::RuntimePhase::HandBack||initial.pending_events!=0||expected!=stored.accepted_sequence||expected!=persisted.sequence{return Err(crate::Error::Conflict)},
        None if !newly_activated&&(initial.sequence!=expected||expected!=persisted.sequence)=>return Err(crate::Error::Conflict),
        None if expected!=persisted.sequence=>return Err(crate::Error::Conflict),None=>{}
    }
    let bound=runtime.apply(crate::execution_runtime::RuntimeCommand::BindPlan{task_id:task_id.into(),plan_id:plan_id.into(),plan_version,current_slot:stored.current_slot,source_sequence:stored.accepted_sequence}).map_err(runtime_error)?;
    let mut task=crate::Task{status:runtime_status(&bound.phase),sequence:bound.sequence,..persisted};
    let remaining=stored.fragment.slots.len().saturating_sub(usize::from(bound.active_plan.as_ref().ok_or(crate::Error::Conflict)?.current_slot));if remaining==0{return Ok((task,"awaiting-goal-verification",None));}
    let allowed=remaining.min(usize::try_from(config.step_limit).unwrap_or(usize::MAX));let mut observed_context=None;let mut last_observation=None;
    for _ in 0..allowed{
        if computer.explicit_takeover_requested(task_id){return Ok((task,"takeover-requested",None));}
        let (next,disposition,observation)=execute_one(store,admission,Some(runtime),Some(&stored),computer,targets,config,jev,intents,auth,task_id,plan_id,plan_version,task.sequence,now_ms,host_session_id,&mut observed_context)?;task=next;last_observation=observation;
        if disposition!="advanced"{return Ok((task,disposition,last_observation));}
        let cursor=runtime.snapshot(task_id).map_err(runtime_error)?.active_plan.ok_or(crate::Error::Conflict)?;if usize::from(cursor.current_slot)>=stored.fragment.slots.len(){return Ok((task,"awaiting-goal-verification",last_observation));}
    }
    let sequence=task.sequence;task=hand_back(store,Some(runtime),&task,task_id,plan_id,plan_version,sequence,"计划片段步数预算已耗尽")?;Ok((task,"handback",last_observation))
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
        && candidate.candidate_id != jev_runtime::HAND_BACK
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
            yonder_protocol::CuaActionKind::FocusControl | yonder_protocol::CuaActionKind::InputText | yonder_protocol::CuaActionKind::ActivateControl | yonder_protocol::CuaActionKind::FocusTargetSearch | yonder_protocol::CuaActionKind::EnterTargetQuery | yonder_protocol::CuaActionKind::ActivateTarget | yonder_protocol::CuaActionKind::FocusMessageComposer | yonder_protocol::CuaActionKind::DraftMessageRef => candidate.confirmation_ref.is_none() && semantic_arguments(candidate.action_kind,&candidate.tool_name,&candidate.arguments_json),
            _ => candidate.confirmation_ref.as_deref().is_none_or(valid_id),
        }
}

fn semantic_arguments(kind:yonder_protocol::CuaActionKind,tool_name:&str,value:&str)->bool{serde_json::from_str::<serde_json::Value>(value).is_ok_and(|value|yonder_protocol::valid_plan_action_arguments(kind,tool_name,&value))}

fn valid_observe_conditions(conditions: &[yonder_protocol::CuaObserveConditionParams]) -> bool {
    let mut facts = std::collections::HashSet::new();
    conditions.iter().all(|condition| facts.insert(condition.fact as u8))
}

/// 从指定槽位作一次有界选择。调用方必须在持久化读取、CAS、控制检查及
/// 当前 Observe 检查后调用；返回 HandBack 不会产生副作用。
fn select(
    config: &crate::jev_config::JevConfig,
    port: &(impl JevDecisionPort + ?Sized),
    fragment: &PlanFragment,
    slot_index: usize,
    observed:Option<&ObservedContext>,
    task_goal:Option<&str>,
) -> Result<Selection, JevDecisionError> {
    validate(fragment).map_err(|_| JevDecisionError::InvalidInput)?;
    let slot = fragment.slots.get(slot_index).ok_or(JevDecisionError::InvalidInput)?;
    if slot.candidates.len()==1&&is_observed_element_template(&slot.candidates[0]){if let Some(observed)=observed{return select_observed_element(config,port,fragment,slot,&slot.candidates[0],observed,task_goal);}}
    let observed_facts=observed.map(|value|value.facts);
    let eligible=slot.candidates.iter().filter(|candidate|observed_facts.is_some_and(|facts|facts.satisfies(&candidate.preconditions))||(observed_facts.is_none()&&slot.candidates.len()==1)).collect::<Vec<_>>();
    if eligible.is_empty(){return Ok(Selection::HandBack{reason:"preconditions-not-observed"});}
    if eligible.len()==1{return Ok(Selection::Dispatch{action:eligible[0].clone(),via_jev:false});}
    let candidates = eligible.iter().map(|candidate| JevCandidate {
        id: candidate.candidate_id.clone(), dispatchable: true, parameter_complete: true,
        action_kind: format!("{:?}", candidate.action_kind), target_ref: candidate.target_ref.clone(),
        preconditions: candidate.preconditions.iter().map(|condition| format!("{:?}={}", condition.fact, condition.expected)).collect(),
        expected_observe: candidate.expected_observe.iter().map(|condition| format!("{:?}={}", condition.fact, condition.expected)).collect(),
        element_role:None, element_label:None,
    }).collect::<Vec<_>>();
    let task_goal=task_goal.filter(|goal|yonder_protocol::valid_task_name(goal)).ok_or(JevDecisionError::InvalidInput)?;
    let plan_steps=fragment.slots.iter().map(|slot|slot.label.clone()).collect();
    match jev_runtime::decide(config, port, &JevDecisionRequest { task_id: fragment.task_id.clone(), task_goal:task_goal.into(), plan_steps, current_step_index:u16::try_from(slot_index).map_err(|_|JevDecisionError::InvalidInput)?, step_id: slot.step_id.clone(), step_label:slot.label.clone(), capability: JevCapability::Cua, observed_facts:observed_facts.expect("多个合格候选一定有新鲜事实").labels(), candidates })? {
        JevDecision::Dispatch { candidate_id } => Ok(Selection::Dispatch{action:slot.candidates.iter().find(|candidate| candidate.candidate_id == candidate_id).expect("Jev decision only returns submitted candidate").clone(),via_jev:true}),
        JevDecision::HandBack { reason } => Ok(Selection::HandBack { reason }),
    }
}

fn is_observed_element_template(candidate:&CandidateAction)->bool{candidate.action_kind==yonder_protocol::CuaActionKind::ActivateControl&&candidate.tool_name=="click"&&serde_json::from_str::<serde_json::Value>(&candidate.arguments_json).is_ok_and(|value|value.as_object().is_some_and(serde_json::Map::is_empty))}
fn actionable_element(role:yonder_protocol::ObservedElementRole)->bool{use yonder_protocol::ObservedElementRole::*;matches!(role,Button|MenuItem|TextField|CheckBox|RadioButton|Link|Tab|ComboBox|SelectableRow|Image)}
fn bounded_element_label(value:&str)->String{let mut label=String::new();for character in value.chars(){if label.len()+character.len_utf8()>128{break;}label.push(character);}label}
fn semantic_chars(value:&str)->Vec<char>{value.chars().filter(|value|value.is_alphanumeric()).flat_map(char::to_lowercase).take(512).collect()}
fn longest_common_run(left:&[char],right:&[char])->usize{let mut previous=vec![0;right.len()+1];let mut best=0;for left_char in left{let mut current=vec![0;right.len()+1];for(index,right_char)in right.iter().enumerate(){if left_char==right_char{current[index+1]=previous[index]+1;best=best.max(current[index+1]);}}previous=current;}best}
fn element_relevance(context:&str,element:&yonder_protocol::ObservedElement)->(usize,usize){let Some(label)=element.label.as_deref().map(str::trim).filter(|value|!value.is_empty())else{return(0,0)};let context_lower=context.to_lowercase();let exact=label.split(|character:char|!character.is_alphanumeric()).filter(|token|token.chars().count()>=2).filter(|token|context_lower.contains(&token.to_lowercase())).map(|token|token.chars().count()).max().unwrap_or(0);(exact,longest_common_run(&semantic_chars(context),&semantic_chars(label)))}
fn select_observed_element(config:&crate::jev_config::JevConfig,port:&(impl JevDecisionPort + ?Sized),fragment:&PlanFragment,slot:&PlanSlot,template:&CandidateAction,observed:&ObservedContext,task_goal:Option<&str>)->Result<Selection,JevDecisionError>{
    let Some(reference)=observed.observation_ref.as_deref().filter(|value|valid_id(value))else{return Ok(Selection::HandBack{reason:"preconditions-not-observed"});};
    if !observed.facts.satisfies(&template.preconditions){return Ok(Selection::HandBack{reason:"preconditions-not-observed"});}
    let context=format!("{} {}",task_goal.unwrap_or_default(),slot.label);let mut elements=observed.elements.iter().filter(|element|actionable_element(element.role)).collect::<Vec<_>>();elements.sort_by(|left,right|element_relevance(&context,right).cmp(&element_relevance(&context,left)));
    let actions=elements.into_iter().take(jev_runtime::MAX_CANDIDATES).map(|element|{let id=format!("ax-element-{}",element.index);let mut action=template.clone();action.candidate_id=id.clone();action.arguments_json=serde_json::json!({"observed_element_index":element.index,"observation_ref":reference}).to_string();let candidate=JevCandidate{id,dispatchable:true,parameter_complete:true,action_kind:format!("{:?}",template.action_kind),target_ref:template.target_ref.clone(),preconditions:template.preconditions.iter().map(|condition|format!("{:?}={}",condition.fact,condition.expected)).collect(),expected_observe:template.expected_observe.iter().map(|condition|format!("{:?}={}",condition.fact,condition.expected)).collect(),element_role:Some(format!("{:?}",element.role)),element_label:element.label.as_deref().map(str::trim).filter(|value|!value.is_empty()).map(bounded_element_label)};(action,candidate)}).collect::<Vec<_>>();
    if actions.is_empty(){return Ok(Selection::HandBack{reason:"preconditions-not-observed"});}if actions.len()==1{return Ok(Selection::Dispatch{action:actions[0].0.clone(),via_jev:false});}
    let task_goal=task_goal.filter(|goal|yonder_protocol::valid_task_name(goal)).ok_or(JevDecisionError::InvalidInput)?;let candidates=actions.iter().map(|(_,candidate)|candidate.clone()).collect();let plan_steps=fragment.slots.iter().map(|slot|slot.label.clone()).collect();match jev_runtime::decide(config,port,&JevDecisionRequest{task_id:fragment.task_id.clone(),task_goal:task_goal.into(),plan_steps,current_step_index:u16::try_from(fragment.slots.iter().position(|value|value.step_id==slot.step_id).ok_or(JevDecisionError::InvalidInput)?).map_err(|_|JevDecisionError::InvalidInput)?,step_id:slot.step_id.clone(),step_label:slot.label.clone(),capability:JevCapability::Cua,observed_facts:observed.facts.labels(),candidates})?{JevDecision::Dispatch{candidate_id}=>Ok(Selection::Dispatch{action:actions.into_iter().find(|(_,candidate)|candidate.id==candidate_id).expect("Jev decision only returns submitted candidate").0,via_jev:true}),JevDecision::HandBack{reason}=>Ok(Selection::HandBack{reason})}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jev_config::{JEV_REMOTE_ENDPOINT, JevConfig, JevServiceMode};
    use crate::jev_runtime::JevModelChoice;
    use std::cell::RefCell;
    struct Fake;
    impl JevDecisionPort for Fake { fn choose(&self, _: &JevConfig, _: &JevDecisionRequest) -> Result<JevModelChoice, JevDecisionError> { Ok(JevModelChoice { candidate_id: "click".into(), confidence: 0.9 }) } }
    fn candidate() -> CandidateAction { CandidateAction { candidate_id:"click".into(), tool_name:"bring_to_front".into(), arguments_json:"{}".into(), action_kind:yonder_protocol::CuaActionKind::BringToFront, target_ref:"wecom-app".into(), preconditions:vec![yonder_protocol::CuaObserveConditionParams { fact:yonder_protocol::CuaObserveFact::ObserveValid, expected:true }], expected_observe:vec![yonder_protocol::CuaObserveConditionParams { fact:yonder_protocol::CuaObserveFact::ActionSucceeded, expected:true }], confirmation_ref:None } }
    fn fragment() -> PlanFragment { PlanFragment { plan_id:"plan-1".into(), plan_version:1, task_id:"task-1".into(), expected_sequence:1, deadline_ms:1, token_budget:1, slots:vec![PlanSlot { step_id:"step-1".into(), label:"点击继续".into(), candidates:vec![candidate()] }] } }
    fn config() -> JevConfig { JevConfig { enabled:true, service_mode:JevServiceMode::Remote, endpoint:JEV_REMOTE_ENDPOINT.into(), model:"jev-latest".into(), confidence_metric:crate::jev_config::JevConfidenceMetric::AnswerConfidence, confidence_threshold:0.75, step_limit:10, time_limit_ms:60_000, token_limit:10_000, capabilities:vec![JevCapability::Cua] } }
    #[test] fn selects_only_submitted_candidate() { assert_eq!(select(&config(), &Fake, &fragment(), 0,None,Some("完成测试任务")).unwrap(), Selection::Dispatch{action:fragment().slots[0].candidates[0].clone(),via_jev:false}); }
    struct MustNotChoose;
    impl JevDecisionPort for MustNotChoose { fn choose(&self, _: &JevConfig, _: &JevDecisionRequest) -> Result<JevModelChoice, JevDecisionError> { panic!("单一路径不得请求 Jev") } }
    #[test] fn dispatches_a_single_verified_slow_brain_step_without_jev() { assert_eq!(select(&config(), &MustNotChoose, &fragment(), 0,None,Some("完成测试任务")).unwrap(), Selection::Dispatch{action:candidate(),via_jev:false}); }
    struct RecordingPort { request:RefCell<Option<JevDecisionRequest>> }
    impl JevDecisionPort for RecordingPort { fn choose(&self,_:&JevConfig,request:&JevDecisionRequest)->Result<JevModelChoice,JevDecisionError>{self.request.replace(Some(request.clone()));Ok(JevModelChoice{candidate_id:"click".into(),confidence:0.9})} }
    #[test] fn multiple_candidates_require_fresh_observe(){let mut value=fragment();let mut second=candidate();second.candidate_id="click-alternative".into();value.slots[0].candidates.push(second);assert_eq!(select(&config(),&MustNotChoose,&value,0,None,Some("完成测试任务")).unwrap(),Selection::HandBack{reason:"preconditions-not-observed"});}
    #[test] fn jev_receives_driver_facts_and_only_eligible_candidates(){let mut value=fragment();let mut eligible=candidate();eligible.candidate_id="click-alternative".into();let mut ineligible=candidate();ineligible.candidate_id="not-visible".into();ineligible.preconditions=vec![yonder_protocol::CuaObserveConditionParams{fact:yonder_protocol::CuaObserveFact::TargetVisible,expected:false}];value.slots[0].candidates.extend([eligible,ineligible]);let port=RecordingPort{request:RefCell::new(None)};let observed=ObservedContext{facts:ObservedFacts{action_succeeded:Some(true),observe_valid:Some(true),target_visible:Some(true)},..ObservedContext::default()};assert!(matches!(select(&config(),&port,&value,0,Some(&observed),Some("完成测试任务")).unwrap(),Selection::Dispatch{via_jev:true,..}));let request=port.request.borrow();let request=request.as_ref().unwrap();assert_eq!(request.task_goal,"完成测试任务");assert_eq!(request.plan_steps,vec!["点击继续"]);assert_eq!(request.current_step_index,0);assert_eq!(request.step_label,"点击继续");assert_eq!(request.observed_facts,vec!["ActionSucceeded=true","ObserveValid=true","TargetVisible=true"]);assert_eq!(request.candidates.iter().map(|candidate|candidate.id.as_str()).collect::<Vec<_>>(),vec!["click","click-alternative"]);}
    #[test] fn missing_task_goal_never_reaches_jev(){let mut value=fragment();let mut second=candidate();second.candidate_id="click-alternative".into();value.slots[0].candidates.push(second);let observed=ObservedContext{facts:ObservedFacts{action_succeeded:Some(true),observe_valid:Some(true),target_visible:Some(true)},..ObservedContext::default()};assert_eq!(select(&config(),&MustNotChoose,&value,0,Some(&observed),None),Err(JevDecisionError::InvalidInput));}
    #[test] fn one_observe_eligible_candidate_bypasses_jev(){let mut value=fragment();let mut ineligible=candidate();ineligible.candidate_id="not-visible".into();ineligible.preconditions=vec![yonder_protocol::CuaObserveConditionParams{fact:yonder_protocol::CuaObserveFact::TargetVisible,expected:false}];value.slots[0].candidates.push(ineligible);let observed=ObservedContext{facts:ObservedFacts{action_succeeded:Some(true),observe_valid:Some(true),target_visible:Some(true)},..ObservedContext::default()};assert_eq!(select(&config(),&MustNotChoose,&value,0,Some(&observed),Some("完成测试任务")).unwrap(),Selection::Dispatch{action:candidate(),via_jev:false});}
    fn observed_element(index:u16,label:&str)->yonder_protocol::ObservedElement{yonder_protocol::ObservedElement{index,role:yonder_protocol::ObservedElementRole::Button,label:Some(label.into())}}
    fn element_fragment()->PlanFragment{let mut value=fragment();let candidate=&mut value.slots[0].candidates[0];candidate.action_kind=yonder_protocol::CuaActionKind::ActivateControl;candidate.tool_name="click".into();candidate.arguments_json="{}".into();value.slots[0].label="播放 One Last Kiss".into();value}
    #[test] fn one_observed_element_bypasses_jev_and_injects_fresh_handle(){let observed=ObservedContext{facts:ObservedFacts{action_succeeded:Some(true),observe_valid:Some(true),target_visible:Some(true)},observation_ref:Some("observation-7".into()),elements:vec![observed_element(3,"播放")]};let Selection::Dispatch{action,via_jev}=select(&config(),&MustNotChoose,&element_fragment(),0,Some(&observed),Some("播放指定歌曲")).unwrap()else{panic!()};assert!(!via_jev);assert_eq!(serde_json::from_str::<serde_json::Value>(&action.arguments_json).unwrap(),serde_json::json!({"observed_element_index":3,"observation_ref":"observation-7"}));}
    struct ElementPort{request:RefCell<Option<JevDecisionRequest>>}
    impl JevDecisionPort for ElementPort{fn choose(&self,_:&JevConfig,request:&JevDecisionRequest)->Result<JevModelChoice,JevDecisionError>{self.request.replace(Some(request.clone()));Ok(JevModelChoice{candidate_id:"ax-element-8".into(),confidence:0.9})}}
    #[test] fn jev_selects_from_natural_observed_element_candidates(){let observed=ObservedContext{facts:ObservedFacts{action_succeeded:Some(true),observe_valid:Some(true),target_visible:Some(true)},observation_ref:Some("observation-7".into()),elements:vec![observed_element(3,"歌曲结果"),observed_element(8,"播放 One Last Kiss")]};let port=ElementPort{request:RefCell::new(None)};let Selection::Dispatch{action,via_jev}=select(&config(),&port,&element_fragment(),0,Some(&observed),Some("在 QQ 音乐播放宇多田光版本")).unwrap()else{panic!()};assert!(via_jev);assert_eq!(serde_json::from_str::<serde_json::Value>(&action.arguments_json).unwrap(),serde_json::json!({"observed_element_index":8,"observation_ref":"observation-7"}));let request=port.request.borrow();let request=request.as_ref().unwrap();assert_eq!(request.task_goal,"在 QQ 音乐播放宇多田光版本");assert_eq!(request.plan_steps,vec!["播放 One Last Kiss"]);assert_eq!(request.current_step_index,0);assert_eq!(request.step_label,"播放 One Last Kiss");let selected=request.candidates.iter().find(|candidate|candidate.id=="ax-element-8").unwrap();assert_eq!(selected.element_role.as_deref(),Some("Button"));assert_eq!(selected.element_label.as_deref(),Some("播放 One Last Kiss"));}
    struct SearchPort{request:RefCell<Option<JevDecisionRequest>>}
    impl JevDecisionPort for SearchPort{fn choose(&self,_:&JevConfig,request:&JevDecisionRequest)->Result<JevModelChoice,JevDecisionError>{self.request.replace(Some(request.clone()));Ok(JevModelChoice{candidate_id:"ax-element-228".into(),confidence:0.9})}}
    #[test] fn cjk_relevance_keeps_late_text_field_inside_bounded_candidates(){let mut elements=(1..=12).map(|index|observed_element(index,&format!("按钮 {index}"))).collect::<Vec<_>>();elements.push(yonder_protocol::ObservedElement{index:228,role:yonder_protocol::ObservedElementRole::TextField,label:Some("文本框 搜索".into())});let observed=ObservedContext{facts:ObservedFacts{action_succeeded:Some(true),observe_valid:Some(true),target_visible:Some(true)},observation_ref:Some("observation-7".into()),elements};let mut fragment=element_fragment();fragment.slots[0].label="从当前界面选择并聚焦歌曲搜索框".into();let port=SearchPort{request:RefCell::new(None)};let Selection::Dispatch{action,via_jev}=select(&config(),&port,&fragment,0,Some(&observed),Some("在 QQ 音乐搜索歌曲")).unwrap()else{panic!()};assert!(via_jev);assert_eq!(serde_json::from_str::<serde_json::Value>(&action.arguments_json).unwrap(),serde_json::json!({"observed_element_index":228,"observation_ref":"observation-7"}));let request=port.request.borrow();let candidates=&request.as_ref().unwrap().candidates;assert_eq!(candidates.len(),jev_runtime::MAX_CANDIDATES);assert_eq!(candidates[0].id,"ax-element-228");assert_eq!(candidates[0].element_role.as_deref(),Some("TextField"));}
    #[test] fn missing_fresh_observation_handle_hands_back_without_jev(){let observed=ObservedContext{facts:ObservedFacts{action_succeeded:Some(true),observe_valid:Some(true),target_visible:Some(true)},observation_ref:None,elements:vec![observed_element(3,"播放")]};assert_eq!(select(&config(),&MustNotChoose,&element_fragment(),0,Some(&observed),Some("播放指定歌曲")).unwrap(),Selection::HandBack{reason:"preconditions-not-observed"});}
    #[test] fn action_kind_never_synthesizes_business_facts(){for kind in [yonder_protocol::CuaActionKind::ActivateControl,yonder_protocol::CuaActionKind::FocusMessageComposer,yonder_protocol::CuaActionKind::SendMessage]{let facts=ObservedFacts::from_driver(&crate::AttemptConclusion::Observed{action_succeeded:true},Some(&crate::computer_use::ComputerObservation{element_count:1,screenshot_path:None,screenshot_mime:None,screenshot_width_px:None,screenshot_height_px:None,target_visible:Some(true),observation_ref:None,transcript:None,elements:vec![]}));assert_eq!(facts.get(yonder_protocol::CuaObserveFact::ActionSucceeded),Some(true));assert_eq!(facts.get(yonder_protocol::CuaObserveFact::ObserveValid),Some(true));assert_eq!(facts.get(yonder_protocol::CuaObserveFact::TargetVisible),Some(true));assert_eq!(facts.get(yonder_protocol::CuaObserveFact::TargetResolved),None);let _=kind;}}
    #[test] fn rejects_free_form_or_oversized_fragment() { let mut value=fragment(); value.slots[0].candidates[0].tool_name="Shell".into(); assert_eq!(validate(&value),Err(crate::Error::InvalidInput)); value=fragment(); value.slots=vec![]; assert_eq!(validate(&value),Err(crate::Error::InvalidInput)); value=fragment();value.slots[0].candidates[0].candidate_id=jev_runtime::HAND_BACK.into();assert_eq!(validate(&value),Err(crate::Error::InvalidInput)); }
    #[test] fn send_candidate_requires_opaque_confirmation_and_no_body() { let mut value=fragment(); value.slots[0].candidates[0].action_kind=yonder_protocol::CuaActionKind::SendMessage; value.slots[0].candidates[0].tool_name="press_key".into(); value.slots[0].candidates[0].confirmation_ref=Some("confirm-1".into()); assert_eq!(validate(&value),Ok(())); value.slots[0].candidates[0].arguments_json=r#"{\"text\":\"hi\"}"#.into(); assert_eq!(validate(&value),Err(crate::Error::InvalidInput)); }
    #[test] fn coordinates_are_only_valid_for_bounded_semantic_actions() { let mut value=fragment(); {let candidate=&mut value.slots[0].candidates[0];candidate.action_kind=yonder_protocol::CuaActionKind::EnterTargetQuery;candidate.tool_name="type_text".into();candidate.target_ref="intent-1".into();candidate.arguments_json=r#"{"x":10,"y":20,"observation_ref":"observation-1"}"#.into();} assert_eq!(validate(&value),Ok(()));value.slots[0].candidates[0].arguments_json=r#"{"x":10,"y":20,"delivery_mode":"foreground"}"#.into();assert_eq!(validate(&value),Err(crate::Error::InvalidInput));{let candidate=&mut value.slots[0].candidates[0];candidate.action_kind=yonder_protocol::CuaActionKind::FocusTargetSearch;candidate.tool_name="click".into();candidate.arguments_json=r#"{"x":10,"y":20,"observation_ref":"observation-1"}"#.into();} assert_eq!(validate(&value),Ok(()));value.slots[0].candidates[0].arguments_json=r#"{"x":10,"y":20}"#.into();assert_eq!(validate(&value),Err(crate::Error::InvalidInput));{let candidate=&mut value.slots[0].candidates[0];candidate.tool_name="hotkey".into();candidate.arguments_json=r#"{"keys":["cmd","f"]}"#.into();} assert_eq!(validate(&value),Ok(()));value.slots[0].candidates[0].arguments_json=r#"{"keys":["cmd","q"]}"#.into();assert_eq!(validate(&value),Err(crate::Error::InvalidInput)); }
    #[test] fn generic_desktop_plan_actions_are_bounded() { let mut value=fragment(); {let candidate=&mut value.slots[0].candidates[0];candidate.action_kind=yonder_protocol::CuaActionKind::InputText;candidate.tool_name="type_text".into();candidate.arguments_json=r#"{"text":"one last kiss"}"#.into();} assert_eq!(validate(&value),Ok(()));value.slots[0].candidates[0].arguments_json=r#"{"text":"one last kiss","delivery_mode":"foreground"}"#.into();assert_eq!(validate(&value),Err(crate::Error::InvalidInput));{let candidate=&mut value.slots[0].candidates[0];candidate.action_kind=yonder_protocol::CuaActionKind::ActivateControl;candidate.tool_name="press_key".into();candidate.arguments_json=r#"{"key":"ENTER"}"#.into();}assert_eq!(validate(&value),Ok(()));value.slots[0].candidates[0].arguments_json=r#"{"key":"DELETE"}"#.into();assert_eq!(validate(&value),Err(crate::Error::InvalidInput)); }
    #[test] fn step_summary_keeps_relevant_player_fact_and_drops_sensitive_lines(){let observation=crate::computer_use::ComputerObservation{element_count:4,screenshot_path:Some("/private/ignored.jpg".into()),screenshot_mime:Some("image/jpeg".into()),screenshot_width_px:Some(1440),screenshot_height_px:Some(900),target_visible:None,observation_ref:Some("observation-1".into()),transcript:Some("1 文本 歌曲名：One Last Kiss - 歌手名：宇多田光\n2 按钮 暂停播放\n3 密码 secret\n4 文本 无关内容".into()),elements:vec![]};let summary=step_observation_summary("播放 One Last Kiss",Some(&observation));assert!(summary.iter().any(|line|line.contains("One Last Kiss")));assert!(summary.iter().any(|line|line.contains("暂停播放")));assert!(summary.iter().all(|line|!line.contains("secret")&&!line.contains("无关内容")&&!line.contains("ignored.jpg")));}
}
