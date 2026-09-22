# 步骤边界停止

## ADDED Requirements

### Requirement: observed边界
系统必须只为当前完整身份的observed attempt确认停止；prepared、unknown、旧attempt/Worker/host均不得产生停止事实。

#### Scenario: 非observed请求不停止

- **WHEN** attempt为prepared或unknown，或身份、Worker、host任一不匹配
- **THEN** 不生成停止事实，任务保持占用

### Requirement: 原子停止
暂停/接管必须原子提交running→paused，取消必须原子提交running→cancelled并保留数据；任务状态、停止记录、事件和Outbox必须同事务。

#### Scenario: 事务失败保留运行状态

- **WHEN** 停止事务提交失败
- **THEN** running状态、停止记录、事件和Outbox均不产生部分提交

### Requirement: 先提交后释放
执行Permit只能在停止事务提交成功后释放；任何失败必须交还原Permit并保持占用。

#### Scenario: 事务回滚后恢复Permit

- **WHEN** 停止事务失败
- **THEN** Permit交还原持有者，任务继续显示占用

### Requirement: 子范围边界
停止成功不表示工作已定位、Recording已开始或任务已恢复；这些行为必须由后续独立事实驱动。

#### Scenario: 停止后未定位状态

- **WHEN** 匹配控制的observed边界停止提交成功
- **THEN** 任务停止并释放Permit，但不因此声明工作定位或恢复完成
