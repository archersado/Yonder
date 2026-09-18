# Browser Reference Delta Spec

## ADDED Requirements

### Requirement: Browser引用与Observe同事务

成功Bridge结果 MUST 与attempt observed、任务事件、Outbox和当前Browser引用同事务提交；失败 MUST 全部回滚。

#### Scenario: 重启读取

- **WHEN** Task Space创建结果提交并重新打开数据库
- **THEN** 相同任务返回相同external_task_ref、所有权、页数和更新时间序号

#### Scenario: unknown不覆盖

- **WHEN** 后续Browser动作结果为unknown
- **THEN** attempt记录unknown，当前Browser引用保持上一次已提交事实
