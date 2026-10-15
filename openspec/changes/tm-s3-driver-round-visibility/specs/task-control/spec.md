# task-control Delta Specification

## ADDED Requirements

### Requirement: CUA 控制条必须展示真实执行过程

系统 MUST 在真实 CUA 执行区间展示规划、快脑、当前步骤及最近六轮 Driver 脱敏日志。展示不可用不得阻塞执行，Driver 动作成功不得被解释为用户目标完成。

#### Scenario: Driver 轮次逐轮更新

- **WHEN** 宿主向正式 CUA Driver 派发动作并取得动作后 Observe
- **THEN** 控制条先显示该轮正在执行，再以同一轮更新已观察成功、明确未成功或待核实结论
- **AND** 不显示动作参数、正文、截图、transcript、完整 Payload 或内部身份

#### Scenario: replan 保留轮次

- **WHEN** 单轮未成功并经同一 Gateway 提交新计划
- **THEN** 新 Driver 动作以新轮次追加，旧轮次仍在当前任务的有界列表中可见
- **AND** 只有慢脑目标核验可以驱动任务完成
