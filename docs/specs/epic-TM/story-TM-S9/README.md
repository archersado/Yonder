# TM-S9 事件驱动执行运行时

Story: TM-S9
Epic: TM
Status: verifying
OpenSpec: tm-s9-event-driven-runtime

状态：verifying。依据 [AD-TM-23](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-23-EVENT-DRIVEN-EXECUTION-RUNTIME.md)，活动任务不再由同步 SQLite 写入驱动。OpenSpec：[`tm-s9-event-driven-runtime`](../../../../openspec/changes/tm-s9-event-driven-runtime/)。

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

2026-10-08 已完成 EX-S2 `task.plan.execute` 迁移：不可变片段仅在执行入口读取一次，连续槽位、游标与 HandBack 由同一内存 reducer 推进，SQLite/events/Outbox 通过连续事件异步投影；测试证明 projector 尚未 ACK 时两个槽位仍可连续执行。

同日完成运行时控制与活动查询迁移：正式 Desktop 组合根的取消、暂停、接管、HandBack 均直接进入同一 reducer；顶部控制条沿 Driver 派发路径即时显示 effect/Observe 分类，Gateway `task.get` 与 `task.events` 会把未 checkpoint 的内存快照及连续尾部覆盖到旧 SQLite 投影之上。

完成范围同时包含 CUA、EX-S2 计划片段、BUA、Document/Office、Command 和控制链路。允许按能力分阶段提交，但任一链路仍依赖同步 SQLite 推进时不得完成或 Archive。Windows继续暂缓。
