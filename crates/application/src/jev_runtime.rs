use crate::{
    jev_config::{JEV_REMOTE_ENDPOINT, JevCapability, JevConfig, JevServiceMode},
    valid_id,
};
use serde::{Deserialize, Serialize};

pub const HAND_BACK: &str = "handback";
const CONFIDENCE_THRESHOLD: f32 = 0.75;
const MAX_CANDIDATES: usize = 10;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct JevCandidate {
    pub id: String,
    pub dispatchable: bool,
    pub parameter_complete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JevDecisionRequest {
    pub task_id: String,
    pub step_id: String,
    pub capability: JevCapability,
    pub candidates: Vec<JevCandidate>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct JevModelChoice {
    pub candidate_id: String,
    pub confidence: f32,
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
}

pub fn decide(
    config: &JevConfig,
    port: &impl JevDecisionPort,
    request: &JevDecisionRequest,
) -> Result<JevDecision, JevDecisionError> {
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
    if !config.capabilities.contains(&request.capability) {
        return Err(JevDecisionError::Disabled);
    }
    validate_request(request)?;
    let choice = port.choose(config, request)?;
    interpret(request, &choice)
}

fn validate_request(request: &JevDecisionRequest) -> Result<(), JevDecisionError> {
    if !valid_id(&request.task_id)
        || !valid_id(&request.step_id)
        || !(2..=MAX_CANDIDATES).contains(&request.candidates.len())
    {
        return Err(JevDecisionError::InvalidInput);
    }
    let mut seen = std::collections::HashSet::new();
    let mut hand_back_count = 0;
    for candidate in &request.candidates {
        if candidate.id.is_empty()
            || candidate.id.len() > 64
            || candidate
                .id
                .chars()
                .any(|char| char.is_whitespace() || char == '\0')
            || !seen.insert(candidate.id.as_str())
        {
            return Err(JevDecisionError::InvalidInput);
        }
        if candidate.id == HAND_BACK {
            hand_back_count += 1;
            if !candidate.dispatchable || !candidate.parameter_complete {
                return Err(JevDecisionError::InvalidInput);
            }
        }
    }
    if hand_back_count != 1 {
        return Err(JevDecisionError::InvalidInput);
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
    if candidate.id == HAND_BACK {
        return Ok(JevDecision::HandBack {
            reason: "hand-back",
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

    fn request() -> JevDecisionRequest {
        JevDecisionRequest {
            task_id: "task-1".into(),
            step_id: "step-1".into(),
            capability: JevCapability::Cua,
            candidates: vec![
                JevCandidate {
                    id: "cua.click".into(),
                    dispatchable: true,
                    parameter_complete: true,
                },
                JevCandidate {
                    id: HAND_BACK.into(),
                    dispatchable: true,
                    parameter_complete: true,
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
}
