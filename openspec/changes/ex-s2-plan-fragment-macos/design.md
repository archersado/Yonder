# 设计

Rust 协议是唯一模型来源，新增 `task.plan.submit` 与 `task.plan.execute`。提交参数包含可信会话身份、`task_id`、`expected_sequence`、`plan_id`、`plan_version`、`deadline`、动作/令牌预算和最多十个顺序槽位；候选仅映射为已校验的 CUA `tool_name + arguments`，且每个槽位有唯一交回语义。SQLite 保存不可变片段与当前槽位；提交、片段事件及 Outbox 在一个事务内，CAS 失败不写入。

执行时 Gateway 验证协商能力、归属和 deadline，Application 读取片段并复核当前 sequence、控制状态、预算和前一 Observe。它调用既有 Jev 决策 Port，只接受片段中候选；随后复用 `start_execution`、桌面租约、CUA dispatch 和 Observe。动作结果已观察后推进任务步骤并记录槽位；每个执行请求只消费一个槽位，留出用户控制优先级。任何不确定或不适合继续的结果以片段交回事件和 Outbox 表达，任务事实仍由既有状态机维护。

测试覆盖协议派生、身份/CAS/版本/期限/候选校验、事务回滚、Jev 低置信交回、已观察动作与未知结果、取消/控制优先。macOS 原生验证仅在代码完整后执行；Windows 不在本 Change 验证范围。
