# Verification Goal：TM-S9 终态异步投影恢复

Result: PASS（macOS，2026-10-13）

使用正式 macOS `Yonda.app → MCP stdio → Agent Gateway` 创建无用户数据任务，进入活动 Runtime 后通过正式控制入口取消。响应必须立即返回终态且不等待 SQLite；随后有界等待 projector 完成 checkpoint，重启 Yonder 后 `task.get/events/list` 仍须返回相同取消序号，不能变为 `interrupted`。

自动化覆盖一次投影失败后的重试、终态控制事件、正常 Drop 排空、SQLite 状态/事件/Outbox 同事务和失败回滚。Windows 按用户决定 Deferred。

## 结果

- 正式链路：`Yonda.app → MCP stdio → Agent Gateway`。
- 验证任务：`task_c378c7490b4f95605ec2b559ef8c50da`。
- CUA worker 在序号 4 返回 `unknown/worker-failed`；正式 `task.control(cancel)` 在序号 5 返回 `cancelled`，未等待 SQLite checkpoint 才响应。
- 优雅退出、换入最新 dev 正式包并重启后，`task.get`、`task.events`、`task.list` 均保持 `cancelled@5`；事件 1–5 连续，序号 4 的未知结论与序号 5 的取消控制事实均可读取，未回退为 `interrupted`。
- Adapter 87 项与 Desktop 45 项通过；两次全量运行各出现一次与本变更无关的既有时序抖动，单项立即复跑通过，目标回归持续通过。
- 结构化证据：`apps/desktop/evidence/tm-s9-terminal-projection-recovery-macos-20261013/result.json`。
- Windows 原生验证按主人决定 Deferred。
