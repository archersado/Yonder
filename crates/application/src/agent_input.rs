pub use yonder_protocol::{AgentAttachmentBeginParams, AgentAttachmentChunkParams, AgentAttachmentFinishParams, AgentAttachmentMime, AgentInputParams, AgentInputSource, AgentRequest, AgentResponse, Version};

pub fn registration(bytes:&[u8])->Option<(String,String,bool)>{yonder_protocol::input_registration(bytes)}
pub fn hello_accepted(bytes:&[u8])->bool{matches!(yonder_protocol::decode_response(bytes),Ok(yonder_protocol::Response::Success{..}))}
pub fn encode_request(request:&AgentRequest)->Result<Vec<u8>,yonder_protocol::RpcError>{yonder_protocol::encode_agent_request(request).map_err(|_|yonder_protocol::RpcError::new(-32603,"Agent输入编码失败"))}
pub fn decode_response(bytes:&[u8])->Result<AgentResponse,yonder_protocol::RpcError>{yonder_protocol::decode_agent_response(bytes).map_err(|_|yonder_protocol::RpcError::new(-32700,"Agent确认无效"))}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryOutcome { Accepted, Rejected, Unknown }

pub struct AgentInputAttachment { pub mime: AgentAttachmentMime, pub bytes: Vec<u8> }

pub trait AgentInputSink {
    fn deliver(&self, input: AgentInputParams, attachment: Option<AgentInputAttachment>) -> DeliveryOutcome;
}

pub fn submit(
    sink: &dyn AgentInputSink,
    input_id: &str,
    session_id: &str,
    source: AgentInputSource,
    content: &str,
    now_ms: u64,
) -> DeliveryOutcome {
    let input = AgentInputParams {
        input_id: input_id.into(), session_id: session_id.into(), source,
        content: content.trim().into(), attachment_id: None, created_at: now_ms, deadline: now_ms.saturating_add(10_000),
    };
    if input.validate(now_ms).is_err() { return DeliveryOutcome::Rejected; }
    sink.deliver(input, None)
}

pub fn submit_selection(
    sink: &dyn AgentInputSink,
    input_id: &str,
    attachment_id: &str,
    session_id: &str,
    content: &str,
    bytes: Vec<u8>,
    now_ms: u64,
) -> DeliveryOutcome {
    let input = AgentInputParams {
        input_id: input_id.into(), session_id: session_id.into(), source: AgentInputSource::Selection,
        content: content.trim().into(), attachment_id: Some(attachment_id.into()), created_at: now_ms,
        deadline: now_ms.saturating_add(60_000),
    };
    if input.validate(now_ms).is_err() || bytes.is_empty() || bytes.len() > yonder_protocol::MAX_AGENT_ATTACHMENT_BYTES as usize {
        return DeliveryOutcome::Rejected;
    }
    sink.deliver(input, Some(AgentInputAttachment { mime: AgentAttachmentMime::ImagePng, bytes }))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Sink;
    impl AgentInputSink for Sink { fn deliver(&self, _: AgentInputParams, _: Option<AgentInputAttachment>) -> DeliveryOutcome { DeliveryOutcome::Accepted } }

    #[test]
    fn input_is_bounded_before_delivery() {
        assert_eq!(submit(&Sink,"input_1","session_1",AgentInputSource::Voice," 你好 ",1000),DeliveryOutcome::Accepted);
        assert_eq!(submit(&Sink,"input_2","session_1",AgentInputSource::Voice," ",1000),DeliveryOutcome::Rejected);
        assert_eq!(submit(&Sink,"input_3","session_1",AgentInputSource::Voice,&"a".repeat(16*1024+1),1000),DeliveryOutcome::Rejected);
        assert_eq!(submit_selection(&Sink,"input_4","image_4","session_1","看这里",vec![1],1000),DeliveryOutcome::Accepted);
        assert_eq!(submit_selection(&Sink,"input_5","image_5","session_1"," ",vec![1],1000),DeliveryOutcome::Rejected);
        assert_eq!(submit_selection(&Sink,"input_6","image_6","session_1","看这里",vec![0;4*1024*1024+1],1000),DeliveryOutcome::Rejected);
    }
}
