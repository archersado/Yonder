# EX-S2 快慢脑交接与最小 Jev 配置界面

Story: EX-S2  
Epic: EX  
Status: implementing
OpenSpec: ex-s2-plan-fragment-macos

实施 Change：[openspec/changes/ex-s2-plan-fragment-macos/](../../../../openspec/changes/ex-s2-plan-fragment-macos/proposal.md)

目标动作与发送确认增量：[ex-s2-cua-target-intent-confirmation](../../../../openspec/changes/ex-s2-cua-target-intent-confirmation/proposal.md)。

本轮实施 Accepted AD-AG-09 的受限 macOS CUA 计划片段基线；macOS-only 有界 Jev 决策接线已经完成并归档。Windows 原生验证及跨 Driver 的 Recipe/DSL 仍不在本轮范围。

设计：[产品需求](product-requirements.md) · [架构设计](architecture-design.md) · [视觉交互设计](visual-interaction-design.md)。

## 当前状态与前置条件

三份设计已完成。[最小 Jev 配置界面](../../../../openspec/changes/archive/2026-09-24-ex-s2-jev-config-interface/proposal.md) 已完成并归档；2026-09-24 的 macOS-only 子路线已完成决策 Port、系统凭据读取和远端调用，不包含计划片段执行或 Windows 注册。

2026-09-22：独立最小 Jev 配置窗口已实施并通过 macOS 原生 Verification Goal，见[已归档配置界面 Change](../../../../openspec/changes/archive/2026-09-24-ex-s2-jev-config-interface/proposal.md)与[验证记录](../../../../openspec/changes/archive/2026-09-24-ex-s2-jev-config-interface/verification-goal.md)。Task Space 不再承载配置交互；配置子范围完成，Story 整体保持 implementing。

2026-09-23 设计增量：已明确 EX-S2 与 Proposed AD-EX-04 的边界；当前仅保留“每步 Jev 决策”基线，不实施 Recipe、批处理执行或可执行 DSL。

2026-09-23：计划片段入口已与 AG-S1 联审候选对齐，见 [架构设计](architecture-design.md)。该对齐只明确 Gateway 校验顺序与幂等基线，不改变 `AD-EX-02` 技术路线门禁，也不授权执行接线实施。

2026-09-23：已补充前端回归，锁住“Jev 配置不进入 Task Space”这一边界；Task Space 不加载配置页面、不包含设置表单/能力开关/关闭控件，也不会调用任何 `jev_*` 命令。

同日补齐 `JevConfig::validate` 的端点查询串、URL 内嵌凭据与远端 HTTPS 校验回归，防止配置携带 token、用户名/密码或远端明文端点。
 
2026-09-27：用户要求推进计划片段执行后，[AD-AG-09](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-09-PLAN-FRAGMENT-GATEWAY-CONTRACT.md) 已接受受限 macOS CUA 基线：Gateway 提交不可变片段，Application 在每次 Observe 后仅从已提交候选中请求 Jev 决策；每次 Gateway 执行只消费一个槽位，低置信、偏离、取消、接管、用户输入或 `unknown` 一律交回归属 Agent。该增量不实现 Recipe、自由 DSL、后台批量循环或 Windows 支持。

2026-09-28：用户进一步明确快脑应在慢脑计划上连续判断和执行，而不是要求慢脑逐槽位续调。AD-AG-09 已修订为单次 Gateway 调用内的同步有界连续推进；每动作仍强制 Observe，槽位间重新检查控制与任务事实，异常立即交回，响应后不留后台执行。

2026-09-24 用户变更：先实施 macOS-only Jev 决策接线，Windows 后补；API Key 只从 macOS 系统凭据入口读取，不进入配置、任务数据或日志。该子范围的真实 Keychain/远端调用、离线回归和独立 Verification Goal 已通过，见[已归档 macOS-only 接线 OpenSpec](../../../../openspec/changes/archive/2026-09-24-ex-s2-macos-only-jev-wiring/proposal.md)。Story 整体仍为 implementing；计划片段执行、Windows 与费用证据须先满足各自门禁并另建 Change。

2026-10-08 用户修订：Jev 快脑改为从同次新鲜 AX Observe 自然形成的多控件动作空间中选择，不再为触发快脑构造应用恢复分支。正式安装包 MCP/Gateway macOS 样本已完成“语义模板→AX 多候选→Jev 选择→临时句柄复核→点击→动作后 Observe→Goal Verification→completed”，证据见[计划片段 Verification Goal](../../../../openspec/changes/ex-s2-plan-fragment-macos/verification-goal.md)。Windows 证据仍按决定暂缓，Story 保持 implementing。

2026-10-08 收尾复核确认事件驱动交回已由TM-S9运行时完整承接并通过当前回归；正式QQ音乐AD-EX-05样本仍被CU-S4动作确认阻断。产品任务成功完成启动和搜索入口聚焦，但文本输入、临时元素句柄点击与封闭`SPACE`激活均在进入Jev步骤核验前返回`unknown(driver-action-unconfirmed)`；系统逐次安全交回且未自动重试，测试任务最终取消。修复CU-S4并重建产品前，EX-S2不得Archive。

2026-10-09 用户要求本地 Jev 模型名成为面板配置。增量 Change [ex-s2-configurable-jev-model](../../../../openspec/changes/ex-s2-configurable-jev-model/proposal.md) 修复设置面板已允许 `local`、运行时却只接受远端的契约分裂，并使候选选择与步骤核验统一使用配置快照中的受限模型标识；不改变计划片段、任务状态或敏感数据边界。

同日用户进一步要求置信度字段与阈值可配置。AD-EX-03 与本 Change 已扩展为可填写的响应顶层字段名和 `0..1` 阈值；字段名未填写时使用 `confidence`，旧 `confidence_metric` JSON 兼容迁移，已配置字段缺失时安全交回且不得探测其他属性。结构回归已通过，待重建正式 macOS GUI 后补面板保存与真实 Gateway 样本。

同日正式 Gateway 终局复验 PASS：任务 `task_7ab0958017947f67f79283cfd8ae09c3` 在更新弹窗交回后按新鲜事实递增计划版本，本地 Jev 从 QQ 音乐 252 个元素的主窗口 Observation 中完成自然候选决策，搜索框动作在序号 19 得到 `action_succeeded=true / observe_valid=true`，经目标核验后 `completed@23`。该结果关闭可配置置信度字段的 macOS 产品验证；Windows 继续暂缓，EX-S2 整体仍按计划片段 Change 的剩余项保持 implementing。
