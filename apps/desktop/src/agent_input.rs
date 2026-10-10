use base64::Engine;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::mpsc as tokio_mpsc;
use yonder_application::agent_input::{
    AgentInputAttachment, AgentInputParams, AgentInputSink, AgentInputSource, AgentReplyMode,
    AgentReplyParams, AgentReplyPhase, DeliveryOutcome, MAX_AGENT_ATTACHMENT_BYTES, submit,
    submit_selection,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct AgentConnectionPresence {
    pub gateway_connected: bool,
    pub input_connected: bool,
}

pub(crate) struct Delivery {
    pub input: AgentInputParams,
    pub attachment: Option<AgentInputAttachment>,
    pub reply: mpsc::Sender<DeliveryOutcome>,
}

#[derive(Clone)]
struct SessionSink {
    agent_id: String,
    session_id: String,
    supports_attachment: bool,
    supports_replan: bool,
    supports_reply: bool,
    tx: tokio_mpsc::UnboundedSender<Delivery>,
    close: tokio_mpsc::UnboundedSender<()>,
}

impl AgentInputSink for SessionSink {
    fn deliver(
        &self,
        input: AgentInputParams,
        attachment: Option<AgentInputAttachment>,
    ) -> DeliveryOutcome {
        if attachment.is_some() && !self.supports_attachment {
            return DeliveryOutcome::Rejected;
        }
        let (reply, result) = mpsc::channel();
        let timeout = if attachment.is_some() {
            Duration::from_secs(61)
        } else {
            Duration::from_secs(11)
        };
        if self
            .tx
            .send(Delivery {
                input,
                attachment,
                reply,
            })
            .is_err()
        {
            return DeliveryOutcome::Unknown;
        }
        result
            .recv_timeout(timeout)
            .unwrap_or(DeliveryOutcome::Unknown)
    }
}

#[derive(Clone, Default)]
pub struct AgentInputHub {
    inner: Arc<Mutex<HubState>>,
    next_input: Arc<AtomicU64>,
    next_session: Arc<AtomicU64>,
    next_gateway: Arc<AtomicU64>,
}

#[derive(Default)]
struct ReplyState {
    input_id: String,
    session_id: String,
    reply_id: Option<String>,
    next_sequence: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionDelivery {
    pub input_id: String,
    pub outcome: DeliveryOutcome,
    pub reply_expected: bool,
}

#[derive(Default)]
struct HubState {
    sessions: HashMap<String, (u64, SessionSink)>,
    gateways: HashMap<u64, String>,
    active: Option<String>,
    reply: Option<ReplyState>,
}

impl AgentInputHub {
    pub(crate) fn register_gateway(&self, agent_id: String) -> (u64, AgentConnectionPresence) {
        let token = self.next_gateway.fetch_add(1, Ordering::Relaxed) + 1;
        if let Ok(mut state) = self.inner.lock() {
            state.gateways.insert(token, agent_id);
        }
        (token, self.presence())
    }
    pub(crate) fn unregister_gateway(&self, token: u64) -> AgentConnectionPresence {
        if let Ok(mut state) = self.inner.lock() {
            state.gateways.remove(&token);
        }
        self.presence()
    }
    pub(crate) fn register(
        &self,
        agent_id: String,
        session_id: String,
        supports_attachment: bool,
        supports_replan: bool,
        supports_reply: bool,
        tx: tokio_mpsc::UnboundedSender<Delivery>,
        close: tokio_mpsc::UnboundedSender<()>,
    ) -> (u64, bool) {
        let token = self.next_session.fetch_add(1, Ordering::Relaxed) + 1;
        let connected = if let Ok(mut state) = self.inner.lock() {
            let replacing = state.sessions.contains_key(&session_id);
            state.sessions.insert(
                session_id.clone(),
                (
                    token,
                    SessionSink {
                        agent_id,
                        session_id: session_id.clone(),
                        supports_attachment,
                        supports_replan,
                        supports_reply,
                        tx,
                        close,
                    },
                ),
            );
            if state.sessions.len() == 1
                || replacing && state.active.as_deref() == Some(&session_id)
            {
                state.active = Some(session_id);
            } else if !replacing {
                state.active = None;
            }
            true
        } else {
            false
        };
        (token, connected)
    }

    pub(crate) fn unregister(&self, session_id: &str, token: u64) -> bool {
        if let Ok(mut state) = self.inner.lock() {
            if state
                .sessions
                .get(session_id)
                .is_some_and(|(current, _)| *current == token)
            {
                state.sessions.remove(session_id);
                if state
                    .reply
                    .as_ref()
                    .is_some_and(|reply| reply.session_id == session_id)
                {
                    state.reply = None;
                }
                state.active = state
                    .sessions
                    .keys()
                    .next()
                    .cloned()
                    .filter(|_| state.sessions.len() == 1);
            }
            !state.sessions.is_empty()
        } else {
            false
        }
    }

    pub fn disconnect_agent(&self, agent_id: &str) -> bool {
        let mut state = match self.inner.lock() {
            Ok(state) => state,
            Err(_) => return false,
        };
        let removed: Vec<String> = state
            .sessions
            .iter()
            .filter(|(_, (_, sink))| sink.agent_id == agent_id)
            .map(|(session_id, _)| session_id.clone())
            .collect();
        for session_id in &removed {
            if let Some((_, sink)) = state.sessions.remove(session_id.as_str()) {
                let _ = sink.close.send(());
            }
        }
        if state
            .reply
            .as_ref()
            .is_some_and(|reply| removed.contains(&reply.session_id))
        {
            state.reply = None;
        }
        if removed.is_empty() {
            return !state.sessions.is_empty();
        }
        state.active = state
            .sessions
            .keys()
            .next()
            .cloned()
            .filter(|_| state.sessions.len() == 1);
        !state.sessions.is_empty()
    }

    pub fn connected(&self) -> bool {
        self.presence().input_connected
    }
    pub fn presence(&self) -> AgentConnectionPresence {
        self.inner
            .lock()
            .map(|state| AgentConnectionPresence {
                gateway_connected: !state.gateways.is_empty(),
                input_connected: !state.sessions.is_empty(),
            })
            .unwrap_or_default()
    }

    pub fn deliver_voice(&self, content: &str) -> Result<DeliveryOutcome, &'static str> {
        let sink = self.active_sink()?;
        let now = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "系统时间不可用")?
                .as_millis(),
        )
        .map_err(|_| "系统时间不可用")?;
        let id = format!(
            "voice_{}_{}",
            std::process::id(),
            self.next_input.fetch_add(1, Ordering::Relaxed) + 1
        );
        Ok(submit(
            &sink,
            &id,
            &sink.session_id,
            AgentInputSource::Voice,
            content,
            now,
        ))
    }

    pub fn deliver_selection(
        &self,
        content: &str,
        image_base64: &str,
    ) -> Result<SelectionDelivery, &'static str> {
        let sink = self.active_sink()?;
        if !sink.supports_attachment {
            return Err("当前Agent不支持截图提问");
        }
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(image_base64)
            .map_err(|_| "截图数据无效")?;
        if bytes.len() > MAX_AGENT_ATTACHMENT_BYTES {
            return Err("截图超过Agent附件上限");
        }
        let now = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "系统时间不可用")?
                .as_millis(),
        )
        .map_err(|_| "系统时间不可用")?;
        let suffix = self.next_input.fetch_add(1, Ordering::Relaxed) + 1;
        let input_id = format!("selection_{}_{}", std::process::id(), suffix);
        if sink.supports_reply {
            let mut state = self.inner.lock().map_err(|_| "Agent输入通道不可用")?;
            if state.reply.is_some() {
                return Err("当前仍有Agent回复进行中");
            }
            state.reply = Some(ReplyState {
                input_id: input_id.clone(),
                session_id: sink.session_id.clone(),
                reply_id: None,
                next_sequence: 1,
            });
        }
        let outcome = submit_selection(
            &sink,
            &input_id,
            &format!("image_{}_{}", std::process::id(), suffix),
            &sink.session_id,
            content,
            bytes,
            sink.supports_reply.then_some(AgentReplyMode::Stream),
            now,
        );
        if outcome != DeliveryOutcome::Accepted && sink.supports_reply {
            self.dismiss_reply(&input_id);
        }
        Ok(SelectionDelivery {
            input_id,
            outcome,
            reply_expected: sink.supports_reply,
        })
    }

    pub fn deliver_selection_text(&self, content: &str) -> Result<SelectionDelivery, &'static str> {
        let sink = self.active_sink()?;
        let now = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "系统时间不可用")?
                .as_millis(),
        )
        .map_err(|_| "系统时间不可用")?;
        let suffix = self.next_input.fetch_add(1, Ordering::Relaxed) + 1;
        let input_id = format!("selection_{}_{}", std::process::id(), suffix);
        if sink.supports_reply {
            let mut state = self.inner.lock().map_err(|_| "Agent输入通道不可用")?;
            if state.reply.is_some() {
                return Err("当前仍有Agent回复进行中");
            }
            state.reply = Some(ReplyState {
                input_id: input_id.clone(),
                session_id: sink.session_id.clone(),
                reply_id: None,
                next_sequence: 1,
            });
        }
        let input = AgentInputParams {
            input_id: input_id.clone(),
            session_id: sink.session_id.clone(),
            source: AgentInputSource::Selection,
            content: content.trim().into(),
            attachment_id: None,
            reply_mode: sink.supports_reply.then_some(AgentReplyMode::Stream),
            created_at: now,
            deadline: now.saturating_add(10_000),
        };
        let outcome = if input.validate(now).is_err() {
            DeliveryOutcome::Rejected
        } else {
            sink.deliver(input, None)
        };
        if outcome != DeliveryOutcome::Accepted && sink.supports_reply {
            self.dismiss_reply(&input_id);
        }
        Ok(SelectionDelivery {
            input_id,
            outcome,
            reply_expected: sink.supports_reply,
        })
    }

    fn active_sink(&self) -> Result<SessionSink, &'static str> {
        let state = self.inner.lock().map_err(|_| "Agent输入通道不可用")?;
        let session = state.active.as_ref().ok_or(if state.sessions.is_empty() {
            "当前没有可接收输入的Agent"
        } else {
            "请选择接收输入的Agent"
        })?;
        state
            .sessions
            .get(session)
            .map(|(_, sink)| sink.clone())
            .ok_or("Agent会话已断开")
    }

    pub(crate) fn accept_reply(&self, params: &AgentReplyParams) -> bool {
        if params.validate().is_err() {
            return false;
        }
        let mut state = match self.inner.lock() {
            Ok(state) => state,
            Err(_) => return false,
        };
        let Some(reply) = state.reply.as_mut() else {
            return false;
        };
        if reply.input_id != params.input_id
            || reply.session_id != params.session_id
            || reply.next_sequence != params.sequence
        {
            return false;
        }
        if let Some(reply_id) = reply.reply_id.as_deref() {
            if reply_id != params.reply_id {
                return false;
            }
        } else if params.phase == AgentReplyPhase::Started {
            reply.reply_id = Some(params.reply_id.clone());
        } else {
            return false;
        }
        reply.next_sequence = reply.next_sequence.saturating_add(1);
        if matches!(
            params.phase,
            AgentReplyPhase::Completed | AgentReplyPhase::Failed | AgentReplyPhase::Interrupted
        ) {
            state.reply = None;
        }
        true
    }

    pub fn dismiss_reply(&self, input_id: &str) -> bool {
        let mut state = match self.inner.lock() {
            Ok(state) => state,
            Err(_) => return false,
        };
        if state
            .reply
            .as_ref()
            .is_some_and(|reply| reply.input_id == input_id)
        {
            state.reply = None;
            true
        } else {
            false
        }
    }

    pub(crate) fn interrupt_reply(&self, session_id: &str) -> Option<AgentReplyParams> {
        let mut state = self.inner.lock().ok()?;
        let reply = state
            .reply
            .take()
            .filter(|reply| reply.session_id == session_id)?;
        Some(AgentReplyParams {
            input_id: reply.input_id.clone(),
            session_id: reply.session_id,
            reply_id: reply.reply_id.unwrap_or(reply.input_id),
            sequence: reply.next_sequence,
            phase: AgentReplyPhase::Interrupted,
            content: None,
            truncated: false,
        })
    }

    pub fn deliver_for_agent(
        &self,
        agent_id: &str,
        content: &str,
    ) -> Result<DeliveryOutcome, &'static str> {
        let sink = {
            let state = self.inner.lock().map_err(|_| "Agent输入通道不可用")?;
            let mut sessions = state
                .sessions
                .values()
                .filter(|(_, sink)| sink.agent_id == agent_id)
                .map(|(_, sink)| sink.clone());
            let sink = sessions.next().ok_or("任务归属Agent未连接")?;
            if sessions.next().is_some() {
                return Err("任务归属Agent存在多个会话");
            }
            sink
        };
        let now = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "系统时间不可用")?
                .as_millis(),
        )
        .map_err(|_| "系统时间不可用")?;
        let id = format!(
            "input_{}_{}",
            std::process::id(),
            self.next_input.fetch_add(1, Ordering::Relaxed) + 1
        );
        Ok(submit(
            &sink,
            &id,
            &sink.session_id,
            AgentInputSource::Voice,
            content,
            now,
        ))
    }

    pub fn deliver_replan(
        &self,
        agent_id: &str,
        task_id: &str,
        sequence: &str,
        reason: &str,
    ) -> Result<DeliveryOutcome, &'static str> {
        self.deliver_replan_with_experiences(agent_id, task_id, sequence, reason, &[])
    }

    pub fn deliver_replan_with_experiences(
        &self,
        agent_id: &str,
        task_id: &str,
        sequence: &str,
        reason: &str,
        experiences: &[String],
    ) -> Result<DeliveryOutcome, &'static str> {
        if !yonder_application::valid_id(task_id)
            || sequence.parse::<u64>().is_err()
            || reason.is_empty()
            || reason.len() > 160
            || reason.contains('\0')
            || experiences.len()
                > yonder_application::execution_experience::MAX_RECALLED_EXPERIENCES
            || experiences.iter().any(|value| {
                value.is_empty()
                    || value.len() > 512
                    || value.chars().any(|character| character.is_control())
            })
        {
            return Err("重新规划唤醒内容无效");
        }
        let sink = {
            let state = self.inner.lock().map_err(|_| "Agent输入通道不可用")?;
            let mut sessions = state
                .sessions
                .values()
                .filter(|(_, sink)| sink.agent_id == agent_id && sink.supports_replan)
                .map(|(_, sink)| sink.clone());
            let sink = sessions.next().ok_or("任务归属Agent未连接")?;
            if sessions.next().is_some() {
                return Err("任务归属Agent存在多个会话");
            }
            sink
        };
        let now = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| "系统时间不可用")?
                .as_millis(),
        )
        .map_err(|_| "系统时间不可用")?;
        let suffix = self.next_input.fetch_add(1, Ordering::Relaxed) + 1;
        let id = format!("replan_{}_{}", std::process::id(), suffix);
        let recalled = if experiences.is_empty() {
            "未召回到可用的已验证执行经验。".into()
        } else {
            format!(
                "已召回以下历史验证经验（仅为策略提示，不是当前事实）：{}。",
                experiences
                    .iter()
                    .enumerate()
                    .map(|(index, value)| format!("{}. {value}", index + 1))
                    .collect::<Vec<_>>()
                    .join("；")
            )
        };
        let content = format!(
            "Yonder任务 {task_id} 在序号 {sequence} 已交回慢脑：{reason}。{recalled}请先通过现有Gateway读取task.get与task.events取得新鲜事实，再结合提示重新规划并提交新的受限计划片段；不要自动重试结果待核实的副作用。"
        );
        Ok(submit(
            &sink,
            &id,
            &sink.session_id,
            AgentInputSource::Replan,
            &content,
            now,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_gateway_presence_from_input_capability() {
        let hub = AgentInputHub::default();
        assert_eq!(hub.presence(), AgentConnectionPresence::default());
        let (gateway_token, presence) = hub.register_gateway("agent-a".into());
        assert_eq!(
            presence,
            AgentConnectionPresence {
                gateway_connected: true,
                input_connected: false
            }
        );
        let (tx, _rx) = tokio_mpsc::unbounded_channel();
        let (close_tx, _close_rx) = tokio_mpsc::unbounded_channel();
        let (session_token, _) = hub.register(
            "agent-a".into(),
            "session-a".into(),
            false,
            false,
            false,
            tx,
            close_tx,
        );
        assert_eq!(
            hub.presence(),
            AgentConnectionPresence {
                gateway_connected: true,
                input_connected: true
            }
        );
        hub.unregister("session-a", session_token);
        assert_eq!(
            hub.presence(),
            AgentConnectionPresence {
                gateway_connected: true,
                input_connected: false
            }
        );
        assert_eq!(
            hub.unregister_gateway(gateway_token),
            AgentConnectionPresence::default()
        );
    }

    #[test]
    fn routes_to_the_registered_agent_connection() {
        let hub = AgentInputHub::default();
        let (first, mut first_rx) = tokio_mpsc::unbounded_channel();
        let (second, mut second_rx) = tokio_mpsc::unbounded_channel();
        let (first_close, _) = tokio_mpsc::unbounded_channel();
        let (second_close, _) = tokio_mpsc::unbounded_channel();
        let (first_token, connected) = hub.register(
            "agent-a".into(),
            "session-a".into(),
            false,
            false,
            false,
            first,
            first_close,
        );
        assert!(connected && hub.connected());
        let (second_token, _) = hub.register(
            "agent-b".into(),
            "session-b".into(),
            true,
            false,
            false,
            second,
            second_close,
        );
        let worker = std::thread::spawn(move || {
            assert!(first_rx.try_recv().is_err());
            let delivery = second_rx.blocking_recv().unwrap();
            assert_eq!(delivery.input.session_id, "session-b");
            assert!(delivery.attachment.is_none());
            delivery.reply.send(DeliveryOutcome::Accepted).unwrap();
            let delivery = second_rx.blocking_recv().unwrap();
            assert_eq!(delivery.input.source, AgentInputSource::Selection);
            assert!(delivery.input.attachment_id.is_some());
            assert_eq!(delivery.attachment.unwrap().bytes, b"x");
            delivery.reply.send(DeliveryOutcome::Accepted).unwrap();
        });
        assert_eq!(
            hub.deliver_for_agent("agent-b", "你好"),
            Ok(DeliveryOutcome::Accepted)
        );
        assert_eq!(hub.deliver_voice("你好"), Err("请选择接收输入的Agent"));
        assert!(hub.unregister("session-a", first_token));
        assert_eq!(
            hub.deliver_selection("看这里", "eA==")
                .map(|value| value.outcome),
            Ok(DeliveryOutcome::Accepted)
        );
        worker.join().unwrap();
        assert!(!hub.unregister("session-b", second_token));
        assert!(!hub.connected());
    }

    #[test]
    fn selection_text_does_not_require_attachment_support() {
        let hub = AgentInputHub::default();
        let (tx, mut rx) = tokio_mpsc::unbounded_channel();
        let (close_tx, _) = tokio_mpsc::unbounded_channel();
        hub.register(
            "agent-a".into(),
            "session-a".into(),
            false,
            false,
            false,
            tx,
            close_tx,
        );
        let worker = std::thread::spawn(move || {
            let delivery = rx.blocking_recv().unwrap();
            assert_eq!(delivery.input.source, AgentInputSource::Selection);
            assert!(delivery.input.attachment_id.is_none() && delivery.attachment.is_none());
            delivery.reply.send(DeliveryOutcome::Accepted).unwrap();
        });
        assert_eq!(
            hub.deliver_selection_text("仅文字")
                .map(|value| value.outcome),
            Ok(DeliveryOutcome::Accepted)
        );
        worker.join().unwrap();
    }

    #[test]
    fn oversized_selection_is_rejected_before_delivery() {
        let hub = AgentInputHub::default();
        let (tx, mut rx) = tokio_mpsc::unbounded_channel();
        let (close_tx, _) = tokio_mpsc::unbounded_channel();
        hub.register(
            "agent-a".into(),
            "session-a".into(),
            true,
            false,
            true,
            tx,
            close_tx,
        );
        let image = base64::engine::general_purpose::STANDARD.encode(vec![
            0;
            MAX_AGENT_ATTACHMENT_BYTES + 1
        ]);
        assert_eq!(
            hub.deliver_selection("看这里", &image),
            Err("截图超过Agent附件上限")
        );
        assert!(rx.try_recv().is_err());
        assert!(hub.inner.lock().unwrap().reply.is_none());
    }

    #[test]
    fn reply_stream_is_correlated_ordered_and_terminal() {
        let hub = AgentInputHub::default();
        let (tx, mut rx) = tokio_mpsc::unbounded_channel();
        let (close_tx, _) = tokio_mpsc::unbounded_channel();
        hub.register(
            "agent-a".into(),
            "session-a".into(),
            false,
            false,
            true,
            tx,
            close_tx,
        );
        let worker = std::thread::spawn(move || {
            let delivery = rx.blocking_recv().unwrap();
            assert_eq!(delivery.input.reply_mode, Some(AgentReplyMode::Stream));
            delivery.reply.send(DeliveryOutcome::Accepted).unwrap();
        });
        let delivery = hub.deliver_selection_text("解释这里").unwrap();
        worker.join().unwrap();
        assert!(delivery.reply_expected);
        let reply = |sequence, phase, session: &str| AgentReplyParams {
            input_id: delivery.input_id.clone(),
            session_id: session.into(),
            reply_id: "turn-1".into(),
            sequence,
            phase,
            content: (phase == AgentReplyPhase::Delta).then(|| "回答".into()),
            truncated: false,
        };
        assert!(!hub.accept_reply(&reply(1, AgentReplyPhase::Started, "other-session")));
        assert!(hub.accept_reply(&reply(1, AgentReplyPhase::Started, "session-a")));
        assert!(!hub.accept_reply(&reply(1, AgentReplyPhase::Delta, "session-a")));
        assert!(hub.accept_reply(&reply(2, AgentReplyPhase::Delta, "session-a")));
        assert!(hub.accept_reply(&reply(3, AgentReplyPhase::Completed, "session-a")));
        assert!(!hub.accept_reply(&reply(4, AgentReplyPhase::Completed, "session-a")));
    }

    #[test]
    fn disconnect_converts_pending_reply_to_interrupted() {
        let hub = AgentInputHub::default();
        let (tx, mut rx) = tokio_mpsc::unbounded_channel();
        let (close_tx, _) = tokio_mpsc::unbounded_channel();
        hub.register(
            "agent-a".into(),
            "session-a".into(),
            false,
            false,
            true,
            tx,
            close_tx,
        );
        let worker = std::thread::spawn(move || {
            rx.blocking_recv()
                .unwrap()
                .reply
                .send(DeliveryOutcome::Accepted)
                .unwrap()
        });
        let delivery = hub.deliver_selection_text("解释这里").unwrap();
        worker.join().unwrap();
        let interrupted = hub.interrupt_reply("session-a").unwrap();
        assert_eq!(interrupted.input_id, delivery.input_id);
        assert_eq!(interrupted.phase, AgentReplyPhase::Interrupted);
        assert_eq!(interrupted.sequence, 1);
        assert!(hub.interrupt_reply("session-a").is_none());
    }

    #[test]
    fn replan_wake_targets_only_the_unique_owner_session() {
        let hub = AgentInputHub::default();
        let (tx, mut rx) = tokio_mpsc::unbounded_channel();
        let (close_tx, _) = tokio_mpsc::unbounded_channel();
        hub.register(
            "agent-a".into(),
            "session-a".into(),
            false,
            true,
            false,
            tx,
            close_tx,
        );
        let worker = std::thread::spawn(move || {
            let delivery = rx.blocking_recv().unwrap();
            assert_eq!(delivery.input.source, AgentInputSource::Replan);
            assert!(
                delivery.input.content.contains("task_1")
                    && delivery.input.content.contains("序号 8")
            );
            assert!(delivery.attachment.is_none());
            delivery.reply.send(DeliveryOutcome::Accepted).unwrap();
        });
        assert_eq!(
            hub.deliver_replan("agent-a", "task_1", "8", "需要慢脑重新 Observe 或规划"),
            Ok(DeliveryOutcome::Accepted)
        );
        assert_eq!(
            hub.deliver_replan("agent-b", "task_1", "8", "需要慢脑重新 Observe 或规划"),
            Err("任务归属Agent未连接")
        );
        worker.join().unwrap();
    }

    #[test]
    fn replan_wake_injects_bounded_experience_before_planning() {
        let hub = AgentInputHub::default();
        let (tx, mut rx) = tokio_mpsc::unbounded_channel();
        let (close_tx, _) = tokio_mpsc::unbounded_channel();
        hub.register(
            "agent-a".into(),
            "session-a".into(),
            false,
            true,
            false,
            tx,
            close_tx,
        );
        let worker = std::thread::spawn(move || {
            let delivery = rx.blocking_recv().unwrap();
            let recall = delivery.input.content.find("历史验证经验").unwrap();
            let fresh = delivery.input.content.find("task.get").unwrap();
            assert!(recall < fresh);
            assert!(delivery.input.content.contains("visual-double-click"));
            delivery.reply.send(DeliveryOutcome::Accepted).unwrap();
        });
        assert_eq!(hub.deliver_replan_with_experiences("agent-a","task_1","8","需要慢脑重新规划",&["策略=visual-double-click；适用前置=observe-valid=true；成功条件=action-succeeded=true；历史验证=1次".into()]),Ok(DeliveryOutcome::Accepted));
        worker.join().unwrap();
    }

    #[test]
    fn disconnect_agent_removes_its_sessions() {
        let hub = AgentInputHub::default();
        let (tx, mut rx) = tokio_mpsc::unbounded_channel();
        let (close_tx, mut close_rx) = tokio_mpsc::unbounded_channel();
        hub.register(
            "agent-a".into(),
            "session-a".into(),
            false,
            false,
            false,
            tx,
            close_tx,
        );
        assert!(hub.connected());
        assert!(!hub.disconnect_agent("agent-a"));
        assert!(!hub.connected());
        assert!(rx.try_recv().is_err());
        assert!(close_rx.try_recv().is_ok());
    }
}
