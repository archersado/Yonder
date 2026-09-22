# task-presentation Specification

## Purpose
定义任务来源、当前步骤、观察摘要和下一步意图的分作者持久化与查询契约，保证详情与列表展示使用已提交事实且不伪造缺失信息。

## Requirements

### Requirement: 分作者展示事实
系统 MUST 只通过可信分作者用例写入任务来源、当前步骤、观察摘要和下一步意图；MUST NOT 提供任意 metadata patch，MUST NOT 由 Agent 或 UI 自报来源或观察结果。

#### Scenario: 创建绑定来源
- Given 归属 Agent 经本地 Gateway 创建任务
- When 创建事务提交
- Then `name/source` 与创建状态、事件和 Outbox 同事务保存，来源为 `local-agent`

#### Scenario: 观察摘要
- Given 当前完整执行尝试已完成可信 Observe
- When 可信执行器提交有界 `observation`
- Then 当前值、递增 sequence、有类型事件和 Outbox 同事务提交；迟到或身份不匹配的观察被拒绝

#### Scenario: 下一步意图
- Given 非终态任务且调用方有控制权限
- When 显式声明 `next_intent`
- Then 只更新该作者字段并同事务递增 sequence、追加事件和 Outbox

### Requirement: 查询投影
`task.get` MUST 返回完整展示快照；`task.list` MUST 返回有界摘要。缺失的展示事实 MUST 为 `null`，MUST NOT 用空字符串或占位文本冒充。

#### Scenario: 详情查询
- Given 任务已有名称、来源、当前步骤、观察摘要和下一步意图
- When 1.19 会话调用 `task.get`
- Then 响应返回这些已提交事实

#### Scenario: 旧会话
- Given 协议版本低于 1.19
- When 查询任务
- Then 新增展示字段不改变旧会话语义

### Requirement: schema 15 迁移
SQLite schema 14 升级到 15 前 MUST 先创建可恢复备份；历史任务 MUST 映射为 `source=legacy` 且其余新增字段为空。

#### Scenario: 历史库迁移
- Given 一个 schema 14 的 SQLite 任务库
- When schema 升级到 15
- Then 升级前创建可恢复备份，历史任务返回 `source=legacy`，其余新增展示字段为 `null`
