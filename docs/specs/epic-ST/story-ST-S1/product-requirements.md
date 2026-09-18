# ST-S1 产品需求

## 问题与目标

SQLCipher/FTS5/附件认证加密、错误密钥拒绝、状态事件 Outbox 原子性。

## 范围与非目标

本 Story 仅负责“加密存储技术验证”。保留现有证据，不新增密钥工作；v1 迁移须单独设计和备份。

## 验收条件

- 目标行为有可复现成功样本，失败不得伪报成功。
- SQLCipher/FTS5/附件认证加密、错误密钥拒绝、状态事件 Outbox 原子性。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。

## 待决事项

保留现有证据，不新增密钥工作；v1 迁移须单独设计和备份。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：MVP 主干链路、待决策事项。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：个人上下文、非功能与发布。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。
