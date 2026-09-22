# DS-S4 桌宠动画资源包导入

Story: DS-S4  
Epic: DS  
Status: verifying
OpenSpec: ds-s4-mascot-pack-import

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

产品简报要求用户导入符合规范的桌宠动画资源包。用户现进一步要求参考 Hatch Pet：上传自定义形象，由生成流程产出 Yonder 定义的完整状态动画。当前正式桌宠只使用内置素材；本 Story 定义生成契约，以及受限 ZIP 的校验、暂存与原子切换，不改变任务、录制或动画生命周期事实。

## OpenSpec 与验证

[导入 Change](../../../../openspec/changes/ds-s4-mascot-pack-import/proposal.md)已进入实现验证：v1 manifest 限额、九状态 ZIP 校验、暂存与带回滚切换已完成。用户已明确 Hatch Pet 生成仅保留设计，不创建生成实现 Proposal 或代码；该部分等待 Proposed [AD-DS-04](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-DS-04-MASCOT-GENERATION-CONTRACT.md)、AG-S4 与 FI-S1 门禁后再单独排期。2026-09-22 已补齐真实托盘触发、原生打开面板、无效包拒绝反馈和旧包保留的 macOS 证据；Windows 暂缓，完整 Story 不 Archive。

## 子范围验证

- [子范围 Verification Goal](../../../../openspec/changes/ds-s4-mascot-pack-import/verification-goal.md)
