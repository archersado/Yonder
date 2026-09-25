# TM-S5 历史序号缺口检测

Story：TM-S5。来源：产品简报「MVP 主干链路」第 9 步、「Task Space 与权限模型」及 TM5-AC08。决策：Accepted AD-TM-16。

当前任务历史不做部分裁剪；`task.events` 对已授权读取的页检查逐任务序号连续性。缺口不能被正常空页或后续事件掩盖。

Architecture Impact：conforming。复用 Rust 协议现有失败包络和时间线局部错误 UI；不新增协议字段、SQLite 迁移、依赖或自动修复。产物身份、总配额及清理不属于本 Change。
