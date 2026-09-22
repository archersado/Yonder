# 外部控制请求

## ADDED Requirements

### Requirement: pending不是停止
task.control成功 MUST 只表示控制已登记并正在停止；不得显示已接管、已定位或已记录。

#### Scenario: pending返回不终止任务

- **WHEN** 归属Agent或LocalUser提交匹配条件的控制请求
- **THEN** 调用成功返回pending，任务仍保持running且不产生停止事实

### Requirement: 原子与幂等
pending控制、任务sequence、事件和Outbox MUST 同事务；相同控制重投 MUST 幂等，不同控制 MUST 冲突。

#### Scenario: 相同控制重投

- **WHEN** 同一任务重复提交相同控制
- **THEN** 不新增控制、事件或Outbox记录，调用返回既有请求状态

### Requirement: 授权与冻结
LocalUser MUST 可控制已有running任务，Agent MUST 仅所属；pending后 MUST NOT 派发新动作，只有匹配控制的边界停止事务才能完成并释放Permit。

#### Scenario: pending后拒绝新动作

- **WHEN** 任务已有pending控制且请求派发新动作
- **THEN** 新动作被拒绝，直到匹配控制的停止事务提交并释放Permit
