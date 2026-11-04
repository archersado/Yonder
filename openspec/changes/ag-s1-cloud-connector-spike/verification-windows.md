# 独立 Verification Goal：Windows 云端 Connector 传输

日期：2026-10-23
结论：FAIL（外部可信 WSS 回显不稳定；完整 Change 未通过）

- Windows Rust 样本首次运行完成可信 TLS、Ping/Pong、主动关闭后重连、无效证书拒绝、65536 字节上限和有界退避断言。
- 随后两次相同运行均在可信 WSS 回显阶段超时，无法得到可重复的 Windows PASS。
- 本地隔离 WSS 协议夹具另在 AG-S5 范围通过；它不替代系统信任根和公网连接验证。
- 失败证据：[`result.json`](../../../spikes/cloud-connector/evidence/windows-20261023/result.json)、[`result-retry.json`](../../../spikes/cloud-connector/evidence/windows-20261023/result-retry.json)。

因此不勾选 Windows Verification Goal，不接受 AD-AG-06，也不授权产品 Connector 实施。
