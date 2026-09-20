# 本地 Agent 会话身份绑定 Delta

## ADDED Requirements

### Requirement: 首帧绑定

系统 MUST 要求本地 UDS 连接的第一帧为 `gateway.hello`，并以该帧的有效 `agent_id` 绑定该连接的 GatewaySession。

### Requirement: 连接内身份一致性

系统 MUST 拒绝同一连接后续帧中的不同 `agent_id`。不同连接 MAY 使用不同有效 Agent ID，任务访问仍 MUST 按既有归属规则隔离。

### Requirement: CLI 显式身份

`yonder mcp` MUST 要求调用方提供有效的 `YONDER_AGENT_ID`，并 MUST NOT 内置或默认使用 `codex-cli`。
