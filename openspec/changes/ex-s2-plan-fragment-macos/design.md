# 设计

Rust 协议是唯一模型来源，新增 `task.plan.submit` 与 `task.plan.execute`。提交参数包含可信会话身份、`task_id`、`expected_sequence`、`plan_id`、`plan_version`、`deadline`、动作/令牌预算和最多十个顺序槽位；候选仅映射为已校验的 CUA `tool_name + arguments`，且每个槽位有唯一交回语义。SQLite 保存不可变片段与当前槽位；提交、片段事件及 Outbox 在一个事务内，CAS 失败不写入。

执行时 Gateway 验证协商能力、归属和 deadline，Application 读取片段并复核当前 sequence、控制状态、预算和前一 Observe。单一已验证候选直接派发；仅多候选动作空间调用既有 Jev 决策 Port，且只接受片段中候选。随后复用 `start_execution`、桌面租约、CUA dispatch 和 Observe。动作结果已观察后推进任务步骤并记录槽位；同一次执行请求继续读取下一槽位，但在下一副作用前重新检查任务 sequence、控制和 Observe 边界。任何不确定或不适合继续的结果立即停止同步推进，并以片段交回事件和 Outbox 表达；响应返回后不留后台执行，任务事实仍由既有状态机维护。

测试覆盖协议派生、身份/CAS/版本/期限/候选校验、事务回滚、Jev 低置信交回、已观察动作与未知结果、取消/控制优先。macOS 原生验证仅在代码完整后执行；Windows 不在本 Change 验证范围。

MCP 增量只增加 `task_plan_submit` 与 `task_plan_execute` 两个薄适配工具：把 MCP JSON 参数反序列化为 Rust `PlanSubmitParams`/`PlanExecuteParams`，再复用现有 `gateway()` 的协议 1.31 握手与 UDS 请求。候选嵌套结构由 Rust 协议反序列化和 Gateway 验证，CLI 不接受自由 DSL、不直接调用 Jev/Driver，也不返回完整模型请求。
