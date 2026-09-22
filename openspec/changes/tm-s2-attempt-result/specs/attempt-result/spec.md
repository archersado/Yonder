# 执行尝试结果

## ADDED Requirements

### Requirement: 原子结果事实
当前完整attempt的动作/Observe结论必须与任务sequence、追加事件和Outbox同事务提交；提交失败不得留下部分结果。

#### Scenario: 结果事务失败

- **WHEN** 尝试结果事务提交失败
- **THEN** 任务sequence、事件、Outbox和结果事实不出现部分写入

### Requirement: 结果幂等与身份隔离
相同完整身份及相同结论重投必须返回原事实且不增加事件；不同结论、旧Worker/host/attempt或旧sequence必须拒绝。

#### Scenario: 相同结果重投

- **WHEN** 同一完整身份以相同结论重投
- **THEN** 返回原事实且不追加事件

### Requirement: unknown保守处理
超时、崩溃、断连、非法回包、身份不符或Observe失败必须记录unknown；不得自动重试动作、释放占用或显示完成。

#### Scenario: 执行器断连

- **WHEN** Driver回包超时、断连或无法解析
- **THEN** 该attempt记录unknown，任务占用保持且不显示完成

### Requirement: 版本化Agent可见性
协议1.5必须通过task.events的可选attempt_result提供有界分类事实；旧协议响应不得包含该字段。任何版本均不得返回输入正文、窗口树、截图或完整Driver Payload。

#### Scenario: 旧协议不返回结果字段

- **WHEN** 低于1.5的会话读取task.events
- **THEN** 响应不包含attempt_result，也不包含受保护正文或截图
