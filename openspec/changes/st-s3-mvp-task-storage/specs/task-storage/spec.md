# 任务存储增量

## ADDED Requirements

### Requirement: 显式MVP未加密任务存储

Adapter SHALL提供不需要密钥的SQLite打开入口并复用现有TaskStore。

#### Scenario: 持久化与恢复

- WHEN可信调用方创建/更新任务并重开未加密库
- THEN任务、归属、事件及Outbox持久化，已有恢复用例将running转interrupted
- AND不自动执行任务

#### Scenario: 事务失败

- WHEN事件或Outbox写入失败
- THEN状态、事件及Outbox全部回滚

#### Scenario: 已有库不兼容

- WHEN未加密入口读取加密库、错误格式或未知版本
- THEN返回存储不可用，保留原文件，不自动回退或覆盖
