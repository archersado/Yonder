# Agent Session Targeting Delta

## ADDED Requirements

### Requirement: 在线会话可显式选择
Yonder MUST 在多个可接收输入的在线 AgentSession 并存时提供本地显式选择入口，且 MUST NOT 猜测目标。

#### Scenario: 多会话未选择
- **WHEN** 第二个会话连接且当前目标仅由唯一会话自动产生
- **THEN** 系统清除自动目标并拒绝输入
- **AND** Agent 管理展示所有当前在线会话的选择入口
- **AND** 保留输入的圈选卡直接提供打开 Agent 管理的入口

#### Scenario: 用户选择目标
- **WHEN** 用户选择一个仍在线的不透明 `session_ref`
- **THEN** 后续圈选与语音输入只投递到该会话
- **AND** 非目标会话不接收正文或附件
- **AND** 原圈选卡提示可以重新发送且不丢失草稿或截图

### Requirement: 自动选择与断连稳定
Yonder MUST 自动选择唯一在线会话，并在显式目标有效期间保持选择；失效引用不得继续投递。

#### Scenario: 非目标断连
- **WHEN** 用户已显式选择目标且其他会话断连
- **THEN** 当前目标保持不变

#### Scenario: 目标断连
- **WHEN** 当前目标断连
- **THEN** 系统立即清除该引用
- **AND** 只剩一个会话时自动选择，否则继续要求用户选择

### Requirement: 会话身份最小披露
Yonder MUST 只向本地 UI 暴露进程内不透明引用和有界摘要；完整 session/thread ID 与选择状态 MUST NOT 持久化或进入日志。

#### Scenario: Agent 管理展示会话
- **WHEN** Agent 管理读取在线会话
- **THEN** 每行只显示 Agent ID、进程内会话序号、能力和当前目标
- **AND** 不显示完整 session/thread ID
