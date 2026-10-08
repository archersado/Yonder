use crate::{
    jev_config::{JevCapability, JevConfig, JevServiceMode, JEV_REMOTE_ENDPOINT},
    valid_id,
};
use serde::{Deserialize, Serialize};

const CONFIDENCE_THRESHOLD: f32 = 0.75;
pub(crate) const MAX_CANDIDATES: usize = 10;
pub(crate) const HAND_BACK: &str = "handback";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JevCandidate {
    pub id: String,
    pub dispatchable: bool,
    pub parameter_complete: bool,
    /// 受限动作语义；不含 Driver 参数、联系人、消息正文或原始 Observe 内容。
    pub action_kind: String,
    pub target_ref: String,
    pub preconditions: Vec<String>,
    pub expected_observe: Vec<String>,
    /// 仅供新鲜 AX/UIA 动作空间选择的有界控件摘要。
    pub element_role: Option<String>,
    pub element_label: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JevDecisionRequest {
    pub task_id: String,
    /// 用户可见任务名称形成的整体目标；不含任务说明或敏感正文。
    pub task_goal: String,
    /// 当前不可变片段的完整步骤标签，最多十项。
    pub plan_steps: Vec<String>,
    pub current_step_index: u16,
    pub step_id: String,
    pub step_label: String,
    pub capability: JevCapability,
    pub observed_facts: Vec<String>,
    pub candidates: Vec<JevCandidate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JevModelChoice {
    pub candidate_id: String,
    pub confidence: f32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JevStepVerificationRequest {
    pub task_id: String,
    pub step_id: String,
    pub step_label: String,
    pub capability: JevCapability,
    pub action_kind: String,
    pub observed_facts: Vec<String>,
    pub expected_observe: Vec<String>,
    /// 本轮 Observe 派生的短生命周期脱敏摘要；不得持久化或记录。
    pub observation_summary: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JevStepOutcome {
    Complete,
    Incomplete,
    Uncertain,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JevModelStepVerification {
    pub outcome: JevStepOutcome,
    pub confidence: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JevStepDecision {
    Complete,
    Incomplete,
    HandBack,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JevDecision {
    Dispatch { candidate_id: String },
    HandBack { reason: &'static str },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JevDecisionError {
    Disabled,
    InvalidConfig,
    InvalidInput,
    CredentialUnavailable,
    DependencyUnavailable,
    WorkerFailed,
    TimedOut,
    RemoteError,
    InvalidResponse,
}

pub trait JevDecisionPort {
    fn choose(
        &self,
        config: &JevConfig,
        request: &JevDecisionRequest,
    ) -> Result<JevModelChoice, JevDecisionError>;
    fn verify_step(
        &self,
        _config: &JevConfig,
        _request: &JevStepVerificationRequest,
    ) -> Result<JevModelStepVerification, JevDecisionError> {
        Err(JevDecisionError::DependencyUnavailable)
    }
}

pub fn verify_step(
    config: &JevConfig,
    port: &(impl JevDecisionPort + ?Sized),
    request: &JevStepVerificationRequest,
) -> Result<JevStepDecision, JevDecisionError> {
    validate_config_and_capability(config, request.capability)?;
    validate_step_verification_request(request)?;
    let result = port.verify_step(config, request)?;
    if !result.confidence.is_finite() {
        return Err(JevDecisionError::InvalidResponse);
    }
    if result.confidence < CONFIDENCE_THRESHOLD || result.outcome == JevStepOutcome::Uncertain {
        return Ok(JevStepDecision::HandBack);
    }
    Ok(match result.outcome {
        JevStepOutcome::Complete => JevStepDecision::Complete,
        JevStepOutcome::Incomplete => JevStepDecision::Incomplete,
        JevStepOutcome::Uncertain => JevStepDecision::HandBack,
    })
}

pub fn decide(
    config: &JevConfig,
    port: &(impl JevDecisionPort + ?Sized),
    request: &JevDecisionRequest,
) -> Result<JevDecision, JevDecisionError> {
    validate_config_and_capability(config, request.capability)?;
    validate_request(request)?;
    let choice = port.choose(config, request)?;
    interpret(request, &choice)
}

fn validate_config_and_capability(
    config: &JevConfig,
    capability: JevCapability,
) -> Result<(), JevDecisionError> {
    if !config.enabled {
        return Err(JevDecisionError::Disabled);
    }
    config
        .validate()
        .map_err(|_| JevDecisionError::InvalidConfig)?;
    if config.service_mode != JevServiceMode::Remote
        || !matches!(
            config.endpoint.as_str(),
            JEV_REMOTE_ENDPOINT | "https://api.typesafe.ai/"
        )
    {
        return Err(JevDecisionError::InvalidConfig);
    }
    if !config.capabilities.contains(&capability) {
        return Err(JevDecisionError::Disabled);
    }
    Ok(())
}

fn validate_step_verification_request(
    request: &JevStepVerificationRequest,
) -> Result<(), JevDecisionError> {
    if !valid_id(&request.task_id)
        || !valid_id(&request.step_id)
        || !crate::valid_step_label(&request.step_label)
        || request.action_kind.is_empty()
        || request.action_kind.len() > 64
        || request.observed_facts.is_empty()
        || request.observed_facts.len() > 4
        || request.expected_observe.is_empty()
        || request.expected_observe.len() > 4
        || request.observation_summary.is_empty()
        || request.observation_summary.len() > 32
        || request
            .observed_facts
            .iter()
            .chain(&request.expected_observe)
            .any(|value| value.is_empty() || value.len() > 64 || value.contains('\0'))
        || request
            .observation_summary
            .iter()
            .any(|value| value.is_empty() || value.len() > 256 || value.contains('\0'))
    {
        return Err(JevDecisionError::InvalidInput);
    }
    Ok(())
}

fn validate_request(request: &JevDecisionRequest) -> Result<(), JevDecisionError> {
    let mut observed = std::collections::HashSet::new();
    if !valid_id(&request.task_id)
        || !crate::valid_step_label(&request.task_goal)
        || request.plan_steps.is_empty()
        || request.plan_steps.len() > 10
        || usize::from(request.current_step_index) >= request.plan_steps.len()
        || request
            .plan_steps
            .iter()
            .any(|label| !crate::valid_step_label(label))
        || request.plan_steps[usize::from(request.current_step_index)] != request.step_label
        || !valid_id(&request.step_id)
        || !crate::valid_step_label(&request.step_label)
        || !(2..=MAX_CANDIDATES).contains(&request.candidates.len())
        || request.observed_facts.len() > 4
        || request.observed_facts.iter().any(|fact| {
            fact.is_empty()
                || fact.len() > 64
                || fact.contains('\0')
                || !observed.insert(fact.as_str())
        })
    {
        return Err(JevDecisionError::InvalidInput);
    }
    let mut seen = std::collections::HashSet::new();
    for candidate in &request.candidates {
        if candidate.id.is_empty()
            || candidate.id.len() > 64
            || candidate.id == HAND_BACK
            || candidate
                .id
                .chars()
                .any(|char| char.is_whitespace() || char == '\0')
            || !seen.insert(candidate.id.as_str())
        {
            return Err(JevDecisionError::InvalidInput);
        }
        if candidate.action_kind.is_empty()
            || candidate.action_kind.len() > 64
            || candidate.target_ref.is_empty()
            || candidate.target_ref.len() > 64
            || candidate.preconditions.len() > 4
            || candidate.expected_observe.len() > 4
            || candidate
                .element_role
                .as_ref()
                .is_some_and(|value| value.is_empty() || value.len() > 32 || value.contains('\0'))
            || candidate
                .element_label
                .as_ref()
                .is_some_and(|value| value.is_empty() || value.len() > 128 || value.contains('\0'))
            || candidate
                .preconditions
                .iter()
                .chain(&candidate.expected_observe)
                .any(|condition| {
                    condition.is_empty() || condition.len() > 64 || condition.contains('\0')
                })
        {
            return Err(JevDecisionError::InvalidInput);
        }
    }
    Ok(())
}

fn interpret(
    request: &JevDecisionRequest,
    choice: &JevModelChoice,
) -> Result<JevDecision, JevDecisionError> {
    if !choice.confidence.is_finite() {
        return Err(JevDecisionError::InvalidResponse);
    }
    let Some(candidate) = request
        .candidates
        .iter()
        .find(|candidate| candidate.id == choice.candidate_id)
    else {
        return Ok(JevDecision::HandBack {
            reason: "unknown-candidate",
        });
    };
    if choice.confidence < CONFIDENCE_THRESHOLD {
        return Ok(JevDecision::HandBack {
            reason: "low-confidence",
        });
    }
    if !candidate.dispatchable || !candidate.parameter_complete {
        return Ok(JevDecision::HandBack {
            reason: "candidate-not-dispatchable",
        });
    }
    Ok(JevDecision::Dispatch {
        candidate_id: candidate.id.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakePort(Result<JevModelChoice, JevDecisionError>);

    impl JevDecisionPort for FakePort {
        fn choose(
            &self,
            _: &JevConfig,
            _: &JevDecisionRequest,
        ) -> Result<JevModelChoice, JevDecisionError> {
            self.0.clone()
        }
    }

    struct StepPort(JevModelStepVerification);
    impl JevDecisionPort for StepPort {
        fn choose(
            &self,
            _: &JevConfig,
            _: &JevDecisionRequest,
        ) -> Result<JevModelChoice, JevDecisionError> {
            panic!("步骤核验不得选择动作")
        }
        fn verify_step(
            &self,
            _: &JevConfig,
            _: &JevStepVerificationRequest,
        ) -> Result<JevModelStepVerification, JevDecisionError> {
            Ok(self.0.clone())
        }
    }

    fn step_request() -> JevStepVerificationRequest {
        JevStepVerificationRequest {
            task_id: "task-1".into(),
            step_id: "play-song".into(),
            step_label: "播放 One Last Kiss".into(),
            capability: JevCapability::Cua,
            action_kind: "ActivateControl".into(),
            observed_facts: vec!["ActionSucceeded=true".into(), "ObserveValid=true".into()],
            expected_observe: vec!["ActionSucceeded=true".into()],
            observation_summary: vec!["歌曲名：One Last Kiss - 歌手名：宇多田光".into()],
        }
    }

    fn request() -> JevDecisionRequest {
        JevDecisionRequest {
            task_id: "task-1".into(),
            task_goal: "完成控件选择验证".into(),
            plan_steps: vec!["选择并激活目标控件".into()],
            current_step_index: 0,
            step_id: "step-1".into(),
            step_label: "选择并激活目标控件".into(),
            capability: JevCapability::Cua,
            observed_facts: vec!["ApplicationReady=true".into()],
            candidates: vec![
                JevCandidate {
                    id: "cua.click".into(),
                    dispatchable: true,
                    parameter_complete: true,
                    action_kind: "bring-to-front".into(),
                    target_ref: "app-1".into(),
                    preconditions: vec!["application-ready=true".into()],
                    expected_observe: vec!["target-resolved=true".into()],
                    element_role: None,
                    element_label: None,
                },
                JevCandidate {
                    id: "cua.raise".into(),
                    dispatchable: true,
                    parameter_complete: true,
                    action_kind: "bring-to-front".into(),
                    target_ref: "app-1".into(),
                    preconditions: vec!["application-ready=true".into()],
                    expected_observe: vec!["target-resolved=true".into()],
                    element_role: None,
                    element_label: None,
                },
            ],
        }
    }

    fn config() -> JevConfig {
        JevConfig {
            enabled: true,
            service_mode: JevServiceMode::Remote,
            endpoint: JEV_REMOTE_ENDPOINT.into(),
            ..JevConfig::default()
        }
    }

    #[test]
    fn dispatches_supported_high_confidence_candidate() {
        let config = config();
        let result = decide(
            &config,
            &FakePort(Ok(JevModelChoice {
                candidate_id: "cua.click".into(),
                confidence: 0.99,
            })),
            &request(),
        );
        assert_eq!(
            result,
            Ok(JevDecision::Dispatch {
                candidate_id: "cua.click".into()
            })
        );
    }

    #[test]
    fn hands_back_low_confidence_and_disabled_candidates() {
        let config = config();
        let result = decide(
            &config,
            &FakePort(Ok(JevModelChoice {
                candidate_id: "cua.click".into(),
                confidence: 0.5,
            })),
            &request(),
        );
        assert_eq!(
            result,
            Ok(JevDecision::HandBack {
                reason: "low-confidence"
            })
        );

        let mut disabled = JevCandidate {
            dispatchable: false,
            ..request().candidates[0].clone()
        };
        disabled.id = "cua.disabled".into();
        let mut local_request = request();
        local_request.candidates[0] = disabled;
        let result = decide(
            &config,
            &FakePort(Ok(JevModelChoice {
                candidate_id: "cua.disabled".into(),
                confidence: 0.99,
            })),
            &local_request,
        );
        assert_eq!(
            result,
            Ok(JevDecision::HandBack {
                reason: "candidate-not-dispatchable"
            })
        );
    }

    #[test]
    fn rejects_disabled_config_and_invalid_candidates() {
        let mut disabled_config = config();
        disabled_config.enabled = false;
        assert_eq!(
            decide(
                &disabled_config,
                &FakePort(Ok(JevModelChoice {
                    candidate_id: "cua.click".into(),
                    confidence: 1.0
                })),
                &request()
            ),
            Err(JevDecisionError::Disabled)
        );

        let valid_config = config();
        let mut invalid = request();
        invalid.candidates.truncate(1);
        assert_eq!(
            decide(
                &valid_config,
                &FakePort(Ok(JevModelChoice {
                    candidate_id: "cua.click".into(),
                    confidence: 1.0
                })),
                &invalid
            ),
            Err(JevDecisionError::InvalidInput)
        );

        let mut reserved = request();
        reserved.candidates[0].id = HAND_BACK.into();
        assert_eq!(
            decide(
                &valid_config,
                &FakePort(Ok(JevModelChoice {
                    candidate_id: HAND_BACK.into(),
                    confidence: 1.0
                })),
                &reserved
            ),
            Err(JevDecisionError::InvalidInput)
        );

        let mut local_config = config();
        local_config.service_mode = JevServiceMode::Local;
        local_config.endpoint = "http://localhost".into();
        assert_eq!(
            decide(
                &local_config,
                &FakePort(Ok(JevModelChoice {
                    candidate_id: "cua.click".into(),
                    confidence: 1.0
                })),
                &request()
            ),
            Err(JevDecisionError::InvalidConfig)
        );
    }

    #[test]
    fn verifies_step_with_three_value_safety_boundary() {
        assert_eq!(
            verify_step(
                &config(),
                &StepPort(JevModelStepVerification {
                    outcome: JevStepOutcome::Complete,
                    confidence: 0.99
                }),
                &step_request()
            ),
            Ok(JevStepDecision::Complete)
        );
        assert_eq!(
            verify_step(
                &config(),
                &StepPort(JevModelStepVerification {
                    outcome: JevStepOutcome::Incomplete,
                    confidence: 0.99
                }),
                &step_request()
            ),
            Ok(JevStepDecision::Incomplete)
        );
        assert_eq!(
            verify_step(
                &config(),
                &StepPort(JevModelStepVerification {
                    outcome: JevStepOutcome::Uncertain,
                    confidence: 0.99
                }),
                &step_request()
            ),
            Ok(JevStepDecision::HandBack)
        );
        assert_eq!(
            verify_step(
                &config(),
                &StepPort(JevModelStepVerification {
                    outcome: JevStepOutcome::Complete,
                    confidence: 0.5
                }),
                &step_request()
            ),
            Ok(JevStepDecision::HandBack)
        );
    }
}
