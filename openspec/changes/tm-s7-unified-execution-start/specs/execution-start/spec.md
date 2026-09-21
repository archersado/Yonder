# 统一执行启动规范

## Requirement：首次副作用前提交执行事实

系统 SHALL 在任一执行能力首次派发副作用前，原子提交任务 `created→running`、步骤、prepared attempt、状态事件及 Outbox。

### Scenario：事务失败

- **WHEN** 启动事务无法提交
- **THEN** 系统不得调用能力 Adapter，任务不得显示为运行中

### Scenario：首次 CUA、BUA、Document 或 Command 执行

- **WHEN** 已授权的 created 任务请求首次执行
- **THEN** 它们使用同一 Application 启动语义，能力仅声明自身资源与派发/Observe Port

## Requirement：未知副作用不自动重试

系统 SHALL 将派发或 Observe 结果不明记录为 unknown，并保持既有停止/接管门禁。

### Scenario：执行器断连

- **WHEN** 副作用结果无法确认
- **THEN** 系统不得自动重新派发，也不得报告完成
