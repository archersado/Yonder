# Proposal：CU-S5 Driver Observation 慢脑判定

## Why

计划片段当前通过 `ObservedFacts::after(action_kind)` 把任意动作硬编码为 `application-ready=true`、`target-resolved=true`，并对消息动作继续合成 composer/delivery 事实。这会把动作类型误当真实界面状态，违背目标核验由慢脑基于最新 Observation 决定的架构。

## What Changes

- 协议增加 Driver 可直接证明的结构条件：动作明确成功、Observe 有效、目标可见。
- Application 从真实 `DispatchOutcome` 与 `ComputerObservation` 构造结构事实，不再按动作类型合成业务事实。
- 计划片段仅用结构事实决定能否继续已授权候选；片段结束仍把最新 Observation 返回归属慢脑。
- 用户目标完成只接受慢脑提交的 `task.goal.verify`，不增加媒体播放等硬编码业务规则。

## Impact

更新 AD-TM-24、CU-S5 Story、Rust 协议及生成物、EX-S2 计划片段选择与 Gateway 能力版本。无 SQLite 迁移，不改变 Driver 接口，不保存 transcript、截图或模型推理。
