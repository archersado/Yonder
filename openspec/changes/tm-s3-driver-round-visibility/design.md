# Design：Driver 轮次内存投影

`CuaControlPort::dispatch` 是正式 CUA Driver 的宿主派发边界。调用底层 Port 前，它以 task/step 和安全动作类别通知 Hub 开始一轮；返回后按 `DispatchOutcome` 更新同一轮。投影是 best-effort，不能改变 Driver 返回值、Observe、任务状态、租约或 Gateway 响应。

`CuaControlPresentation` 增加最多六条 `driver_rounds`。每条只含递增轮次、步骤标签、固定动作摘要、固定状态与固定结论。动作摘要由受控 tool name 映射，未知名称统一为“执行桌面动作”；绝不复制 arguments JSON。

Webview 继续通过现有只读 Tauri command 拉取 Hub 快照。日志列表自身滚动；replan 替换计划投影时保留轮次，Hub finish 时清理。
