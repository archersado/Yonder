# Design：临时观察元素句柄

Sky Worker 每次动作后从新鲜 transcript 中选择可操作角色，生成有界句柄，并为该集合生成绑定任务与 attempt 的不透明 `observation_ref`。Adapter与Gateway只做类型映射，不解析原始 AX 文本。

下一计划片段若使用元素索引，必须同时携带该引用。Worker 先确认引用是同任务最近集合，再重新Observe并确认索引仍是可操作元素；随后仅把数字索引转换为Sky内部参数。任一校验失败均不派发点击，返回临时Observation交回慢脑。

句柄只出现在调用响应与下一计划参数，生命周期不跨 Worker 重建。SQLite、事件、Outbox、Runtime快照、顶部浮窗与日志都不保存元素内容或引用。
