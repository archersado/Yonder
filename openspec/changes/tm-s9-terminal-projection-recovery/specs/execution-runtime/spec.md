## ADDED Requirements

### Requirement: 终态事件最终形成可恢复 checkpoint

系统 MUST 让 Runtime 已接受的终态或 HandBack 事件在不阻塞执行响应的前提下由 projector 持续重试，直至状态、事件与 Outbox 原子提交并 ACK；正常宿主关闭必须有界排空已接受事件。

#### Scenario: 取消后重启仍保持取消

- **WHEN** 活动任务通过正式控制入口在 Runtime 中进入取消终态
- **THEN** Gateway 立即返回取消状态，projector 异步提交连续事件，正常重启后任务仍为同一取消序号而不是 `interrupted`

#### Scenario: 投影短暂失败不丢失终态

- **WHEN** SQLite projector 对终态事件首次返回不可用或冲突
- **THEN** Worker 保留该任务为 dirty、按有界间隔重试且不 ACK 未提交事件，不阻塞 Driver 或 Gateway 响应

#### Scenario: 诊断不泄露任务内容

- **WHEN** 终态投影超过健康期限仍未成功
- **THEN** 系统只输出错误分类和有界计数，不记录任务正文、动作参数、截图、输入或完整事件 Payload
