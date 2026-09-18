# 设计

Application 用请求类型限制事件来源，只接受 `task.complete` 与 `browser.execute(finish)`；再从成功响应读取终态快照，输出 `state` 与 `task_id:sequence:status` 内部标识。desktop 在同一次 Gateway 调用后取得现有 `TaskHost::presentation()`，终态时只发布一个包含恢复状态的 `yonda-presentation` 事件。

pet 前端按标识去重，以单个 1.8 秒定时器恢复事件携带的派生状态。普通展示事件取消终态定时器；隐藏态沿用既有状态变化唤醒。读取、拒绝、存储错误与非终态结果不生成脉冲。当前公开能力只能真实产生成功；失败判定保留在同一共享出口，等待既有 Application 失败提交能力获得正式 Gateway 写入口。
