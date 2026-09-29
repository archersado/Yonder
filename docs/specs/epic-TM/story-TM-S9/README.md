# TM-S9 事件驱动执行运行时

状态：design-ready。依据 [AD-TM-23](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-23-EVENT-DRIVEN-EXECUTION-RUNTIME.md)，活动任务不再由同步 SQLite 写入驱动。OpenSpec：[`tm-s9-event-driven-runtime`](../../../../openspec/changes/tm-s9-event-driven-runtime/)。

完成范围同时包含 CUA、EX-S2 计划片段、BUA、Document/Office、Command 和控制链路。允许按能力分阶段提交，但任一链路仍依赖同步 SQLite 推进时不得完成或 Archive。Windows继续暂缓。
