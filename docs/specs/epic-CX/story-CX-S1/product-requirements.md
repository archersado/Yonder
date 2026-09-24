# CX-S1 产品需求

## 问题与目标

授权范围内事件驱动采集，不读 History DB、不全盘扫描；密码/隐私窗口排除。

## 范围与非目标

本 Story 仅负责“原生上下文与浏览扩展验证”。Windows 已验证；2026-09-24后续用户推进指令解除macOS延期，仅授权继续完成macOS技术验证。产品采集、存储、同步与检索接线仍待后续独立Story或Change。

## 验收条件

- 目标行为有可复现成功样本，失败不得伪报成功。
- 授权范围内事件驱动采集，不读 History DB、不全盘扫描；密码/隐私窗口排除。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。
- macOS当前窗口样本由系统事件触发，只记录应用标识、进程标识、窗口元数据是否可用与事件计数；不得保存窗口标题、控件正文、键值或坐标。
- macOS浏览样本使用Chrome/Edge扩展经用户级Native Messaging Host建立真实连接；扩展不申请`history`权限，隐私窗口事件发送前拒绝，Host标准输出只承载协议帧。
- Accessibility或浏览器连接权限不足时明确报告能力不可用；不得通过轮询、截图、History数据库、通配来源或自动扩大权限绕过。

## 待决事项

Microsoft Edge在当前macOS验证环境未安装，因此本轮允许先形成Google Chrome子范围证据，但不能把它标记为macOS双浏览器完成。产品Context Port字段、配额、保留与用户开关不在本Spike定案。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：产品定义、MVP 主干链路、MVP 明确不做。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：个人上下文。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。
- 后续用户变更：2026-09-24“继续推进”，在已明确不推进Windows验证的上下文中，恢复CX-S1的macOS验证；不扩大到产品接线。

## 验收映射

| 验收 | 来源 | 证据 |
|---|---|---|
| CX1-01 事件驱动取得当前应用/窗口元数据且不保存正文 | 产品简报“显式上下文录制”；架构主干“当前窗口” | macOS原生结构化日志 |
| CX1-02 Chrome Native Messaging真实连接，协议有界且stdout隔离 | 架构主干“浏览”；Accepted AD-E0-05跨平台围栏 | Chrome实连与Host计数证据 |
| CX1-03 History/隐私窗口/密码与安全界面fail-closed | AGENTS与架构主干隐私约束 | Manifest静态检查、隐私拒绝样本、无正文证据 |
| CX1-04 权限不足、浏览器缺失和未验证项不伪报成功 | 原始验收“平台限制与未实现项明确” | 稳定结果码与Verification Goal |
