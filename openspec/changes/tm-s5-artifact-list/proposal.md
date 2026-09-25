# TM-S5 产物清单读取与 Task Space 展示

Story：TM-S5。来源：产品简报「Task Space 与权限模型」「MVP 主干链路」第 9 步、TM5-AC05/06/08；决策：Accepted AD-TM-22 的 2026-09-25 清单读取协议增量。

在既有不可变清单核心之上增加协议 1.26 `task.artifacts`，让当前有权会话按固定版本和稳定 ordinal 有界读取条目，并在 Task Space 展示可用性、空清单、分页和局部错误。旧协议拒绝新方法；UI 不发布清单、不解析路径、不提供打开产物入口。

Architecture Impact：architecture-change。变更 Rust 协议、Gateway 版本协商和 Task Space 展示；不迁移 SQLite，不改变产物所有权或能力 Adapter 边界。
