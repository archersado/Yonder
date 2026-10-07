# Proposal：CU-S5 目标状态验证闭环

关联 CU-S5 与 Accepted AD-TM-24。Architecture Impact：architecture-change（协议、任务完成语义、运行时事实与事后投影）。

## Why

动作回执、动作后 Observe 和用户目标完成目前没有独立门禁，导致计划中间界面可能被错误终结为 completed。

## What Changes

- 新增 sequence-bound 的 `task.goal.verify` Gateway 命令与协议生成物。
- 计划片段耗尽后显式等待目标核验。
- `task.complete` 必须引用最新 achieved 核验。
- ExecutionRuntime 持有活动核验事实，SQLite 异步投影并为兼容链提供检查点。
- 顶部浮窗展示等待、通过和交回状态。
- 增加 Domain、协议、Gateway、SQLite、Runtime 与 macOS 正式验证。

## Non-goals

不暴露模型思维链，不让 Jev 或 Driver判断开放式用户目标，不自动重试 unknown 副作用，不引入第二状态机或同步数据库推进。
