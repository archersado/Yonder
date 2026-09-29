# Proposal：TM-S9 事件驱动执行运行时

关联 Story TM-S9 与 Accepted AD-TM-23。正式废除活动任务依赖同步 SQLite 逐步推进的旧约束，把活动 CUA/计划片段迁移到 Application 单一有界事件循环；SQLite/events/Outbox 改为异步连续投影。

Architecture Impact：architecture-change。改变活动状态所有者与持久化时序；不改变 Gateway 入口、Driver 边界、确认规则或历史 SQLite schema，不引入完整 Event Sourcing。
