# Execution Attempt Delta Spec

## ADDED Requirements

### Requirement: Observed attempt 可普通推进

系统 MUST 只在当前attempt为observed、身份与sequence匹配且没有pending控制时，原子记录普通步骤边界并保持任务running。

#### Scenario: 连续两步

- **WHEN** 第一步完成后置Observe并提交普通边界，Agent声明第二步
- **THEN** 第二attempt可在同一任务资源Permit下准备，两个边界均有递增事件和Outbox

#### Scenario: 不安全状态拒绝

- **WHEN** attempt为prepared或unknown，或已有pending控制
- **THEN** 不写任务、事件、Outbox或attempt停止事实
