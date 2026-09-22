# EX-S2 快慢脑交接与最小 Jev 配置界面

Story: EX-S2  
Epic: EX  
Status: implementing
OpenSpec: ex-s2-jev-config-interface

EX-S1/AD-EX-02/TM-S7 门禁通过后开放执行接线；本子范围只实现最小配置界面。

设计：[产品需求](product-requirements.md) · [架构设计](architecture-design.md) · [视觉交互设计](visual-interaction-design.md)。

## 当前状态与前置条件

三份设计已完成。本子范围先实现 [最小 Jev 配置界面](../../../../openspec/changes/ex-s2-jev-config-interface/proposal.md)；执行接线仍等 `AD-EX-02` 双平台通过。

2026-09-22：独立最小 Jev 配置窗口已实施并通过 macOS 原生 Verification Goal，见 [验证记录](../../../../openspec/changes/ex-s2-jev-config-interface/verification-goal.md)。Task Space 不再承载配置交互；执行接线仍未解锁，Story 整体保持 implementing。

2026-09-23：计划片段入口已与 AG-S1 联审候选对齐，见 [架构设计](architecture-design.md)。该对齐只明确 Gateway 校验顺序与幂等基线，不改变 `AD-EX-02` 技术路线门禁，也不授权执行接线实施。
