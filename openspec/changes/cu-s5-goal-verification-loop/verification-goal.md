# 独立 Verification Goal：CU-S5 目标状态验证闭环

状态：待执行

## 通过条件

- 无核验、not-achieved、陈旧核验和错误 ID 均不能调用 `task.complete`。
- 最新 achieved 核验可由归属 Agent 完成任务。
- not-achieved 后可提交新计划，unknown 副作用没有自动重试。
- 活动 Runtime 在 SQLite projector 延迟时仍即时返回核验状态。
- 顶部浮窗正确展示等待核验、目标已核验和交回慢脑。
- macOS 正式 Gateway CUA 样本不会把中间面板误判为目标完成。

Windows 对等验证按用户决定暂缓，macOS 证据不得外推。
