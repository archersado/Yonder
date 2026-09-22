# 执行尝试准备

## ADDED Requirements

### Requirement: 可信完整执行身份
系统 MUST 只把已接受的当前step_id绑定到Application生成的attempt_id、worker_instance_id和host_session_id；不得把Agent请求ID或自报结果作为尝试身份。

#### Scenario: 拒绝Agent自报身份

- **WHEN** Agent提交自报的attempt_id、Worker或host作为执行身份
- **THEN** 系统拒绝该身份，不生成prepared尝试

### Requirement: 原子准备
created→running、Start事件、Outbox和prepared尝试 MUST 在同一事务提交；失败不得改变任何一项。同一完整身份重试 MUST NOT 增加事件，不同身份或已有活动尝试 MUST 拒绝。

#### Scenario: 准备事务失败

- **WHEN** prepared尝试事务提交失败
- **THEN** 任务状态、Start事件、Outbox和prepared尝试均不改变

### Requirement: 准入保守释放
资源准入后准备失败 MUST 释放本次从未派发的Permit；准备成功后Permit在安全停止确认前 MUST NOT 释放或通过Drop清除。

#### Scenario: 准备失败释放未派发Permit

- **WHEN** 资源准入后准备事务失败
- **THEN** 释放本次Permit，不派发动作且不增加事件

### Requirement: 子范围边界
prepared MUST NOT 表示动作已派发、Observe有效或安全停止。真实Driver、控制、定位、Recording及Windows验证仍须后续规格。

#### Scenario: prepared不显示动作完成

- **WHEN** 尝试仅完成prepared事务
- **THEN** 不声明动作已派发、Observe有效或安全停止
