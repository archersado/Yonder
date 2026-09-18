# AG-S5 Agent Input Spike

目标：验证Yonder可通过Agent Runtime已经建立的双向会话主动发送`agent.input`，不依赖MCP、不启动第二个Agent，也不把具体Agent命令写入Yonder核心。

统一样本：Agent声明`user_input`与稳定`session_id`；Yonder向同一连接发送一条语音来源输入；覆盖accepted、拒绝、断连、超时及重复`input_id`。日志只输出传输、绑定和结果，不输出正文。

```bash
python3 spikes/agent-input/duplex_probe.py
node spikes/agent-input/wss_probe.mjs
```

淘汰门槛：不能稳定绑定原会话、需要Yonder识别Agent厂商、需要第二Agent进程、无法明确确认接收，任一成立即淘汰候选。macOS使用UDS；云端Runtime使用现有WSS双工信道；Windows必须以Named Pipe运行同一语义样本后，AD-VI-02才可Accepted。
