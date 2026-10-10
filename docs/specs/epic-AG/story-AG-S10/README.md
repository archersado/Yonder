# AG-S10 圈选提交失败恢复与错误反馈

Story: AG-S10
Epic: AG
Status: done
OpenSpec: ag-s10-circle-submit-recovery
Verification: openspec/changes/archive/2026-10-10-ag-s10-circle-submit-recovery/verification-goal.md

关联变更：[ag-s10-circle-submit-recovery](../../../../openspec/changes/ag-s10-circle-submit-recovery/proposal.md)。

## 目标

用户点击发送后，Yonder 只有在 Agent 明确接受输入时才清除圈选临时数据；任何未接受结果都保留当前图片和问题并给出准确、可操作的反馈。

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 范围

- macOS 圈选确认卡、Application 临时会话和现有 AgentInputHub。
- 不新增持久化、协议字段、旁路或截图压缩。
- Windows 证据继续 Deferred。

## 完成证据

- PreviewSession、附件上限、Desktop 投影和圈选卡失败恢复已合入 `dev`。
- [独立 macOS Verification Goal](../../../../openspec/changes/archive/2026-10-10-ag-s10-circle-submit-recovery/verification-goal.md)通过；正式 GUI 证明无 Agent 时图片和问题不再消失。
