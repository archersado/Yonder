# 撤回删除方案 Delta

## ADDED Requirements

### Requirement: 撤回的终态删除不得进入产品

被 AD-TM-06 撤回的终态任务清理方案 MUST NOT 实施；取消任务必须保留任务、事件、Outbox 和幂等数据。

#### Scenario: 用户在任务卡片选择删除

- **WHEN** 用户对任务卡片执行当前产品中的删除/取消动作
- **THEN** 系统只能进入已接受的取消并保留数据语义
- **AND** 不得调用本 Change 中已撤回的 `task.delete` 或 SQLite 清理迁移
