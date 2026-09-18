# 执行尝试准备

## ADDED Requirements

### Requirement: 可信完整执行身份
系统必须只把已接受的当前step_id绑定到Application生成的attempt_id、worker_instance_id和host_session_id；不得把Agent请求ID或自报结果作为尝试身份。

### Requirement: 原子准备
created→running、Start事件、Outbox和prepared尝试必须在同一事务提交；失败不得改变任何一项。同一完整身份重试不得增加事件，不同身份或已有活动尝试必须拒绝。

### Requirement: 准入保守释放
资源准入后准备失败必须释放本次从未派发的Permit；准备成功后Permit在安全停止确认前不得释放或通过Drop清除。

### Requirement: 子范围边界
prepared不能表示动作已派发、Observe有效或安全停止。真实Driver、控制、定位、Recording及Windows验证仍须后续规格。
