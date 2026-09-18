use std::{collections::HashMap, sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}, mpsc}, time::{Duration, SystemTime, UNIX_EPOCH}};
use tokio::sync::mpsc as tokio_mpsc;
use yonder_application::agent_input::{AgentInputParams, AgentInputSink, AgentInputSource, DeliveryOutcome, submit};

pub(crate) struct Delivery { pub input: AgentInputParams, pub reply: mpsc::Sender<DeliveryOutcome> }

#[derive(Clone)]
struct SessionSink { agent_id: String, session_id: String, tx: tokio_mpsc::UnboundedSender<Delivery> }

impl AgentInputSink for SessionSink {
    fn deliver(&self, input: AgentInputParams) -> DeliveryOutcome {
        let (reply, result) = mpsc::channel();
        if self.tx.send(Delivery { input, reply }).is_err() { return DeliveryOutcome::Unknown; }
        result.recv_timeout(Duration::from_secs(11)).unwrap_or(DeliveryOutcome::Unknown)
    }
}

#[derive(Clone, Default)]
pub struct AgentInputHub { inner: Arc<Mutex<HubState>>, next_input: Arc<AtomicU64>, next_session: Arc<AtomicU64> }

#[derive(Default)]
struct HubState { sessions: HashMap<String, (u64, SessionSink)>, active: Option<String> }

impl AgentInputHub {
    pub(crate) fn register(&self, agent_id: String, session_id: String, tx: tokio_mpsc::UnboundedSender<Delivery>) -> (u64, bool) {
        let token = self.next_session.fetch_add(1, Ordering::Relaxed) + 1;
        let connected = if let Ok(mut state) = self.inner.lock() {
            let replacing = state.sessions.contains_key(&session_id);
            state.sessions.insert(session_id.clone(), (token, SessionSink { agent_id, session_id: session_id.clone(), tx }));
            if state.sessions.len() == 1 || replacing && state.active.as_deref() == Some(&session_id) { state.active = Some(session_id); }
            else if !replacing { state.active = None; }
            true
        } else { false };
        (token, connected)
    }

    pub(crate) fn unregister(&self, session_id: &str, token: u64) -> bool {
        if let Ok(mut state) = self.inner.lock() {
            if state.sessions.get(session_id).is_some_and(|(current, _)| *current == token) { state.sessions.remove(session_id); }
            if state.active.as_deref() == Some(session_id) { state.active = state.sessions.keys().next().cloned().filter(|_| state.sessions.len() == 1); }
            !state.sessions.is_empty()
        } else { false }
    }

    pub fn connected(&self) -> bool { self.inner.lock().is_ok_and(|state| !state.sessions.is_empty()) }

    pub fn deliver_voice(&self, content: &str) -> Result<DeliveryOutcome, &'static str> {
        let sink = {
            let state = self.inner.lock().map_err(|_| "Agent输入通道不可用")?;
            let session = state.active.as_ref().ok_or(if state.sessions.is_empty() { "当前没有可接收输入的Agent" } else { "请选择接收输入的Agent" })?;
            state.sessions.get(session).map(|(_, sink)| sink.clone()).ok_or("Agent会话已断开")?
        };
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| "系统时间不可用")?.as_millis()).map_err(|_| "系统时间不可用")?;
        let id = format!("voice_{}_{}", std::process::id(), self.next_input.fetch_add(1, Ordering::Relaxed) + 1);
        Ok(submit(&sink, &id, &sink.session_id, AgentInputSource::Voice, content, now))
    }

    pub fn deliver_for_agent(&self, agent_id: &str, content: &str) -> Result<DeliveryOutcome, &'static str> {
        let sink = {
            let state = self.inner.lock().map_err(|_| "Agent输入通道不可用")?;
            let mut sessions = state.sessions.values().filter(|(_, sink)| sink.agent_id == agent_id).map(|(_, sink)| sink.clone());
            let sink = sessions.next().ok_or("任务归属Agent未连接")?;
            if sessions.next().is_some() { return Err("任务归属Agent存在多个会话"); }
            sink
        };
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| "系统时间不可用")?.as_millis()).map_err(|_| "系统时间不可用")?;
        let id = format!("input_{}_{}", std::process::id(), self.next_input.fetch_add(1, Ordering::Relaxed) + 1);
        Ok(submit(&sink, &id, &sink.session_id, AgentInputSource::Voice, content, now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_to_the_registered_agent_connection() {
        let hub=AgentInputHub::default();
        let (first,mut first_rx)=tokio_mpsc::unbounded_channel();
        let (second,mut second_rx)=tokio_mpsc::unbounded_channel();
        let (first_token, connected)=hub.register("agent-a".into(),"session-a".into(),first);
        assert!(connected && hub.connected());
        let (second_token, _)=hub.register("agent-b".into(),"session-b".into(),second);
        let worker=std::thread::spawn(move||{
            assert!(first_rx.try_recv().is_err());
            let delivery=second_rx.blocking_recv().unwrap();
            assert_eq!(delivery.input.session_id,"session-b");
            delivery.reply.send(DeliveryOutcome::Accepted).unwrap();
        });
        assert_eq!(hub.deliver_for_agent("agent-b","你好"),Ok(DeliveryOutcome::Accepted));
        assert_eq!(hub.deliver_voice("你好"),Err("请选择接收输入的Agent"));
        worker.join().unwrap();
        assert!(hub.unregister("session-a",first_token));
        assert!(!hub.unregister("session-b",second_token));
        assert!(!hub.connected());
    }
}
