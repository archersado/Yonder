# 步骤边界停止

## ADDED Requirements

### Requirement: observed边界
系统必须只为当前完整身份的observed attempt确认停止；prepared、unknown、旧attempt/Worker/host均不得产生停止事实。

### Requirement: 原子停止
暂停/接管必须原子提交running→paused，取消必须原子提交running→cancelled并保留数据；任务状态、停止记录、事件和Outbox必须同事务。

### Requirement: 先提交后释放
执行Permit只能在停止事务提交成功后释放；任何失败必须交还原Permit并保持占用。

### Requirement: 子范围边界
停止成功不表示工作已定位、Recording已开始或任务已恢复；这些行为必须由后续独立事实驱动。
