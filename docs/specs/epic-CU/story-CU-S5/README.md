# CU-S5 CUA 目标状态验证闭环

Story: CU-S5  
Epic: CU  
Status: implementing  
OpenSpec: cu-s5-goal-verification-loop

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 目标

把“动作成功”“步骤效果已 Observe”和“用户目标已达成”拆为独立事实。计划片段结束后由归属慢脑基于最新 Observation 显式核验目标，核验通过前不得完成任务。

Windows 原生验证按主人决定暂缓；macOS 完成协议、Application、Gateway、SQLite 事后投影、顶部状态和正式链路验证后进入 verifying。
