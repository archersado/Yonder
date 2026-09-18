# TM-S5 首批只读时间线

Story：TM-S5；来源：产品简报任务时间线要求及 TM5-AC01/03/08。

在现有轻量 Task Space 详情中展示正式 `task.events` 已提交的状态、步骤声明和执行结果，让用户能查看任务实际进展并识别 unknown。

Architecture Impact：conforming。复用现有 Rust 协议、Application 查询、SQLite 事件和桌面可信窗口；不新增协议、迁移、依赖或状态所有者，不实现产物、用户确认、录制、清理或完整历史分页。
