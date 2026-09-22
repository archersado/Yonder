# 本地 Agent 注册与撤权

## ADDED Requirements

### Requirement: 本地 Agent 注册与撤权

Yonder MUST 提供显式登记、禁用和撤权本地 Agent 的能力。

#### Scenario: 显式登记

用户可以在本地设置中显式登记 Agent，并查看其状态和最近连接时间。

#### Scenario: 撤权即时生效

撤权后，该 Agent 的活动会话立即终止，不能再新建任务、提交步骤、控制任务或接收输入。

#### Scenario: 保留既有任务

撤权后，该 Agent 的既有任务保持可见，用户仍可查看和终止，不自动删除。

### Requirement: 不新增认证边界

本能力 MUST 不改变现有 OS 用户私有端点认证边界。

#### Scenario: 不新增传输认证

本地 Agent 注册与撤权不引入新的本地凭据存储、第二传输认证机制或云端登录流程。
