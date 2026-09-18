pub use yonder_protocol::{AgentInputParams, AgentInputSource, AgentRequest, AgentResponse, Version};

pub fn registration(bytes:&[u8])->Option<(String,String)>{yonder_protocol::input_registration(bytes)}
pub fn hello_accepted(bytes:&[u8])->bool{matches!(yonder_protocol::decode_response(bytes),Ok(yonder_protocol::Response::Success{..}))}
pub fn encode_request(request:&AgentRequest)->Result<Vec<u8>,yonder_protocol::RpcError>{yonder_protocol::encode_agent_request(request).map_err(|_|yonder_protocol::RpcError::new(-32603,"Agent输入编码失败"))}
pub fn decode_response(bytes:&[u8])->Result<AgentResponse,yonder_protocol::RpcError>{yonder_protocol::decode_agent_response(bytes).map_err(|_|yonder_protocol::RpcError::new(-32700,"Agent确认无效"))}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryOutcome { Accepted, Rejected, Unknown }

pub trait AgentInputSink {
    fn deliver(&self, input: AgentInputParams) -> DeliveryOutcome;
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
        content: content.trim().into(), created_at: now_ms, deadline: now_ms.saturating_add(10_000),
    };
    if input.validate(now_ms).is_err() { return DeliveryOutcome::Rejected; }
    sink.deliver(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Sink;
    impl AgentInputSink for Sink { fn deliver(&self, _: AgentInputParams) -> DeliveryOutcome { DeliveryOutcome::Accepted } }

    #[test]
    fn input_is_bounded_before_delivery() {
        assert_eq!(submit(&Sink,"input_1","session_1",AgentInputSource::Voice," 你好 ",1000),DeliveryOutcome::Accepted);
        assert_eq!(submit(&Sink,"input_2","session_1",AgentInputSource::Voice," ",1000),DeliveryOutcome::Rejected);
        assert_eq!(submit(&Sink,"input_3","session_1",AgentInputSource::Voice,&"a".repeat(16*1024+1),1000),DeliveryOutcome::Rejected);
    }
}
