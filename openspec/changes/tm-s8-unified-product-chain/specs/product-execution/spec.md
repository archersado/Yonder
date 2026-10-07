# product-execution Delta Specification

## ADDED Requirements

### Requirement: 四类能力必须通过同一产品任务闭环验收

系统 MUST 允许真实外部慢脑从产品 MCP 创建并完成 CUA、BUA、Office 与 Command 任务；四类任务 MUST 共用 Agent Gateway、任务事实源、统一启动、Task Space、事件/Outbox 与终态协议。

系统 MUST 由外部 Codex 慢脑拥有首次计划和片段外 replan，由 Yonder/Jev 快脑只在已验证片段内选择并连续执行；多候选验证 MUST 证明 Jev 实际调用、低置信或偏离经同一 Gateway 交回慢脑。

#### Scenario: 正式产品闭环

- **WHEN** Codex 通过安装包内 `yonder mcp` 创建并执行任一能力任务
- **THEN** 当前 Yonder GUI 展示真实任务、计划/当前步骤、确认和状态变化，能力 Adapter 执行并 Observe，归属 Agent 最后提交完成或失败

#### Scenario: 首动作前交回仍保持执行中

- **WHEN** 归属慢脑调用 `task.plan.execute`，Jev 在首个 Driver 动作前选择交回
- **THEN** 同一任务原子从 `created` 进入 `running`，事件、Outbox 与下一意图同事务提交，控制条和 Task Space 显示一致

#### Scenario: 标称新版本数据库缺少完整审计表组

- **WHEN** 正式库标称当前 schema，且三张审计业务表全部缺失
- **THEN** 启动只创建空表并保留全部既有任务事实；若仅部分表缺失则拒绝启动且不修改数据库

#### Scenario: 禁止探针替代

- **WHEN** 只有测试宿主、直接 Driver、私有脚本或数据库写入能够完成样本
- **THEN** 对应产品链路判定 FAIL，并返回所属 Story 修复，不得将底层能力证据标为产品 PASS

### Requirement: 快慢脑效果必须使用同任务配对证据

系统 MUST 使用正式产品链路，对相同初态、目标和完成判据的可复位 CUA 任务比较“慢脑计划片段 + Jev 快脑”与“纯慢脑逐步决策”。报告 MUST 同时包含目标完成率与主动耗时，且 MUST 保留超时、`unknown`、错误目标及人工修正为失败样本。

#### Scenario: 配对样本完成效果汇总

- **WHEN** 每个任务类别的两种模式都完成规定数量的有效配对
- **THEN** 汇总完成数/总数、Wilson 95% 区间、主动耗时中位数/P95、慢脑调用数、handback数和`unknown`数
- **AND** 仅对两边均完成的配对另报耗时比，不以幸存样本耗时替代总体完成率

#### Scenario: 用户等待不污染模型性能

- **WHEN** 样本包含本机确认或接管等待
- **THEN** 同时保留墙钟耗时和显式用户等待时长，并以两者差值报告主动耗时
- **AND** 不得删除该样本或把等待时间误归因于 Jev/慢脑

#### Scenario: 不同任务或非产品调用不可比较

- **WHEN** 两种模式使用不同目标/初态，或任一模式绕过 Gateway 直接调用 Driver、测试宿主或脚本 Planner
- **THEN** 该样本无效且不得进入速度或完成率结论

### Requirement: 所有执行能力必须透出统一顶部步骤浮窗

系统 MUST 在 CUA、BUA、Office/Document、Command 及后续统一 Gateway 执行请求开始时显示顶部执行浮窗。浮窗 MUST 展示可信当前步骤与可用规划步骤；不存在计划片段时 MUST 明确标记未提供，不得伪造。只有 CUA 可以显示并接受“接管电脑”。

#### Scenario: BUA 执行显示步骤但不提供桌面接管

- **WHEN** 归属 Agent 调用 `browser.execute` 开始或继续 Browser Task Space
- **THEN** 顶部浮窗显示“Yonder 正在执行任务”和当前 BUA 步骤
- **AND** 不显示“接管电脑”，任务终态后浮窗关闭

#### Scenario: CUA 保留显式接管

- **WHEN** 归属 Agent 调用 CUA 执行请求
- **THEN** 顶部浮窗显示“Yonder 正在控制您的电脑”、规划与当前步骤以及“接管电脑”
- **AND** 普通输入不关闭浮窗，只有终态或显式接管成功关闭

#### Scenario: 非 CUA 伪造接管被拒绝

- **WHEN** 非 CUA 浮窗或其他本机窗口尝试提交桌面接管
- **THEN** Rust 宿主拒绝请求，不改变任务状态、sequence 或执行准入

### Requirement: 交回必须唤醒归属慢脑并透出重新规划

系统 MUST 在计划片段异常交回提交后，通过既有双向Agent输入会话主动通知唯一归属慢脑；慢脑 MUST 从同一Gateway读取新鲜事实并提交replan。系统 MUST 在顶部浮窗汇总交回、重新规划和新计划，不展示模型思维链或敏感动作参数。

#### Scenario: 已连接慢脑收到交回并提交新片段

- **WHEN** 当前槽位失败或待核实，handback事件与Outbox已提交且归属慢脑存在唯一双向会话
- **THEN** Gateway先返回handback，执行锁释放后向该会话投递`replan`输入
- **AND** 顶部浮窗依次显示交回最小原因、慢脑正在重新规划及新计划步数
- **AND** 新片段仍经原Gateway身份、CAS、期限与能力校验

#### Scenario: 慢脑不可唤醒

- **WHEN** 归属慢脑未连接、存在多个会话、拒绝或投递超时
- **THEN** 任务保持等待重新规划且浮窗明确显示等待连接或唤醒失败
- **AND** Yonder不生成本地语义计划、不自动重试失败副作用
