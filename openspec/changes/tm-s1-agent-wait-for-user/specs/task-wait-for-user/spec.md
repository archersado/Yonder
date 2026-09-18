# task.wait_for_user 增量规格

## Requirement: 安全边界等待

系统必须只允许归属 Agent 在运行任务的已 Observe、已推进步骤边界提交等待用户状态。

### Scenario: 成功提交

- Given 任务为 running，最后 attempt 为无控制请求的 stopped
- When Agent 以当前序号和非空原因调用 `task.wait_for_user`
- Then 状态、带原因事件和 Outbox 同事务提交，随后释放任务占用

### Scenario: 未知或活动动作

- Given attempt 未停止、结果 unknown 或存在 pending control
- When Agent 请求等待
- Then 请求被拒绝，状态和占用不变

## Requirement: 协议兼容

1.17 会话必须能写入并读取 `wait_reason`；旧会话必须拒绝写入且读取事件时不含该字段。
