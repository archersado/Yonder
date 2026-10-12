# AG-S12 圈选回复卡呈现与拖拽

Story: AG-S12
Epic: AG
Status: done
OpenSpec: ag-s12-reply-card-interaction
Verification: openspec/changes/archive/2026-10-12-ag-s12-reply-card-interaction/verification-goal.md

关联变更：[ag-s12-reply-card-interaction](../../../../openspec/changes/archive/2026-10-12-ag-s12-reply-card-interaction/proposal.md)。

## 目标

修复圈选回复中常用 Markdown 列表与段落呈现不清晰的问题，并允许用户从卡片标题区拖动无边框圈选窗体。

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 范围

- 首批覆盖 macOS 圈选确认/回复卡。
- 不改变 Agent 回复协议、运行时状态所有者、正文内存边界或持久化。
- Windows 原生证据继续 Deferred。

## 完成证据

- Markdown 松散列表、显式项目标记和标题区原生拖动已合入本 Story 分支。
- [独立 macOS Verification Goal](../../../../openspec/changes/archive/2026-10-12-ag-s12-reply-card-interaction/verification-goal.md)通过；Windows 继续 Deferred。
