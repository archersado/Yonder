# Jev Ultrafast Spike Delta

## ADDED Requirements

### Requirement: 有界快脑决策
- **WHEN** Jev 在 Observe 候选集中选择下一操作
- **THEN** 输入必须来自固定计划和结构化候选摘要
- **AND** 不生成自由文本、命令、文档正文或新权限

### Requirement: 基线对照
- **WHEN** 运行统一样本
- **THEN** 慢脑逐步决策与 Jev 快脑使用相同计划、观察、授权和停止条件
- **AND** 输出任务成功率、误动作率、token、费用、时延、交回率和中断响应指标

### Requirement: 产品门禁
- **WHEN** Spike 尚未通过双平台对照
- **THEN** 不开放产品 Gateway、SQLite 或任务状态
- **AND** 不引入第二执行栈或新通用 Agent
