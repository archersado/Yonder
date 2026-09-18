# 独立 Verification Goal

日期：2026-09-17  
结论：PASS（macOS 接管定位增量）

## 验证目标

验证真实 Agent CUA 步骤完成 Observe 后，用户从 Yonda 任务卡片接管时，系统先提交停止事实，再将原任务窗口精确前置；定位阶段进入事实源，失败不改选窗口，且不启动 Recording。

## 结果

- 协议 1.13、SQLite schema 13、生成类型与旧版本投影检查通过。
- Store 测试覆盖 `running/observed → paused/stopped → locating → focused`，事件与 Outbox 随序号同事务提交。
- 真实 UDS Agent 创建任务并经 trycua SDK 执行 `computer.step`；Yonda 卡片“接管”后任务为 `paused`、控制为 `stopped`、定位为 `focused`。
- 原生隔离窗口重新成为前台；共提交 8 条事件，`recording_started=false`。
- Workspace 46 项测试、JavaScript 语法、协议生成检查和 diff 格式检查通过。

结构化证据：[result.json](../../../apps/desktop/evidence/takeover-focus-20260917/result.json)

## 保留边界

本 Goal 只关闭 macOS 当前 Space 的接管定位增量。跨 Space/多显示器、Windows、Recording 与交回 Observe 仍按 TM-S3/TM-S4/RC-S1 后续门禁处理，不据此将完整 Story Archive。
