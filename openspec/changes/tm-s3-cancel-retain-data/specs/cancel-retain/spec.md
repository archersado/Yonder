# 取消保留数据

## ADDED Requirements

### Requirement: 无数据清理入口
卡片取消只改变合法状态，全部保留任务和历史；不得发task.delete或清理说明/事件/Outbox/幂等。

#### Scenario: 取消后保留任务事实

- **WHEN** 用户从任务卡片取消合法任务
- **THEN** 仅提交取消状态，任务、说明、事件、Outbox和幂等记录全部保留

### Requirement: 安全兼容
实验格式4仅无删除标记且全transition事件时同事务回3，全部记录和序号不变；带标记/坏格式拒绝不修改。不新建清理格式4。

#### Scenario: 实验格式安全回退

- **WHEN** SQLite格式4不含删除标记且全部transition事件有效
- **THEN** 同一事务回退到格式3，全部记录和sequence不变

#### Scenario: 不安全格式拒绝迁移

- **WHEN** 格式4含删除标记或transition事件校验失败
- **THEN** 迁移拒绝，数据库记录不变
