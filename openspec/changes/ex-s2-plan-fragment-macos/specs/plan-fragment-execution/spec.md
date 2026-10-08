# 计划片段执行增量规格

## ADDED Requirements

### Requirement: 快脑从新鲜 AX 动作空间自然选择

系统 SHALL 由 Application 把同一执行调用中新鲜 Observe 的临时元素句柄展开为已声明语义步骤的有界动作空间。唯一句柄 SHALL 直接派发；多句柄 SHALL 由 Jev 在脱敏元素摘要中选择。系统 MUST NOT 为强制调用 Jev 构造无业务意义的恢复候选。

#### Scenario: 多个 AX 控件进入 Jev

- **WHEN** 前一动作已完成 Observe，当前语义步骤映射到两个以上合法的临时 AX 句柄
- **THEN** Jev 只接收步骤标签、封闭角色、脱敏短标签、条件摘要和不透明候选 ID
- **AND** Application 将所选索引与同次引用仅注入当次 Driver 请求，Driver 在动作前重新 Observe 校验
- **AND** transcript、截图、坐标、敏感文本和完整 Agent Payload 不进入 Jev、SQLite、事件、Outbox 或日志

#### Scenario: 唯一元素不调用 Jev

- **WHEN** 当前语义步骤只有一个合法临时句柄
- **THEN** Application 直接派发并保持动作后 Observe
- **AND** 不调用 Jev，不生成人工候选以追求快脑命中

### Requirement: unknown 交回允许新片段重新观察

系统 SHALL 在 Driver 动作结果为 `unknown` 时保留原始结论、原因与可用 Observe 证据，不推进旧片段游标，也不自动重试副作用。交回事件、Outbox 与独立 handback 边界 SHALL 同事务提交，使已结束的旧 attempt 不再占据步骤执行槽；归属慢脑读取 `task.get` 与 `task.events` 后，MAY 通过原 Gateway 以新的 step/attempt 提交受限片段。

#### Scenario: unknown 后提交重新观察片段

- **WHEN** 当前槽位落库为 `unknown` 并完成 handback，归属慢脑读取最新任务与事件后提交新的重新观察片段
- **THEN** 系统保留旧 attempt 的 unknown 结论且旧片段游标不变
- **AND** 新片段以新的 step/attempt 身份通过既有授权、租约和 Observe 门禁执行，不因旧 attempt 永久占槽而拒绝

#### Scenario: 未完成交回不得跨越 unknown

- **WHEN** unknown 结果尚未形成有效 handback 边界，或调用方尝试复用旧 step/attempt
- **THEN** 系统拒绝新副作用，不把 unknown 提升为成功，也不自动重放原动作

### Requirement: 快脑核验当前步骤完成

系统 SHALL 在 Driver 动作后强制 Observe，并在结构事实成立后允许 Jev 基于当前步骤
和本轮有界脱敏观察摘要判断 `complete`、`incomplete` 或 `uncertain`。只有高置信
`complete` SHALL 推进片段槽位；其他结论 SHALL 停止片段并交回归属慢脑。

#### Scenario: 播放了错误歌曲

- **WHEN** 当前步骤要求播放指定歌曲，但最新 Observe 只证明播放器正在播放另一首歌曲
- **THEN** 快脑返回 incomplete，Application 不为当前步骤投影成功图标且不推进槽位
- **AND** 系统交回归属慢脑重新规划，不自动重试点击

#### Scenario: 当前步骤语义已经成立

- **WHEN** 最新 Observe 的有界摘要与结构事实共同证明当前步骤标签描述的效果成立
- **THEN** 快脑高置信返回 complete，Application 推进当前槽位并继续片段
- **AND** 该结论不产生用户目标 achieved，也不能直接完成任务

#### Scenario: 核验输入不足或服务不可用

- **WHEN** 观察摘要为空、Jev 低置信、返回 uncertain、超时或依赖不可用
- **THEN** Application 安全交回慢脑，不把 Driver 动作回执冒充步骤完成
- **AND** 截图、坐标、Driver 参数、消息正文、接收人、完整 transcript 和模型原文不进入持久化或日志

### Requirement: AX 候选选择携带完整目标与步骤上下文

Jev 从新鲜 AX/UIA 元素动作空间选择候选时，Application SHALL 提供有界任务整体目标、
完整片段步骤标签、当前步骤序号与当前步骤标签；这些字段 SHALL 只驻留本轮内存，且不得
包含任务说明、消息正文、坐标、完整 transcript 或历史模型 Payload。

#### Scenario: 同页存在全局播放与目标行内播放

- **GIVEN** 当前 Observation 同时包含全局播放按钮和多个歌曲行内播放按钮
- **WHEN** 当前步骤要求播放指定歌曲
- **THEN** Jev 请求同时包含任务整体目标、计划步骤上下文和当前步骤位置
- **AND** 不得仅凭“播放”短标签把全局控件当作目标行内控件
