# Windows 验证：Agent 输入协议样本

日期：2026-10-23
结论：PASS（隔离协议样本）；BLOCKED（产品链路）

- Windows Named Pipe 完成 `gateway.hello`、`user_input` 能力协商、会话绑定、`agent.input` 和 accepted 确认。
- 本地隔离 WSS 夹具完成相同方向的输入确认；自签名证书仅用于夹具，未作为产品 TLS 结论。
- 修正测试夹具使用 `Start-ThreadJob`，避免进程型 Job 继承管道句柄导致测试退出挂起。
- 证据：[`named-pipe-result.json`](../../../spikes/agent-input/evidence/windows-20261023/named-pipe-result.json)、[`wss-result.json`](../../../spikes/agent-input/evidence/windows-20261023/wss-result.json)。

当前桌面端尚未提供 Windows 产品 Named Pipe，云端产品 `AgentSession` 也未交付；因此 AG-S5 双平台独立 Verification Goal 仍不通过。
