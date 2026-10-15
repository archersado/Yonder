# Agent Session Targeting Delta

## ADDED Requirements

### Requirement: 输入会话单活抢占
Yonder MUST 同时只保留一个可接收用户输入的 AgentSession，新的已认证会话 MUST 抢占并关闭旧会话。

#### Scenario: 后连抢占
- **WHEN** 当前已有输入会话且新的已认证输入会话注册
- **THEN** Hub 原子地把新会话设为唯一目标
- **AND** 向旧会话发出关闭信号
- **AND** 后续圈选与语音输入不得再进入旧会话

#### Scenario: 同会话重连
- **WHEN** 相同 session 标识以新连接 token 重连
- **THEN** 新连接抢占旧连接
- **AND** 旧连接延迟 unregister 不得删除新连接

#### Scenario: 被抢占 Connector 终止
- **WHEN** 新会话注册并抢占当前旧会话
- **THEN** Yonder 向旧 Connector 发送 `agent.session.close` 且原因为 `preempted`
- **AND** 旧 Connector 终止绑定，不得进入自动重连并再次竞争活动槽位

#### Scenario: Yonder 重启仍可恢复
- **WHEN** Connector 因 EOF、连接重置或连接拒绝失去 Yonder，而未收到 `preempted`
- **THEN** Connector 继续使用既有有界退避重连
- **AND** 不重放任何未确认输入或副作用

### Requirement: 抢占保持回复隔离
Yonder MUST 在抢占时释放旧会话的短生命回复关联，并 MUST NOT 把旧回复通知投影到新会话。

#### Scenario: 回复进行中发生抢占
- **WHEN** 旧会话存在未完成回复且新会话注册
- **THEN** 旧回复关联被清除并显示中断
- **AND** 旧会话后续 delta 被拒绝

### Requirement: 不暴露会话路由
Yonder MUST NOT 要求用户选择 Agent 输入会话，也 MUST NOT 在 Desktop UI 暴露会话引用或列表。

#### Scenario: Agent 管理展示
- **WHEN** 用户打开 Agent 管理
- **THEN** 页面只显示 Agent 注册授权与语音回复设置
- **AND** 不显示在线会话、当前接收目标或选择按钮

#### Scenario: 唯一会话不可用
- **WHEN** 当前没有活动输入会话
- **THEN** 圈选或语音输入显示通道不可用并保留草稿
- **AND** 不提示用户选择会话
