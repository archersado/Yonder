# EX-S2 macOS CUA 计划片段执行基线

关联 EX-S2 三份设计、Accepted AD-AG-09（受限 macOS CUA 基线）、AD-EX-02、AD-TM-08/13。Architecture Impact：architecture-change（Gateway 协议、Application 执行协调、SQLite 片段事实与任务事件/Outbox）。

归属 Agent 经既有 Gateway 提交绑定任务、版本、CAS、截止时间、预算和 CUA 候选的不可变计划片段。Application 仅在已 Observe 的边界调用 Jev；Jev 只能从已提交候选选择，低置信、偏离、取消、接管、用户输入、`unknown` 或预算耗尽均产生可观察的交回依据，经事件/Outbox 返回归属 Agent。每次 Gateway 执行只消费一个槽位，不创建后台循环、第二会话、第二状态机或旁路 Driver。

范围只限 macOS CUA。不得实现 Recipe、自由 DSL、批处理、BUA/Document/Command 片段或 Windows 注册。任务完成和失败仍只能由归属 Agent 经现有 Gateway 提交。
