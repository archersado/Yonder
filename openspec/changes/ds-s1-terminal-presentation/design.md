# 设计

Application 用请求类型限制事件来源，只接受 `task.complete` 与 `browser.execute(finish)`；再从成功响应读取终态快照，输出 `state` 与 `task_id:sequence:status` 内部标识。desktop 终态时只发布一个包含终态和事实标识的 `yonda-presentation` 事件，不把同一次 Gateway 返回附近取得的展示快照当作稍后恢复依据。

pet 前端按标识去重，以单个 1.8 秒定时器控制终态展示；到期通过既有 `pet_task_state` 读取 Runtime 优先的最新派生状态。普通展示事件取消终态定时器；异步读取返回前到达的新事件通过代际号使旧结果失效；读取失败回退 `unknown` 并清除步骤。该单次刷新不是状态轮询。隐藏态沿用既有状态变化唤醒。读取、拒绝、存储错误与非终态结果不生成脉冲。当前公开能力只能真实产生成功；失败判定保留在同一共享出口，等待既有 Application 失败提交能力获得正式 Gateway 写入口。
