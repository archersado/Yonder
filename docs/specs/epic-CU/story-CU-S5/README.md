# CU-S5 CUA 目标状态验证闭环

Story: CU-S5
Epic: CU
Status: verifying
OpenSpec: cu-s5-goal-verification-loop

[OpenSpec Change](../../../../openspec/changes/cu-s5-goal-verification-loop/)

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 目标

把“动作成功”“步骤效果已 Observe”和“用户目标已达成”拆为独立事实。计划片段结束后由归属慢脑基于最新 Observation 显式核验目标，核验通过前不得完成任务。

Windows 原生验证按主人决定暂缓；macOS 完成协议、Application、Gateway、SQLite 事后投影、顶部状态和正式链路验证后进入 verifying。

2026-10-07 macOS 独立 Verification Goal PASS：协议 1.43、ExecutionRuntime、SQLite 事后投影、Gateway/CLI、任务事件与顶部浮窗均已接入；全 Workspace 196 个测试通过。Windows 对等证据继续暂缓，因此保持 verifying、不 Archive。验证记录见 [verification-goal.md](../../../../openspec/changes/cu-s5-goal-verification-loop/verification-goal.md)。
