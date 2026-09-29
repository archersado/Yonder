# Delta Spec：活动执行运行时

## Requirement: 活动任务由事件循环推进

系统 MUST 由 Application 单一有界事件循环拥有活动任务状态。Driver 结果与 Observe 事件被 reducer 接受后 MUST 立即推进或 HandBack，不得等待 SQLite/Outbox 投影。

CUA、BUA、Document/Office、Command 与取消/接管/交回 MUST 使用同一个 Runtime Port 和 reducer；系统 MUST NOT 为单个能力保留独立活动状态机或长期同步数据库执行路径。

### Scenario: 持久化延迟不阻塞动作链

- **GIVEN** SQLite projector 被可控延迟
- **WHEN** Driver 返回 confirmed 且 Observe 有效
- **THEN** 当前步骤立即完成并允许 reducer 评估下一步骤
- **AND** SQLite 稍后按相同逻辑序号原子写入快照、事件和 Outbox

### Scenario: 硬背压在下一副作用前暂停

- **GIVEN** 持久化积压达到硬限
- **WHEN** 当前 Driver 与 Observe 已完成
- **THEN** 保留结果并禁止派发下一副作用
- **AND** 显示 persistence-backpressure，不丢弃或重试动作

### Scenario: 活动查询不回退旧 checkpoint

- **GIVEN** 内存逻辑序号领先 SQLite checkpoint
- **WHEN** Gateway 或 UI 查询活动任务
- **THEN** 返回 runtime 快照和连续内存尾部
- **AND** SQLite 旧快照不得覆盖活动状态

### Scenario: 崩溃后保守恢复

- **WHEN** 进程在部分事件尚未 checkpoint 时退出
- **THEN** 重启只读取最后完整 checkpoint
- **AND** 遗留 running 转 interrupted，未确认副作用不得自动重试

### Scenario: 确认门禁保持前置

- **WHEN** 下一动作属于发送、支付、删除、安装或提权
- **THEN** runtime 必须先取得本机确认事件
- **AND** 不得先执行再补确认
