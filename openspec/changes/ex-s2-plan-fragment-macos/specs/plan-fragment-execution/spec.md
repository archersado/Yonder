# 计划片段执行增量规格

## ADDED Requirements

### Requirement: unknown 交回允许新片段重新观察

系统 SHALL 在 Driver 动作结果为 `unknown` 时保留原始结论、原因与可用 Observe 证据，不推进旧片段游标，也不自动重试副作用。交回事件、Outbox 与独立 handback 边界 SHALL 同事务提交，使已结束的旧 attempt 不再占据步骤执行槽；归属慢脑读取 `task.get` 与 `task.events` 后，MAY 通过原 Gateway 以新的 step/attempt 提交受限片段。

#### Scenario: unknown 后提交重新观察片段

- **WHEN** 当前槽位落库为 `unknown` 并完成 handback，归属慢脑读取最新任务与事件后提交新的重新观察片段
- **THEN** 系统保留旧 attempt 的 unknown 结论且旧片段游标不变
- **AND** 新片段以新的 step/attempt 身份通过既有授权、租约和 Observe 门禁执行，不因旧 attempt 永久占槽而拒绝

#### Scenario: 未完成交回不得跨越 unknown

- **WHEN** unknown 结果尚未形成有效 handback 边界，或调用方尝试复用旧 step/attempt
- **THEN** 系统拒绝新副作用，不把 unknown 提升为成功，也不自动重放原动作
