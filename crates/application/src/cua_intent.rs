//! CUA 敏感目标与消息正文的短期内存保管。
//!
//! 本模块只在桌面组合根内存中保存明文。Gateway、计划片段、任务事件、
//! Outbox、SQLite 和 Jev 只能接触不透明引用与状态摘要。
use crate::{AuthContext, Status, Task};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

pub const MAX_CUA_INTENTS: usize = 32;
pub const MAX_CUA_INTENT_LIFETIME_MS: u64 = 15 * 60 * 1000;
pub const MAX_CUA_TARGET_BYTES: usize = 256;
pub const MAX_CUA_MESSAGE_BYTES: usize = 4 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CuaIntentState {
    Prepared,
    AwaitingUser,
    Approved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CuaIntentSummary {
    pub intent_ref: String,
    pub confirmation_ref: String,
    pub state: CuaIntentState,
    pub expires_at_ms: u64,
}

/// 只允许可信本机 UI 获取；不得经 Gateway、日志或任务事实返回。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CuaIntentPreview {
    pub intent_ref: String,
    pub confirmation_ref: String,
    pub target: String,
    pub message: String,
    pub expires_at_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CuaIntentText {
    Target,
    Message,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CuaIntentError {
    InvalidInput,
    PermissionDenied,
    NotFound,
    Expired,
    Rejected,
    Capacity,
    Unavailable,
}

#[derive(Clone)]
struct Intent {
    task_id: String,
    owner_agent_id: String,
    confirmation_ref: String,
    target: String,
    message: String,
    expires_at_ms: u64,
    state: CuaIntentState,
}

#[derive(Default)]
pub struct CuaIntentRegistry {
    intents: Mutex<BTreeMap<String, Intent>>,
    next: AtomicU64,
}

impl CuaIntentRegistry {
    pub fn propose(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        target: String,
        message: String,
        now_ms: u64,
    ) -> Result<CuaIntentSummary, CuaIntentError> {
        require_owner(auth, task)?;
        if !active(task)
            || !valid_text(&target, MAX_CUA_TARGET_BYTES)
            || !valid_text(&message, MAX_CUA_MESSAGE_BYTES)
        {
            return Err(CuaIntentError::InvalidInput);
        }
        let mut intents = self
            .intents
            .lock()
            .map_err(|_| CuaIntentError::Unavailable)?;
        intents.retain(|_, value| value.expires_at_ms > now_ms);
        if intents.len() >= MAX_CUA_INTENTS {
            return Err(CuaIntentError::Capacity);
        }
        let ordinal = self.next.fetch_add(1, Ordering::Relaxed) + 1;
        let intent_ref = opaque_ref("cua_intent", task, ordinal, now_ms);
        let confirmation_ref = opaque_ref("cua_confirmation", task, ordinal, now_ms);
        let expires_at_ms = now_ms.saturating_add(MAX_CUA_INTENT_LIFETIME_MS);
        intents.insert(
            intent_ref.clone(),
            Intent {
                task_id: task.id.clone(),
                owner_agent_id: task.owner_agent_id.clone(),
                confirmation_ref: confirmation_ref.clone(),
                target,
                message,
                expires_at_ms,
                state: CuaIntentState::Prepared,
            },
        );
        Ok(CuaIntentSummary {
            intent_ref,
            confirmation_ref,
            state: CuaIntentState::Prepared,
            expires_at_ms,
        })
    }

    /// 仅在当前一次 Driver 派发前展开明文；计划与持久化对象始终保留引用。
    pub fn resolve_text(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        intent_ref: &str,
        text: CuaIntentText,
        now_ms: u64,
    ) -> Result<String, CuaIntentError> {
        require_owner(auth, task)?;
        let mut intents = self
            .intents
            .lock()
            .map_err(|_| CuaIntentError::Unavailable)?;
        let intent = fetch(&mut intents, task, intent_ref, now_ms)?;
        Ok(match text {
            CuaIntentText::Target => intent.target,
            CuaIntentText::Message => intent.message,
        })
    }

    /// 到达发送槽位时才开放本机确认，避免过早确认失去上下文。
    pub fn arm(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        intent_ref: &str,
        confirmation_ref: &str,
        now_ms: u64,
    ) -> Result<CuaIntentSummary, CuaIntentError> {
        require_owner(auth, task)?;
        let mut intents = self
            .intents
            .lock()
            .map_err(|_| CuaIntentError::Unavailable)?;
        let mut intent = fetch(&mut intents, task, intent_ref, now_ms)?;
        if intent.confirmation_ref != confirmation_ref {
            return Err(CuaIntentError::PermissionDenied);
        }
        if intent.state == CuaIntentState::Prepared {
            intent.state = CuaIntentState::AwaitingUser;
        }
        let summary = summary(intent_ref, &intent);
        intents.insert(intent_ref.into(), intent);
        Ok(summary)
    }

    pub fn list_for_local(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        now_ms: u64,
    ) -> Result<Vec<CuaIntentSummary>, CuaIntentError> {
        require_local(auth)?;
        if !active(task) {
            return Err(CuaIntentError::PermissionDenied);
        }
        let mut intents = self
            .intents
            .lock()
            .map_err(|_| CuaIntentError::Unavailable)?;
        intents.retain(|_, value| value.expires_at_ms > now_ms);
        Ok(intents
            .iter()
            .filter(|(_, value)| {
                value.task_id == task.id
                    && value.owner_agent_id == task.owner_agent_id
                    && value.state != CuaIntentState::Prepared
            })
            .map(|(key, value)| summary(key, value))
            .collect())
    }

    pub fn preview_for_local(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        intent_ref: &str,
        now_ms: u64,
    ) -> Result<CuaIntentPreview, CuaIntentError> {
        require_local(auth)?;
        let mut intents = self
            .intents
            .lock()
            .map_err(|_| CuaIntentError::Unavailable)?;
        let intent = fetch(&mut intents, task, intent_ref, now_ms)?;
        if intent.state == CuaIntentState::Prepared {
            return Err(CuaIntentError::PermissionDenied);
        }
        Ok(CuaIntentPreview {
            intent_ref: intent_ref.into(),
            confirmation_ref: intent.confirmation_ref,
            target: intent.target,
            message: intent.message,
            expires_at_ms: intent.expires_at_ms,
        })
    }

    pub fn approve(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        intent_ref: &str,
        now_ms: u64,
    ) -> Result<CuaIntentSummary, CuaIntentError> {
        require_local(auth)?;
        let mut intents = self
            .intents
            .lock()
            .map_err(|_| CuaIntentError::Unavailable)?;
        let mut intent = fetch(&mut intents, task, intent_ref, now_ms)?;
        if intent.state != CuaIntentState::AwaitingUser {
            return Err(CuaIntentError::Rejected);
        }
        intent.state = CuaIntentState::Approved;
        let result = summary(intent_ref, &intent);
        intents.insert(intent_ref.into(), intent);
        Ok(result)
    }

    pub fn reject(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        intent_ref: &str,
        now_ms: u64,
    ) -> Result<(), CuaIntentError> {
        require_local(auth)?;
        let mut intents = self
            .intents
            .lock()
            .map_err(|_| CuaIntentError::Unavailable)?;
        let intent = fetch(&mut intents, task, intent_ref, now_ms)?;
        if intent.state == CuaIntentState::Prepared {
            return Err(CuaIntentError::PermissionDenied);
        }
        intents.remove(intent_ref);
        Ok(())
    }

    /// 发送派发前消费批准。之后无论成功、unknown、超时或崩溃都不能恢复。
    pub fn consume_send(
        &self,
        auth: AuthContext<'_>,
        task: &Task,
        intent_ref: &str,
        confirmation_ref: &str,
        now_ms: u64,
    ) -> Result<(), CuaIntentError> {
        require_owner(auth, task)?;
        let mut intents = self
            .intents
            .lock()
            .map_err(|_| CuaIntentError::Unavailable)?;
        let intent = fetch(&mut intents, task, intent_ref, now_ms)?;
        if intent.confirmation_ref != confirmation_ref {
            return Err(CuaIntentError::PermissionDenied);
        }
        if intent.state != CuaIntentState::Approved {
            return Err(CuaIntentError::Rejected);
        }
        intents.remove(intent_ref);
        Ok(())
    }

    pub fn revoke_owner(&self, auth: AuthContext<'_>, owner: &str) -> Result<usize, CuaIntentError> {
        require_local(auth)?;
        let mut intents = self.intents.lock().map_err(|_| CuaIntentError::Unavailable)?;
        let count = intents.len();
        intents.retain(|_, value| value.owner_agent_id != owner);
        Ok(count - intents.len())
    }
}

fn fetch(
    intents: &mut BTreeMap<String, Intent>,
    task: &Task,
    intent_ref: &str,
    now_ms: u64,
) -> Result<Intent, CuaIntentError> {
    let intent = intents
        .get(intent_ref)
        .cloned()
        .ok_or(CuaIntentError::NotFound)?;
    if intent.expires_at_ms <= now_ms {
        intents.remove(intent_ref);
        return Err(CuaIntentError::Expired);
    }
    if intent.task_id != task.id || intent.owner_agent_id != task.owner_agent_id || !active(task) {
        return Err(CuaIntentError::PermissionDenied);
    }
    Ok(intent)
}

fn summary(intent_ref: &str, intent: &Intent) -> CuaIntentSummary {
    CuaIntentSummary {
        intent_ref: intent_ref.into(),
        confirmation_ref: intent.confirmation_ref.clone(),
        state: intent.state,
        expires_at_ms: intent.expires_at_ms,
    }
}
fn require_owner(auth: AuthContext<'_>, task: &Task) -> Result<(), CuaIntentError> {
    if matches!(auth, AuthContext::Agent(_)) && auth.agent_id() == task.owner_agent_id {
        Ok(())
    } else {
        Err(CuaIntentError::PermissionDenied)
    }
}
fn require_local(auth: AuthContext<'_>) -> Result<(), CuaIntentError> {
    if matches!(auth, AuthContext::LocalUser(_)) {
        Ok(())
    } else {
        Err(CuaIntentError::PermissionDenied)
    }
}
fn active(task: &Task) -> bool {
    matches!(
        task.status,
        Status::Created | Status::Running | Status::WaitingForUser | Status::Paused
    )
}
fn valid_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.contains('\0')
}
fn opaque_ref(prefix: &str, task: &Task, ordinal: u64, now_ms: u64) -> String {
    let mut digest = Sha256::new();
    digest.update(prefix.as_bytes());
    digest.update(task.id.as_bytes());
    digest.update(task.owner_agent_id.as_bytes());
    digest.update(ordinal.to_be_bytes());
    digest.update(now_ms.to_be_bytes());
    format!("{prefix}_{}", &format!("{:x}", digest.finalize())[..24])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskSource;

    fn task() -> Task {
        Task {
            id: "task-a".into(),
            owner_agent_id: "agent-a".into(),
            name: None,
            source: TaskSource::LocalAgent,
            status: Status::Running,
            sequence: 9,
        }
    }

    #[test]
    fn sensitive_values_are_local_and_send_approval_is_consumed_once() {
        let registry = CuaIntentRegistry::default();
        let task = task();
        let proposed = registry
            .propose(
                AuthContext::Agent("agent-a"),
                &task,
                "宫健的分身".into(),
                "hi".into(),
                100,
            )
            .unwrap();
        assert!(!proposed.intent_ref.contains("宫健"));
        assert_eq!(
            registry
                .list_for_local(AuthContext::LocalUser("desktop"), &task, 101)
                .unwrap(),
            vec![]
        );
        assert_eq!(
            registry
                .resolve_text(
                    AuthContext::Agent("agent-a"),
                    &task,
                    &proposed.intent_ref,
                    CuaIntentText::Target,
                    101
                )
                .unwrap(),
            "宫健的分身"
        );
        registry
            .arm(
                AuthContext::Agent("agent-a"),
                &task,
                &proposed.intent_ref,
                &proposed.confirmation_ref,
                102,
            )
            .unwrap();
        let preview = registry
            .preview_for_local(
                AuthContext::LocalUser("desktop"),
                &task,
                &proposed.intent_ref,
                103,
            )
            .unwrap();
        assert_eq!(
            (preview.target.as_str(), preview.message.as_str()),
            ("宫健的分身", "hi")
        );
        assert_eq!(
            registry.consume_send(
                AuthContext::Agent("agent-a"),
                &task,
                &proposed.intent_ref,
                &proposed.confirmation_ref,
                104
            ),
            Err(CuaIntentError::Rejected)
        );
        registry
            .approve(
                AuthContext::LocalUser("desktop"),
                &task,
                &proposed.intent_ref,
                105,
            )
            .unwrap();
        assert_eq!(
            registry.consume_send(
                AuthContext::Agent("agent-a"),
                &task,
                &proposed.intent_ref,
                &proposed.confirmation_ref,
                106
            ),
            Ok(())
        );
        assert_eq!(
            registry.consume_send(
                AuthContext::Agent("agent-a"),
                &task,
                &proposed.intent_ref,
                &proposed.confirmation_ref,
                107
            ),
            Err(CuaIntentError::NotFound)
        );
    }

    #[test]
    fn intent_is_task_owner_and_expiry_bound_but_not_sequence_bound() {
        let registry = CuaIntentRegistry::default();
        let mut task = task();
        let proposed = registry
            .propose(
                AuthContext::Agent("agent-a"),
                &task,
                "target".into(),
                "message".into(),
                100,
            )
            .unwrap();
        task.sequence += 1;
        assert_eq!(
            registry.resolve_text(
                AuthContext::Agent("other"),
                &task,
                &proposed.intent_ref,
                CuaIntentText::Message,
                101
            ),
            Err(CuaIntentError::PermissionDenied)
        );
        assert_eq!(
            registry
                .resolve_text(
                    AuthContext::Agent("agent-a"),
                    &task,
                    &proposed.intent_ref,
                    CuaIntentText::Message,
                    101
                )
                .unwrap(),
            "message"
        );
        assert_eq!(
            registry.resolve_text(
                AuthContext::Agent("agent-a"),
                &task,
                &proposed.intent_ref,
                CuaIntentText::Message,
                100 + MAX_CUA_INTENT_LIFETIME_MS
            ),
            Err(CuaIntentError::Expired)
        );
    }
}
