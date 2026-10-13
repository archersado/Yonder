# 设计

2026-10-13 回归补充：Local Socket 仅对已解码写请求发布普通展示事件；Task Space 和 Agent 的只读查询不发布展示事件，防止滞后的 SQLite 检查点覆盖 Runtime 终态。

`TaskHost::presentation()` 对存储候选叠加 ExecutionRuntime 快照；Runtime 终态屏蔽旧检查点，活动步骤标签优先取 Runtime，运行时查询异常返回 unknown。所有 `yonda-presentation` 与连接状态事件经 Cocoa 主线程投递。

本机 Task Space 的 `task.list/get/events` 通过 Application Gateway 的共享 Runtime overlay 编码响应，与外部 Agent 查询保持同源，不在 React/desktop 复制覆盖规则。

当同一任务在已 Observe 成功但尚未显式推进的步骤上开始下一次尝试，Application Runtime 先追加 `StepBoundaryAdvanced`，再追加 `StepStarted`；projector 因此能停止旧 attempt 后写入新 attempt，并继续投影终态。

Application 用请求类型限制事件来源，只接受 `task.complete` 与 `browser.execute(finish)`；再从成功响应读取终态快照，输出 `state` 与 `task_id:sequence:status` 内部标识。desktop 终态时只发布一个包含终态和事实标识的 `yonda-presentation` 事件，不把同一次 Gateway 返回附近取得的展示快照当作稍后恢复依据。

pet 前端按标识去重，以单个 1.8 秒定时器控制终态展示；到期通过既有 `pet_task_state` 读取 Runtime 优先的最新派生状态。普通展示事件取消终态定时器；异步读取返回前到达的新事件通过代际号使旧结果失效；读取失败回退 `unknown` 并清除步骤。该单次刷新不是状态轮询。隐藏态沿用既有状态变化唤醒。读取、拒绝、存储错误与非终态结果不生成脉冲。当前公开能力只能真实产生成功；失败判定保留在同一共享出口，等待既有 Application 失败提交能力获得正式 Gateway 写入口。
