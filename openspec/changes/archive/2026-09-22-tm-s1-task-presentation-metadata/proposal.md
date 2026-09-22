# Proposal：TM-S1 任务展示元数据

关联 Story：TM-S1；关联决策：Partially Accepted AD-TM-01（AC10 子范围）。

## Why

任务详情缺少来源、观察摘要和下一步意图的统一契约，当前只能显示名称、状态和步骤声明，不能用空字段或 UI 概括冒充真实进度。

## What Changes

扩展任务展示快照与详情查询，增加可信来源、当前步骤、观察摘要和下一步意图；列表保持有界摘要。新增分作者写入用例与 SQLite schema 15 迁移，所有接受事实同事务更新当前值、任务序号、事件和 Outbox。旧版本剥离新增字段，历史缺失值映射为 `legacy` 或 `null`。

Architecture Impact：architecture-change（协议与 SQLite schema 14→15）。不实现自动脱敏、自动规划、任意 metadata patch、历史保留清理或产物版本。
