# Jev Ultrafast Spike Delta

## ADDED Requirements

### Requirement: 有界快脑决策
- **WHEN** Jev 在 Observe 候选集中选择下一操作
- **THEN** 输入必须来自固定计划和结构化候选摘要
- **AND** 不生成自由文本、命令、文档正文或新权限

#### Scenario: 固定候选
- WHEN 探针提交固定 Observe 摘要
- THEN Jev 只能在候选 ID 中选择
- AND 无可派发候选时必须选择 handback。

#### Scenario: 禁止越权
- WHEN 候选包含不可派发动作
- THEN 探针不执行该动作
- AND 只记录选择、置信和失败分类。

### Requirement: 基线对照
- **WHEN** 运行统一样本
- **THEN** 慢脑逐步决策与 Jev 快脑使用相同计划、观察、授权和停止条件
- **AND** 输出任务成功率、误动作率、token、费用、时延、交回率和中断响应指标

#### Scenario: 相同样本
- WHEN macOS 与 Windows 分别运行同一组样本
- THEN 输出必须使用相同指标结构和判定门槛
- AND 结果可比较。

#### Scenario: 取消检查
- WHEN 决策请求发起后立即取消
- THEN 必须停止派发新动作
- AND 记录停止耗时与错误分类。

### Requirement: 产品门禁
- **WHEN** Spike 尚未通过双平台对照
- **THEN** 不开放产品 Gateway、SQLite 或任务状态
- **AND** 不引入第二执行栈或新通用 Agent

#### Scenario: 双平台未完成
- WHEN Windows 或 macOS 对照证据缺失
- THEN AD-EX-02 保持 Proposed
- AND 不启动 EX-S2 实施。
