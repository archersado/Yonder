# EN-S1 产品需求

## 问题与目标

按技术模块管理 Epic，每个 Story 三份设计明确后才转 OpenSpec 实施。

## 范围与非目标

本 Story 仅负责“模块 Epic 与 Story 设计门禁”。本 Story 为用户明确授权的流程调整；运行门禁自测后进入验证，不改运行时。

## 验收条件

- 目标行为有可复现成功样本，失败不得伪报成功。
- 按技术模块管理 Epic，每个 Story 三份设计明确后才转 OpenSpec 实施。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。
- 13 个技术模块有独立 Epic 目录，当前 34 个 Story 各有四份必需文件；原七份 Story 保留迁移映射和历史正文。
- 删除任何设计文档、清空必需章节、错配模块、以历史平铺文件替代 Story、以 draft/design-review/deferred 提交实施 PR 时门禁失败。
- 当前仓库规划和 EN-S1 的真实路径 PR 关联检查通过；不因此宣布其他 Story 完成。

## 待决事项

无未决产品问题；结构门禁不判断设计内容质量，语义审阅仍由实施者和审阅者承担。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：MVP 成功标准、待决策事项。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：非功能与发布；研发与需求变更模式。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。
- 本 Story 的直接来源是用户要求按技术模块 Epic/Story 目录与三份设计先行的研发围栏，记录于 AD-DEV-01；不是凭产品简报推导的新产品功能。
