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
