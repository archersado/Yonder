# Verification Goal：TM-S9 终态异步投影恢复

Result: Pending

使用正式 macOS `Yonda.app → MCP stdio → Agent Gateway` 创建无用户数据任务，进入活动 Runtime 后通过正式控制入口取消。响应必须立即返回终态且不等待 SQLite；随后有界等待 projector 完成 checkpoint，重启 Yonder 后 `task.get/events/list` 仍须返回相同取消序号，不能变为 `interrupted`。

自动化覆盖一次投影失败后的重试、终态控制事件、正常 Drop 排空、SQLite 状态/事件/Outbox 同事务和失败回滚。Windows 按用户决定 Deferred。
