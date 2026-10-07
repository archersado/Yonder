# Goal Verification Delta

## ADDED Requirements

### Requirement: 目标完成独立核验

系统 MUST 区分动作成功、步骤效果已 Observe 和用户目标已达成；片段结束不得自动产生任务完成事实。

#### Scenario: 中间界面已打开

- **WHEN** 所有计划槽位已执行，但最新 Observation 只证明企业切换面板已打开
- **THEN** 任务进入等待目标核验，`task.complete` 被拒绝

### Requirement: 核验绑定最新事实

归属 Agent MUST 通过同一 Gateway 提交绑定当前 CAS 和最新 Observation/attempt 结果序号的 achieved/not-achieved 核验。

#### Scenario: 新动作使旧核验失效

- **WHEN** achieved 核验之后又发生新计划或动作事件
- **THEN** 旧 `verification_id` 不能完成任务，必须基于新鲜事实重新核验

### Requirement: not-achieved 交回

系统 MUST 在目标未达成时保持非终态并交回归属慢脑，而不是重放动作或结束任务。

#### Scenario: 企业尚未切换

- **WHEN** 慢脑根据新鲜画面提交 not-achieved
- **THEN** 顶部浮窗显示目标未完成，Gateway允许提交新的受限计划片段

### Requirement: 事件驱动且事后投影

活动任务核验 MUST 先进入 ExecutionRuntime 并即时驱动 Gateway/UI；SQLite 只做异步投影和恢复检查点。

#### Scenario: 数据库暂时延迟

- **WHEN** 核验事件已进入运行时但 projector 尚未确认
- **THEN** UI 可立即显示核验状态，数据库延迟不阻塞 Driver/Observe
