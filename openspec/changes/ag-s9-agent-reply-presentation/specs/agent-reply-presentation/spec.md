# Agent Reply Presentation Delta

## ADDED Requirements

### Requirement: 回复能力显式协商

需要回复回显的输入 MUST 只发送到声明 `user_input_reply` 的同一已认证 AgentSession。

#### Scenario: 圈选输入请求回复

- **WHEN** 当前唯一 Agent session 声明回复能力且用户提交圈选问题
- **THEN** 输入携带流式回复模式和稳定 input ID
- **AND** accepted 只表示 Agent 已接收输入

#### Scenario: Agent 不支持回复

- **WHEN** Agent 支持输入但未声明回复能力
- **THEN** 圈选输入仍可按原契约发送
- **AND** Yonder 明确提示用户到 Agent 客户端查看回答

### Requirement: 同一 Gateway 回传有界回复

Connector MUST 通过同一 AgentSession 把匹配输入的最终 Agent 消息按严格序号回传，不建立旁路。

#### Scenario: 成功回答

- **WHEN** App Server 为关联 turn 发布最终 Agent 消息增量，并以 completed 结束
- **THEN** Connector 发送 started、零到多个 delta 和唯一 completed
- **AND** Yonder 只向匹配 input/session 的当前圈选卡展示

#### Scenario: 非成功终态

- **WHEN** turn failed、interrupted、连接断开或通知不合法
- **THEN** Yonder 显示失败或中断
- **AND** 不把 input accepted 或已有部分正文显示为完成

#### Scenario: 回复超时

- **WHEN** 关联回复在 5 分钟内未到达终态
- **THEN** Connector 回传唯一 `interrupted`
- **AND** Yonder 释放本次临时关联而不补抓历史正文

### Requirement: 回复不持久化

回复正文 MUST 只在当前运行时与圈选卡内有界存在。

#### Scenario: 完成、关闭或重启

- **WHEN** 用户关闭卡片、会话终止或进程重启
- **THEN** Yonder 释放问题、截图和回复正文
- **AND** 不从 SQLite、任务事件、Outbox、日志、索引或 Codex 私有历史恢复

### Requirement: UI 有界且可诊断

圈选卡 MUST 区分发送、等待回复、流式回答、完成、失败、中断和截断。

#### Scenario: 回复达到上限

- **WHEN** 累计可见回复达到 64 KiB
- **THEN** 后续正文不再追加
- **AND** 卡片显示“回答过长，已截断”而不是静默丢失
