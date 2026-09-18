# CU-S1 产品需求

## 问题与目标

同一 Windows/macOS 样本验证桌面观察输入与故障语义，形成唯一选型。

## 范围与非目标

后续用户变更：2026-09-14明确仅使用trycua SDK，不再包上游App；Accepted AD-CU-01保留按需Worker与权限/隐私/停止约束。STOP-PKG独立构件分支撤回，不删失败证据、不安装已下载包。新增STOP-SDK01–04分别验收自管SDK子进程就绪、监督终止并确认退出、全新实例只读恢复、私有宿主通道断开后退出。来源为原Driver执行路径/故障验证、架构Supervisor及本次SDK-only用户变更；只读通过不等于执行中键鼠停止通过。

本 Story 仅负责“桌面 Driver 技术选型”。Windows 选型已有证据；macOS 和 Office 延期，不改称全部完成。

## 验收条件

STOP-DRAIN01–03来源原执行原则/故障验证、架构CUA每步Observe及SDK-only接管变更：隔离AX setter提供原生写入开始/应用证据；在未完成时请求SDK shutdown，核实关闭确认晚于原生应用；由新SDK Observe与目标最终状态匹配。调用已完成或未到setter的样本失败，不称执行中排空。只验证后台AX graceful drain，其他原生输入/强制中断及正式宿主仍待验证。

SDK原生输入补充STOP-IN01–03来自原产品执行路径/执行原则、架构CUA权限/每步Observe和本次SDK-only变更：先在导入SDK的宿主只读检查权限，Accessibility缺失不派发输入；隔离目标唯一确认后AX写固定标记并Observe验证；动作间停止阻止新增输入并核实稳定状态。后两项不由权限探针或只读退出测试追认，执行中动作中断仍独立待验证。不请求屏幕录制用于无截图AX样本，不以像素或剪贴板绕过失败。

- 目标行为有可复现成功样本，失败不得伪报成功。
- 同一 Windows/macOS 样本验证桌面观察输入与故障语义，形成唯一选型。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。

## 待决事项

Windows 选型已有证据；macOS 和 Office 延期，不改称全部完成。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：两条执行路径；补充材料 执行原则、验证方案。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：CUA 与 BUA。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。

## macOS接管停止补充Spike

来源：原产品两条执行路径/用户接管变更、架构CUA与任务恢复、AD-E0-02/AD-TM-03；关联spikes/cua-driver-comparison/TAKEOVER-STOP-PLAN.md。期限2026-09-14至2026-09-16，统一样本沿用已有fault/crash-recovery，唯一路线trycua0.25.0，Qwen已淘汰不再运行。STOP-READ01记录预取消只读调用真实结果；STOP-READ02记录关闭/关闭后调用/重复关闭；STOP-READ03异常退出后新实例只读恢复；STOP-READ04只保留布尔/错误分类/时间，不输出正文或应用列表。均对应原Driver故障验证与隐私/恢复约束。

只读生命周期探针不触发键鼠、截图或Recording。取消Promise/Node退出不是系统输入停止证据，未知结果保留占用；没有真实动作/worker树停止证据前CU-S2/TM-S3接管按钮继续禁用。Windows新增测试暂缓，完整双平台CU验收不通过/不Archive。

### 已提交只读调用补充验收

供应链补充STOP-PKG01–03对应原Driver技术选型/安全分发及架构签名边界：官方固定0.25.0 macOS构件校验并安全解包、原生签名/Gatekeeper评估、证据和拒绝结果可追溯。未通过时不运行，不使用重签或移除隔离绕过。只下载到Spike证据目录，不安装系统服务；Windows暂停、真实输入未授权由此直接实施。

来源仍为原产品执行原则、架构故障语义与用户接管变更；同一Spike期限内复用唯一trycua。STOP-SUB01提交list_apps后下一事件循环请求Abort，记录请求是否先于Promise完成，不假定原生动作已经准入；STOP-SUB02取消后的新只读调用可用；STOP-SUB03在只读调用未等待返回时请求shutdown，记录两者结果且关闭后新调用拒绝；STOP-SUB04不采集正文、键鼠或Recording，原生停止证据仍为未验证。Windows同样本暂缓，不把只读竞态标作执行中输入停止通过。
