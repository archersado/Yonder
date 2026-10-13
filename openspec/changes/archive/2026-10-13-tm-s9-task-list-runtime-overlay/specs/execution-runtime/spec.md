# Delta Spec：活动任务列表 Runtime 覆盖

## ADDED Requirements

### Requirement: 活动任务列表不得回退旧 checkpoint

系统 MUST 对 `task.list` 返回页内的活动任务应用不旧于持久项的 Runtime 快照。状态筛选 MUST 在覆盖后执行；系统 MUST NOT 因 SQLite checkpoint 落后，把已取消、暂停、完成、失败或中断的活动任务显示为 running。

#### Scenario: 已取消任务不再显示执行中

- **GIVEN** Runtime 已接受取消并领先 SQLite checkpoint
- **WHEN** Agent 读取 `task.get`、`task.events` 与 `task.list`
- **THEN** 三个入口返回相同的最新取消序号与状态
- **AND** `running_only` 与 `include_finished=false` 不返回该任务

#### Scenario: 列表分页保持稳定

- **GIVEN** SQLite 已返回一个有界列表页及下一页游标
- **WHEN** Gateway 应用 Runtime 覆盖
- **THEN** 只覆盖页内同 ID 项并保留既有游标
- **AND** 不从 Runtime 额外注入未在当前页中的任务
