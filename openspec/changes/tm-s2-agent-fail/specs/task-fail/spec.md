# task.fail 增量规格

## Requirement: 只终结已观察的确定失败

系统必须只允许归属 Agent 把最新已停止且动作确定失败的 Desktop 任务提交为 failed。

### Scenario: 确定失败

- Given 最新 attempt 已 Observe 为 action_succeeded=false，并已推进到 stopped
- When 归属 Agent 以当前序号调用 task.fail
- Then failed、事件和 Outbox 原子提交，随后释放 Desktop 资源并发布一次 failed 展示

### Scenario: 不确定或未停止

- Given attempt 为 prepared、observed 未停止、unknown、动作成功，或存在 pending control
- When Agent 调用 task.fail
- Then 请求拒绝，任务、事件、Outbox和资源占用不变

### Scenario: 读取与旧协议

- Given 已有 failed 历史任务或协议 1.17 会话
- When 查询历史或调用 task.fail
- Then 查询不重播动画，旧会话拒绝写入且不产生记录
