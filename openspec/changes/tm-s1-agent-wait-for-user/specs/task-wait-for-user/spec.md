# task.wait_for_user 增量规格

## ADDED Requirements

### Requirement: 安全边界等待

系统 MUST 只允许归属 Agent 在运行任务的已 Observe、已推进步骤边界提交等待用户状态。

#### Scenario: 成功提交

- Given 任务为 running，最后 attempt 为无控制请求的 stopped
- When Agent 以当前序号和非空原因调用 `task.wait_for_user`
- Then 状态、带原因事件和 Outbox 同事务提交，随后释放任务占用

#### Scenario: 未知或活动动作

- Given attempt 未停止、结果 unknown 或存在 pending control
- When Agent 请求等待
- Then 请求被拒绝，状态和占用不变

### Requirement: 协议兼容

1.17 会话 MUST 能写入并读取 `wait_reason`；旧会话 MUST 拒绝写入且读取事件时不含该字段。

#### Scenario: 1.17 读写

- **WHEN** 1.17 会话提交等待用户请求并读取对应事件
- **THEN** 请求可以携带非空 `wait_reason`，事件也包含该字段

#### Scenario: 旧会话兼容

- **WHEN** 低于 1.17 的会话提交携带 `wait_reason` 的请求并读取对应事件
- **THEN** 写入被拒绝，读取结果不包含该字段
