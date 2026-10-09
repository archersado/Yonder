# 设计

Rust 协议是唯一模型来源，新增 `task.plan.submit` 与 `task.plan.execute`。提交参数包含可信会话身份、`task_id`、`expected_sequence`、`plan_id`、`plan_version`、`deadline`、动作/令牌预算和最多十个顺序槽位；候选仅映射为已校验的 CUA `tool_name + arguments`，且每个槽位有唯一交回语义。SQLite 保存不可变片段与当前槽位；提交、片段事件及 Outbox 在一个事务内，CAS 失败不写入。

执行时 Gateway 验证协商能力、归属和 deadline，Application 读取片段并复核当前 sequence、控制状态、预算和前一 Observe。单一已验证候选直接派发；仅多候选动作空间调用既有 Jev 决策 Port，且只接受片段中经本轮 Observe 校验合格的动作候选。`handback` 不作为模型候选；低置信、未知返回或调用异常由 Application 统一形成唯一安全交回。随后复用 `start_execution`、桌面租约、CUA dispatch 和 Observe。动作结果已观察后推进任务步骤并记录槽位；同一次执行请求继续读取下一槽位，但在下一副作用前重新检查任务 sequence、控制和 Observe 边界。任何不确定或不适合继续的结果立即停止同步推进，并以片段交回事件和 Outbox 表达；SQLite schema 23 为原始 `phase=unknown` 增加独立 handback 边界序号，读取执行槽时将已交回 attempt 投影为停止，不改写未知结论、不推进旧片段。随后只允许归属慢脑读取事实后以新 step/attempt 提交新片段。响应返回后不留后台执行，任务事实仍由既有状态机维护。

测试覆盖协议派生、身份/CAS/版本/期限/候选校验、事务回滚、Jev 低置信交回、已观察动作与未知结果、取消/控制优先。macOS 原生验证仅在代码完整后执行；Windows 不在本 Change 验证范围。

动作后的步骤核验复用 Application 拥有的 Jev Port，但使用独立、封闭的三值请求。
请求只携带步骤标签、已派发动作语义、预期条件、结构事实和本轮 Observe 派生的有界
脱敏文本行；Adapter 不持有任务状态，Worker 不重试。高置信 complete 才推进；其余
结果统一映射到交回。观察摘要不持久化、不记录、不进入 Gateway 响应，最终目标完成
仍走 AD-TM-24 的 `task.goal.verify`。

MCP 增量只增加 `task_plan_submit` 与 `task_plan_execute` 两个薄适配工具：把 MCP JSON 参数反序列化为 Rust `PlanSubmitParams`/`PlanExecuteParams`，再复用现有 `gateway()` 的协议 1.31 握手与 UDS 请求。候选嵌套结构由 Rust 协议反序列化和 Gateway 验证，CLI 不接受自由 DSL、不直接调用 Jev/Driver，也不返回完整模型请求。

交回响应提交后由Desktop组合根在TaskHost锁外调用AG-S5既有`AgentInputHub`，以Rust协议新增的`source=replan`向唯一归属Agent会话投递有界唤醒文本。投递不参与动作结果事务、不阻塞Gateway执行，也不代替事件/Outbox；失败只形成短生命周期展示。顶部浮窗Hub增加`replanning`投影：交回时保留失败步骤，投递被Agent接受后显示慢脑正在规划，新`task.plan.submit`成功后从权威片段刷新计划摘要。慢脑仍从同一Gateway读取事实、提交和执行新片段。

2026-10-08 自然快脑选择增量：Application 在同一执行调用内从最近一次新鲜 AX Observe 展开有界临时元素候选。不变更外部计划的慢脑语义模板，不持久化动态候选。Jev 仅在自然形成多个合法元素时获得步骤标签、封闭角色、脱敏短标签与不透明 ID；唯一元素直接执行。所选句柄与同次 `observation_ref` 只注入本次 Driver 请求，Worker 动作前复核；失效或低置信交回慢脑。2026-10-09 multilingual 真实样本进一步要求候选包含`TextField`与`ComboBox`，并在截断前以任务目标/当前步骤对标签作 CJK/拉丁文本均有效的稳定相关性排序，避免 AX 树前序永久排除目标控件。
