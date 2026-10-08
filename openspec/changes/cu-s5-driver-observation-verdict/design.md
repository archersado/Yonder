# Design：结构事实与慢脑目标核验

`AttemptConclusion::Observed { action_succeeded }` 是 Driver 动作结果；结果为 Observed 即代表本轮 Observe 有效。`ComputerObservation::target_visible` 是 Driver 直接返回的可选目标可见状态。Application 只把这三项映射为 `action-succeeded`、`observe-valid`、`target-visible`，不读取 `CuaActionKind` 推导业务寓意。

旧业务事实枚举继续可解码，以免已有事件或请求无法读取，但运行时不会合成对应真值；以旧业务事实作为条件的片段会因缺少可证明事实而交回慢脑。协议 1.47 的新计划应只提交结构条件。

片段内 Jev 仅取得结构事实和既有脱敏候选；完整 Observation 只经 Gateway 返回归属慢脑。片段耗尽进入 `awaiting-goal-verification`，慢脑检查 transcript、元素或截图引用后提交 achieved/not-achieved。Application 只校验身份、序号、新鲜度和核验门禁。
