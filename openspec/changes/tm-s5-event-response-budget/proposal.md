# TM-S5 任务事件响应字节预算

Story：TM-S5。来源：产品简报任务时间线与 TM5-AC08。决策：Accepted AD-TM-15。

为既有 `task.events(after_sequence, limit)` 加入编码后的事件和整响应字节预算。页仍按实际返回的最后序号继续，不改 SQLite 事件、Outbox 或 UI 状态所有者。

Architecture Impact：architecture-change（Application 查询契约）。无迁移和新依赖；Rust 协议常量定义预算，旧版本在投影后受同一预算。产物清单、附件配额及清理不属于本 Change。
