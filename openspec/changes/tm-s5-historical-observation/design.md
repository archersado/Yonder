# 设计

Adapter 的 `events_with_steps` 对每条 `events` 以相同 task_id/sequence 左联结 `task_presentation_events.kind='observation'`，严格解析既有 `{step_id,result,summary}`。读取畸形 payload 返回存储错误，不静默降级。Application 在当前授权和事件连续性通过后，将该事实映射到复用的 `TaskObservation` 协议类型；只有 1.21 会话与可信同版本查询输出。旧 1.20 会话不包含新字段，字节预算仍在投影后计量。

Task Space 的时间线按优先级展示 Observe 事实，文字标出步骤、匹配结果和获准摘要；纯文本渲染且无新窗口。既有分页从实际最后事件序号继续，错误局部显示并保留已加载内容。

验证用两个真实 Observe 事务及不同摘要证明历史不可由当前快照代替；包括 unknown、旧版投影、畸形 payload、授权、预算和原生 UI。Windows 验证按用户要求暂缓。
