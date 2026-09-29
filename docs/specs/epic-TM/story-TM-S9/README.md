# TM-S9 事件驱动执行运行时

状态：design-ready。依据 [AD-TM-23](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-23-EVENT-DRIVEN-EXECUTION-RUNTIME.md)，活动任务不再由同步 SQLite 写入驱动。OpenSpec：[`tm-s9-event-driven-runtime`](../../../../openspec/changes/tm-s9-event-driven-runtime/)。

首阶段迁移 CUA 与 EX-S2 计划片段；验证后以同一端口迁移 BUA、Document、Command。Windows继续暂缓。
