# FI-S1 产品需求

## 问题与目标

绝对路径防穿越，别名不能绕过同文件互斥，删除默认回收站。

## 范围与非目标

本 Story 仅负责“规范文件身份与受控操作”。双平台身份语义、原子写与锁冲突需设计和验证。

## 验收条件

- FI-01：目标行为有可复现成功样本，失败不得伪报成功。
- FI-02：输入只接受规范化绝对路径；未创建输出通过既有父目录验证授权根，阻止路径穿越和链接逃逸。
- FI-03：软链接、硬链接、大小写别名和重命名不能绕过同文件写互斥。
- FI-04：写入使用同目录临时文件，校验后原子提交；冲突或失败不留下正式目标。
- FI-05：Office/WPS已打开文件不得被绕过锁覆盖；锁冲突返回可观察等待原因。
- FI-06：删除默认进入系统回收站；永久删除不在MVP默认入口。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。

## 待决事项

双平台身份语义、原子写与锁冲突需设计和验证。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：目标用户与工作场景、MVP 主干链路。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：Command、File 与 Document。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。
