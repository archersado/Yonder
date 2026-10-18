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
    supports_goal_verification: bool,
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
    session: Option<(u64, SessionSink)>,
    gateways: HashMap<u64, String>,
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
    ) -> (u64, bool, Option<AgentReplyParams>) {
        let token = self.next_session.fetch_add(1, Ordering::Relaxed) + 1;
        let mut previous = None;
        let mut interrupted = None;
        let connected = match self.inner.lock() {
            Ok(mut state) => {
                previous = state.session.replace((
                    token,
                    SessionSink {
                        agent_id,
                        session_id: session_id.clone(),
                        supports_attachment,
                        supports_replan,
                        supports_goal_verification: false,
                        supports_reply,
                        tx,
                        close,
                    },
                ));
                if let Some(reply) = state
                    .reply
                    .take()
                    .filter(|reply| reply.session_id != session_id || previous.is_some())
                {
                    interrupted = Some(interrupted_reply(reply));
                }
                true
            }
            Err(_) => false,
        };
        if let Some((_, sink)) = previous {
            let _ = sink.close.send(());
        }
        (token, connected, interrupted)
    }

    pub(crate) fn enable_goal_verification(&self, session_id: &str, token: u64) {
        if let Ok(mut state) = self.inner.lock() {
            if let Some((current, sink)) = state.session.as_mut()
                && *current == token
                && sink.session_id == session_id
            {
                sink.supports_goal_verification = true;
            }
        }
    }

    pub(crate) fn unregister(&self, session_id: &str, token: u64) -> bool {
        if let Ok(mut state) = self.inner.lock() {
            if state
                .session
                .as_ref()
                .is_some_and(|(current, sink)| *current == token && sink.session_id == session_id)
            {
                state.session = None;
                if state
                    .reply
                    .as_ref()
                    .is_some_and(|reply| reply.session_id == session_id)
                {
                    state.reply = None;
                }
            }
            state.session.is_some()
        } else {
            false
        }
    }

    pub fn disconnect_agent(&self, agent_id: &str) -> bool {
        let mut state = match self.inner.lock() {
            Ok(state) => state,
            Err(_) => return false,
        };
        if state
            .session
            .as_ref()
            .is_some_and(|(_, sink)| sink.agent_id == agent_id)
        {
            if let Some((_, sink)) = state.session.take() {
                let _ = sink.close.send(());
            }
        }
        state.session.is_some()
    }

    pub fn connected(&self) -> bool {
        self.presence().input_connected
    }
    pub fn presence(&self) -> AgentConnectionPresence {
        self.inner
            .lock()
            .map(|state| AgentConnectionPresence {
                gateway_connected: !state.gateways.is_empty(),
                input_connected: state.session.is_some(),
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
        state
            .session
            .as_ref()
            .map(|(_, sink)| sink.clone())
            .ok_or("当前没有可接收输入的Agent")
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
        Some(interrupted_reply(reply))
    }

    pub fn deliver_for_agent(
        &self,
        agent_id: &str,
        content: &str,
    ) -> Result<DeliveryOutcome, &'static str> {
        let sink = {
            let state = self.inner.lock().map_err(|_| "Agent输入通道不可用")?;
            state
                .session
                .as_ref()
                .filter(|(_, sink)| sink.agent_id == agent_id)
                .map(|(_, sink)| sink.clone())
                .ok_or("任务归属Agent未连接")?
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

    pub fn deliver_goal_verification(
        &self,
        agent_id: &str,
        task_id: &str,
        sequence: &str,
    ) -> Result<DeliveryOutcome, &'static str> {
        if !yonder_application::valid_id(task_id) || sequence.parse::<u64>().is_err() {
            return Err("终局核验唤醒内容无效");
        }
        let sink = {
            let state = self.inner.lock().map_err(|_| "Agent输入通道不可用")?;
            state.session.as_ref()
                .filter(|(_, sink)| sink.agent_id == agent_id && sink.supports_goal_verification)
                .map(|(_, sink)| sink.clone())
                .ok_or("任务归属Agent未连接")?
        };
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| "系统时间不可用")?.as_millis()).map_err(|_| "系统时间不可用")?;
        let id = format!("goal_verify_{}_{}",std::process::id(),self.next_input.fetch_add(1, Ordering::Relaxed)+1);
        let content=format!("Yonder任务 {task_id} 已完成计划片段并在序号 {sequence} 等待终局核验。请通过现有Gateway读取task.get与task.events取得新鲜事实；仅在最新Observation证明目标达成时提交task.goal.verify并task.complete，否则按事实提交not-achieved并重新规划。不要把Driver回执直接视为目标完成，也不要重试结果待核实的副作用。");
        Ok(submit(&sink,&id,&sink.session_id,AgentInputSource::GoalVerification,&content,now))
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
            state
                .session
                .as_ref()
                .filter(|(_, sink)| sink.agent_id == agent_id && sink.supports_replan)
                .map(|(_, sink)| sink.clone())
                .ok_or("任务归属Agent未连接")?
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

fn interrupted_reply(reply: ReplyState) -> AgentReplyParams {
    AgentReplyParams {
        input_id: reply.input_id.clone(),
        session_id: reply.session_id,
        reply_id: reply.reply_id.unwrap_or(reply.input_id),
        sequence: reply.next_sequence,
        phase: AgentReplyPhase::Interrupted,
        content: None,
        truncated: false,
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
        let (session_token, _, _) = hub.register(
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
        let (first_close, mut first_close_rx) = tokio_mpsc::unbounded_channel();
        let (second_close, _) = tokio_mpsc::unbounded_channel();
        let (first_token, connected, interrupted) = hub.register(
            "agent-a".into(),
            "session-a".into(),
            false,
            false,
            false,
            first,
            first_close,
        );
        assert!(connected && hub.connected() && interrupted.is_none());
        let (second_token, connected, interrupted) = hub.register(
            "agent-b".into(),
            "session-b".into(),
            true,
            false,
            false,
            second,
            second_close,
        );
        assert!(connected && interrupted.is_none());
        assert!(first_close_rx.try_recv().is_ok());
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
            hub.deliver_for_agent("agent-a", "你好"),
            Err("任务归属Agent未连接")
        );
        assert_eq!(hub.deliver_voice("你好"), Ok(DeliveryOutcome::Accepted));
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
    fn reconnect_with_same_session_id_preempts_and_ignores_old_unregister() {
        let hub = AgentInputHub::default();
        let (first_tx, _first_rx) = tokio_mpsc::unbounded_channel();
        let (first_close, mut first_close_rx) = tokio_mpsc::unbounded_channel();
        let (first, _, _) = hub.register(
            "codex-cli".into(),
            "session-a".into(),
            false,
            false,
            false,
            first_tx,
            first_close,
        );
        let (second_tx, _second_rx) = tokio_mpsc::unbounded_channel();
        let (second_close, _) = tokio_mpsc::unbounded_channel();
        let (second, _, _) = hub.register(
            "codex-cli".into(),
            "session-a".into(),
            false,
            false,
            false,
            second_tx,
            second_close,
        );
        assert!(first_close_rx.try_recv().is_ok());
        assert!(hub.unregister("session-a", first));
        assert!(hub.connected());
        assert!(!hub.unregister("session-a", second));
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
    fn preemption_interrupts_old_reply_and_rejects_late_delta() {
        let hub = AgentInputHub::default();
        let (first_tx, mut first_rx) = tokio_mpsc::unbounded_channel();
        let (first_close, mut first_close_rx) = tokio_mpsc::unbounded_channel();
        hub.register(
            "agent-a".into(),
            "session-a".into(),
            false,
            false,
            true,
            first_tx,
            first_close,
        );
        let worker = std::thread::spawn(move || {
            first_rx
                .blocking_recv()
                .unwrap()
                .reply
                .send(DeliveryOutcome::Accepted)
                .unwrap();
        });
        let delivery = hub.deliver_selection_text("解释这里").unwrap();
        worker.join().unwrap();

        let (second_tx, _second_rx) = tokio_mpsc::unbounded_channel();
        let (second_close, _) = tokio_mpsc::unbounded_channel();
        let (_, connected, interrupted) = hub.register(
            "agent-a".into(),
            "session-b".into(),
            false,
            false,
            true,
            second_tx,
            second_close,
        );
        let interrupted = interrupted.unwrap();
        assert!(connected && first_close_rx.try_recv().is_ok());
        assert_eq!(interrupted.input_id, delivery.input_id);
        assert_eq!(interrupted.phase, AgentReplyPhase::Interrupted);
        assert!(!hub.accept_reply(&AgentReplyParams {
            input_id: delivery.input_id,
            session_id: "session-a".into(),
            reply_id: "turn-old".into(),
            sequence: 1,
            phase: AgentReplyPhase::Started,
            content: None,
            truncated: false,
        }));
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
    fn goal_verification_wake_uses_the_owner_session_without_claiming_completion() {
        let hub=AgentInputHub::default();
        let (tx,mut rx)=tokio_mpsc::unbounded_channel();let (close_tx,_)=tokio_mpsc::unbounded_channel();
        let (token,_,_)=hub.register("agent-a".into(),"session-a".into(),false,true,false,tx,close_tx);
        hub.enable_goal_verification("session-a",token);
        let worker=std::thread::spawn(move||{let delivery=rx.blocking_recv().unwrap();assert_eq!(delivery.input.source,AgentInputSource::GoalVerification);assert!(delivery.input.content.contains("task_1")&&delivery.input.content.contains("序号 29")&&delivery.input.content.contains("task.goal.verify"));assert!(!delivery.input.content.contains("目标已完成"));delivery.reply.send(DeliveryOutcome::Accepted).unwrap();});
        assert_eq!(hub.deliver_goal_verification("agent-a","task_1","29"),Ok(DeliveryOutcome::Accepted));
        assert_eq!(hub.deliver_goal_verification("agent-b","task_1","29"),Err("任务归属Agent未连接"));
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
