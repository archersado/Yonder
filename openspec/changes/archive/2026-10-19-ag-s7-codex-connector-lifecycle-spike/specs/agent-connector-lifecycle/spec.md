# Agent Connector Lifecycle Spike Delta

## ADDED Requirements

### Requirement: 当前会话显式绑定证据

Spike MUST 证明连接器绑定由当前 Agent 会话通过受支持接口明确提供，不依赖推断。

#### Scenario: 会话绑定

- **WHEN** Codex 客户端声明可接收 Yonder 输入
- **THEN** 证据包含受支持的会话标识来源与 endpoint 来源
- **AND** 不按最近会话、工作目录、进程或单一候选猜测
- **AND** 不读取内部数据库、私有队列或附着私有 stdio

### Requirement: 生命周期恢复矩阵

Spike MUST 使用统一样本验证连接器的冷启动、断线、重启和会话切换。

#### Scenario: Yonder 或连接器重启

- **WHEN** Yonder GUI、Gateway/MCP 或连接器重启
- **THEN** 任务通道与输入通道分别报告真实状态
- **AND** 仅对同一明确会话采用有界退避恢复
- **AND** 未确认输入和任务副作用均不自动重放

#### Scenario: Codex 重启或 thread 切换

- **WHEN** Codex 宿主重启或当前 thread 改变
- **THEN** 旧绑定永久失效
- **AND** 新会话必须主动注册后才可接收输入

### Requirement: 客户端能力不得外推

Spike MUST 分别验证 Codex CLI 与 ChatGPT Desktop，不把一种客户端的接口能力外推到另一种客户端。

#### Scenario: 客户端不支持

- **WHEN** 客户端没有公开的当前会话双向接口
- **THEN** 结论标记为 Unsupported 或 Deferred
- **AND** 不以私有 API、进程注入或第二 Agent 绕过

### Requirement: 终局实施门禁

Spike MUST 在产品实现前产出终局 ADR 与后续 Story 验收映射。

#### Scenario: 平台证据完成或明确延期

- **WHEN** 某个平台的统一样本证据完成，或该平台由用户明确决定延期
- **THEN** AD-AG-10 必须分别标记该平台路线为 Accepted、Rejected 或 Deferred
- **AND** 只有对应平台的 Accepted 路线可以进入后续产品 Story 实施，其他平台不得由其结果外推
