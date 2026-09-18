# 执行尝试结果

## ADDED Requirements

### Requirement: 原子结果事实
当前完整attempt的动作/Observe结论必须与任务sequence、追加事件和Outbox同事务提交；提交失败不得留下部分结果。

### Requirement: 结果幂等与身份隔离
相同完整身份及相同结论重投必须返回原事实且不增加事件；不同结论、旧Worker/host/attempt或旧sequence必须拒绝。

### Requirement: unknown保守处理
超时、崩溃、断连、非法回包、身份不符或Observe失败必须记录unknown；不得自动重试动作、释放占用或显示完成。

### Requirement: 版本化Agent可见性
协议1.5必须通过task.events的可选attempt_result提供有界分类事实；旧协议响应不得包含该字段。任何版本均不得返回输入正文、窗口树、截图或完整Driver Payload。
