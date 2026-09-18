# 独立 Verification Goal：macOS 云端 Connector 传输

日期：2026-09-18  
结论：PASS（macOS Spike 子范围；完整 Change 未通过）

- Rust 单进程客户端通过系统信任根完成可信 WSS 回显和 Ping/Pong。
- 客户端配置 65536 字节消息及帧上限；边界断言通过。
- 主动关闭后按 250 毫秒退避重新建立连接；退避上限及抖动边界断言通过。
- 自签名 TLS 测试端点在握手层被拒绝。
- 样本未启动第二进程、未持久化凭据、未注册产品 `AgentSession`。
- 结构化证据：[`result.json`](../../../spikes/cloud-connector/evidence/macos-20260918/result.json)。

Windows 统一样本按用户决定暂缓，AD-AG-06 保持 Proposed；外部平台配对、端点、设备凭据与 Outbox 恢复仍是产品 Connector 前置门禁。
