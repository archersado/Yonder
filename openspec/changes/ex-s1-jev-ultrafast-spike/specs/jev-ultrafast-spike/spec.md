# Jev Ultrafast Spike Delta

## ADDED Requirements

### Requirement: 有界快脑决策

Jev 只能在固定 Observe 候选集中选择下一步操作；输入 MUST 来自固定计划和结构化候选摘要，输出只能是受支持操作与目标，不生成自由文本、命令、文档正文或新权限。

#### Scenario: 仅在固定候选中选择

- WHEN Jev 接收 Observe 候选集
- THEN 系统只允许返回已定义候选 ID
- AND 不生成自由文本、命令、文档正文或新权限

#### Scenario: 候选缺参

- WHEN 当前候选缺少执行所需参数
- THEN Jev 交回慢脑，不猜测参数

### Requirement: 基线对照

同一统一样本在慢脑逐步决策与 Jev 快脑之间对照，MUST 使用相同计划、观察、授权和停止条件，并输出可比较指标。

#### Scenario: 相同样本对比

- WHEN 运行统一样本
- THEN 两组使用相同计划、观察、授权和停止条件
- AND 记录任务成功率、误动作率、token、费用、时延、交回率和中断响应

### Requirement: 产品门禁

在双平台对照证据未通过前，本 Spike MUST NOT 开放产品 Gateway、SQLite、任务状态或第二执行栈。

#### Scenario: 证据未齐

- WHEN Spike 仅完成 macOS 子范围
- THEN 不开放产品 Gateway、SQLite 或任务状态
- AND 不引入第二执行栈或新通用 Agent
