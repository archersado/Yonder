# 创建权限增量

关联AG-S2 AC1/AC2/AC7与AD-AG-01。

## ADDED Requirements

### Requirement: 禁止人工创建旁路

Application创建 MUST 只接受可信Agent身份，不允许LocalUser，即使ID合法。

#### Scenario: 本机用户创建
- WHEN LocalUser调用create
- THEN 存储调用前PermissionDenied，任务/事件/Outbox无新增。

#### Scenario: 合法Agent
- WHEN 可信Agent调用合法内部create
- THEN 既有事务创建正常，归属绑定上下文。

#### Scenario: 本机用户读取已有任务
- WHEN LocalUser读取已有任务
- THEN 保留既有授权读取，不因创建门禁拒绝。
