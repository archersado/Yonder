# BU-S1 产品需求

## 问题与目标

BUA 复用外部 Task Space，未提供 Runtime 的平台明确不可用。

## 范围与非目标

本 Story 负责 macOS ego-lite Task Space Bridge 与平台能力。Windows Runtime 延期，不能宣称双平台 BUA 可用。

## 验收条件

- 目标行为有可复现成功样本，失败不得伪报成功。
- BUA 复用外部 Task Space，未提供 Runtime 的平台明确不可用。
- macOS 支持创建/复用、观察、交给用户、用户接管和完成外部 Task Space，并返回稳定外部引用、所有权与托管页数量。
- Bridge 失败明确分类，不以 CUA 回退，不自动重试未知副作用。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。

## 待决事项

外部引用进入 SQLite 当前事实源依赖 TM-S1 字段定案；本增量先交付 Bridge 和同一执行尝试的调用契约。Windows Runtime 延期，不能宣称双平台 BUA 可用。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：两条执行路径；补充材料 CUA 与 BUA 的 Task Space。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：CUA 与 BUA。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。
