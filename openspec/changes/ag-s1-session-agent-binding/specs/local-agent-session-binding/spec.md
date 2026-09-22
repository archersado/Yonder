# 本地 Agent 会话身份绑定 Delta

## ADDED Requirements

### Requirement: 首帧绑定

系统 MUST 要求本地 UDS 连接的第一帧为 `gateway.hello`，并以该帧的有效 `agent_id` 绑定该连接的 GatewaySession。

#### Scenario: hello绑定会话

- **WHEN** 本地UDS连接第一帧为携带有效`agent_id`的`gateway.hello`
- **THEN** GatewaySession绑定该Agent，并允许后续任务访问

### Requirement: 连接内身份一致性

系统 MUST 拒绝同一连接后续帧中的不同 `agent_id`。不同连接 MAY 使用不同有效 Agent ID，任务访问仍 MUST 按既有归属规则隔离。

#### Scenario: 连接内更换身份被拒绝

- **WHEN** 同一连接后续帧携带不同`agent_id`
- **THEN** Gateway拒绝该帧，且不改变首帧绑定的会话身份

### Requirement: CLI 显式身份

`yonder mcp` MUST 要求调用方提供有效的 `YONDER_AGENT_ID`，并 MUST NOT 内置或默认使用 `codex-cli`。

#### Scenario: 缺少环境变量拒绝启动

- **WHEN** 调用`yonder mcp`时未提供有效`YONDER_AGENT_ID`
- **THEN** CLI结构化报错且不建立Gateway连接
