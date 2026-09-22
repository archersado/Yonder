# EX-S3 CUA/BUA 动态候选动作

Story: EX-S3  
Epic: EX  
Status: design-review
OpenSpec: -

依赖 EX-S2 和 CU/BU 已验证执行边界后创建 OpenSpec。

2026-09-23 设计增量：已明确 EX-S3 与 EX-S2 计划片段和 Proposed AD-EX-04 的边界；当前仅保留“每步 Jev 决策”基线，不实施 Recipe、批处理执行或可执行 DSL。
2026-09-23 补充候选契约：Driver 输出的 `observation_id/candidate_id/action/target_ref/text_ref` 只在当前 Observe 内有效；派发前必须复核目标新鲜度，派发后必须重新 Observe。

设计：[产品需求](product-requirements.md) · [架构设计](architecture-design.md) · [视觉交互设计](visual-interaction-design.md)。
