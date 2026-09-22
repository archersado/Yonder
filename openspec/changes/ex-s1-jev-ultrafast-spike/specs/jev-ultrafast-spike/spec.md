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

#### Scenario: 固定候选
- WHEN 探针提交固定 Observe 摘要
- THEN Jev 只能在候选 ID 中选择
- AND 无可派发候选时必须选择 handback。

#### Scenario: 禁止越权
- WHEN 候选包含不可派发动作
- THEN 探针不执行该动作
- AND 只记录选择、置信和失败分类。

### Requirement: 基线对照

同一统一样本在慢脑逐步决策与 Jev 快脑之间对照，MUST 使用相同计划、观察、授权和停止条件，并输出可比较指标。

#### Scenario: 相同样本对比

- WHEN 运行统一样本
- THEN 两组使用相同计划、观察、授权和停止条件
- AND 记录任务成功率、误动作率、token、费用、时延、交回率和中断响应

#### Scenario: 相同样本
- WHEN macOS 与 Windows 分别运行同一组样本
- THEN 输出必须使用相同指标结构和判定门槛
- AND 结果可比较。

#### Scenario: 取消检查
- WHEN 决策请求发起后立即取消
- THEN 必须停止派发新动作
- AND 记录停止耗时与错误分类。

### Requirement: 产品门禁
在双平台对照证据未通过前，本 Spike MUST NOT 开放产品 Gateway、任务状态、Jev 执行接线或第二执行栈；AD-EX-03 允许的非敏感配置界面除外。

#### Scenario: 证据未齐

- WHEN Spike 仅完成 macOS 子范围
- THEN AD-EX-02 保持 Proposed
- AND 不开放 Jev 执行接线
- AND 不引入第二执行栈或新通用 Agent
