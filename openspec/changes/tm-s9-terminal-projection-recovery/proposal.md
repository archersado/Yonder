# Proposal：TM-S9 终态异步投影恢复

关联 Story：TM-S9；前置架构：已接受的事件驱动 Runtime 与 SQLite/事件/Outbox 原子 projector。

2026-10-13 正式 MCP 样本在 Runtime 中返回 `cancelled@5`，但宿主重启后恢复为 `interrupted@5`。这证明终态事件没有在有界时间内形成持久 checkpoint；仅用 `task.list` Runtime 覆盖会隐藏而不会修复该缺口。

本 Change 修复 projector 的终态重试/排空与有界诊断，覆盖取消、暂停、完成、失败和 HandBack。Driver、Observe、快脑和 Gateway 响应继续只依赖内存 Runtime，不把 SQLite 写回同步关键路径。

Windows 继续按用户决定暂缓；本 Change 只在 macOS 正式 `Yonda.app → MCP → Gateway` 建立独立 Goal。
