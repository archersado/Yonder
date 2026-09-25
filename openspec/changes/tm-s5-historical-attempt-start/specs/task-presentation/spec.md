# task-presentation Delta

## ADDED Requirements

### Requirement: prepared 尝试按原 Start 事件投影

Yonder MUST 只把 `accepted_sequence` 与 Start 事件相同的执行尝试投影为 `attempt_started`，并保留完整不可变执行身份；MUST NOT 从当前 phase、running 状态或最终结果补造开始事实。

#### Scenario: 尝试开始与结果分别保留

- **GIVEN** 尝试已准备，随后提交观察结果
- **WHEN** 查询历史
- **THEN** prepared 开始事实位于原 Start 序号
- **AND** 最终结果位于自己的结果序号，不覆盖开始事实

#### Scenario: 旧任务没有尝试记录

- **GIVEN** 任务状态曾为 running 但没有 task_attempts 记录
- **WHEN** 查询历史
- **THEN** 不生成 `attempt_started`

### Requirement: 尝试开始历史按协议和授权隔离

Yonder MUST 只对协议 1.25 的已授权读取输出 `attempt_started`；1.24 及以下 MUST 缺省该字段，并保持连续性、分页和编码预算。

#### Scenario: 旧协议读取 Start 事件

- **GIVEN** 事件存在 prepared 尝试
- **WHEN** 协商协议 1.24
- **THEN** 事件仍返回但不包含 `attempt_started`

#### Scenario: 非归属 Agent 查询

- **GIVEN** 调用 Agent 不拥有该任务
- **WHEN** 查询事件
- **THEN** 返回不可见错误且不泄露执行身份

### Requirement: Task Space 不夸大 prepared 语义

Task Space MUST 显示步骤与尝试标识并称为“执行尝试已准备”，MUST NOT 显示 Worker/宿主内部标识或把它描述为已派发、成功、Observe 有效或安全停止。

#### Scenario: 查看尝试开始事件

- **GIVEN** `attempt_started` 关联 step-one 与 attempt-one
- **WHEN** 用户查看时间线
- **THEN** 显示“执行尝试已准备（步骤 step-one · 尝试 attempt-one）”
- **AND** 不显示 worker_instance_id 或 host_session_id
