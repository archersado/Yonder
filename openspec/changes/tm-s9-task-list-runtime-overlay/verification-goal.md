# Verification Goal：TM-S9 活动任务列表 Runtime 覆盖

Result: Pending

使用正式 macOS `Yonda.app → MCP stdio → Agent Gateway` 创建并进入活动 Runtime 的无用户数据任务。在 SQLite projector 尚未覆盖最新终态的窗口读取 `task.get`、`task.events` 与 `task.list`，三者必须返回同一 sequence/status；`running_only` 不得保留已取消任务，`include_finished=false` 同样不得泄漏终态。

自动化覆盖页内覆盖、旧 Runtime 不倒退 Store、新任务分页游标稳定、Runtime 不可用时历史列表不变。Windows按用户决定暂缓，不外推。
