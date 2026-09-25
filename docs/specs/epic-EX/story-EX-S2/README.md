# EX-S2 快慢脑交接与最小 Jev 配置界面

Story: EX-S2  
Epic: EX  
Status: implementing
OpenSpec: ex-s2-macos-only-jev-wiring

完整计划片段执行仍受 EX-S1、TM-S7 与双平台门禁约束；macOS-only 有界 Jev 决策接线已经完成并归档，不派发任务动作。

设计：[产品需求](product-requirements.md) · [架构设计](architecture-design.md) · [视觉交互设计](visual-interaction-design.md)。

## 当前状态与前置条件

三份设计已完成。[最小 Jev 配置界面](../../../../openspec/changes/archive/2026-09-24-ex-s2-jev-config-interface/proposal.md) 已完成并归档；2026-09-24 的 macOS-only 子路线已完成决策 Port、系统凭据读取和远端调用，不包含计划片段执行或 Windows 注册。

2026-09-22：独立最小 Jev 配置窗口已实施并通过 macOS 原生 Verification Goal，见[已归档配置界面 Change](../../../../openspec/changes/archive/2026-09-24-ex-s2-jev-config-interface/proposal.md)与[验证记录](../../../../openspec/changes/archive/2026-09-24-ex-s2-jev-config-interface/verification-goal.md)。Task Space 不再承载配置交互；配置子范围完成，Story 整体保持 implementing。

2026-09-23 设计增量：已明确 EX-S2 与 Proposed AD-EX-04 的边界；当前仅保留“每步 Jev 决策”基线，不实施 Recipe、批处理执行或可执行 DSL。

2026-09-23：计划片段入口已与 AG-S1 联审候选对齐，见 [架构设计](architecture-design.md)。该对齐只明确 Gateway 校验顺序与幂等基线，不改变 `AD-EX-02` 技术路线门禁，也不授权执行接线实施。

2026-09-23：已补充前端回归，锁住“Jev 配置不进入 Task Space”这一边界；Task Space 不加载配置页面、不包含设置表单/能力开关/关闭控件，也不会调用任何 `jev_*` 命令。

同日补齐 `JevConfig::validate` 的端点查询串、URL 内嵌凭据与远端 HTTPS 校验回归，防止配置携带 token、用户名/密码或远端明文端点。
 
2026-09-23：另建立跨 Story 的 [Proposed AD-AG-09 计划片段 Gateway 契约](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-09-PLAN-FRAGMENT-GATEWAY-CONTRACT.md)，用于评估“计划片段 + Driver 本地执行 + Observe 异常升级”能否降低慢脑逐步交互成本。该 ADR 未授权实施，也不替代当前逐步决策架构；执行接线仍等 `AD-EX-02` 双平台通过。

2026-09-24 用户变更：先实施 macOS-only Jev 决策接线，Windows 后补；API Key 只从 macOS 系统凭据入口读取，不进入配置、任务数据或日志。该子范围的真实 Keychain/远端调用、离线回归和独立 Verification Goal 已通过，见[已归档 macOS-only 接线 OpenSpec](../../../../openspec/changes/archive/2026-09-24-ex-s2-macos-only-jev-wiring/proposal.md)。Story 整体仍为 implementing；计划片段执行、Windows 与费用证据须先满足各自门禁并另建 Change。
