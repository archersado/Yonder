# ST-S2 产品需求

2026-09-14后续用户变更（AD-ST-01）：加密整体延期至MVP之后。保留数据库/FTS、附件认证加密、系统Credential Store及用途隔离子密钥；增加MVP明文数据迁移需求。恢复实施前设计备份、校验、原子切换、失败保留原数据，Windows/macOS分别验证。本轮只记录需求，不实施加密或迁移。

## 问题与目标

主密钥只存系统 Credential Store，用途子密钥只驻内存。

## 范围与非目标

本 Story 仅负责“产品凭据与子密钥接线”。用户明确暂停密钥相关工作。

## 验收条件

- 目标行为有可复现成功样本，失败不得伪报成功。
- 主密钥只存系统 Credential Store，用途子密钥只驻内存。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。

## 待决事项

用户明确暂停密钥相关工作。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：MVP 主干链路、待决策事项。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：个人上下文、非功能与发布。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。
- 用户明确暂停密钥工作；保留原安全需求，暂停研发不等于取消加密要求。
