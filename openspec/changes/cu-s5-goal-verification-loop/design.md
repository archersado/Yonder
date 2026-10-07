# Design：目标状态验证闭环

归属慢脑在计划片段结束后消费 `task.get`、`task.events` 与最新 Observation，并提交仅含 achieved/not-achieved 的核验。核验绑定产生它之前的最新结果序号；Application 追加新序号并返回 `verification_id`。

完成命令同时携带当前序号与 `verification_id`。运行时或兼容 Store 只接受“最新事件就是该 achieved 核验”的条件；任何新事件都会使引用失效。not-achieved 形成可观察 HandBack 并允许同一 Gateway 提交新版本片段。

Runtime 的事件即时驱动 Gateway/UI，Projector 延后批量写入任务历史和 Outbox。协议不承载自由文本依据，日志只记录 ID、序号和封闭结论。
