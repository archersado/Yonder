# CX-S1 产品需求

## 问题与目标

授权范围内事件驱动采集，不读 History DB、不全盘扫描；密码/隐私窗口排除。

## 范围与非目标

本 Story 仅负责“原生上下文与浏览扩展验证”。Windows 已验证，macOS 延期；产品采集与检索接线待定。

## 验收条件

- 目标行为有可复现成功样本，失败不得伪报成功。
- 授权范围内事件驱动采集，不读 History DB、不全盘扫描；密码/隐私窗口排除。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。

## 待决事项

Windows 已验证，macOS 延期；产品采集与检索接线待定。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：产品定义、MVP 主干链路、MVP 明确不做。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：个人上下文。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。
