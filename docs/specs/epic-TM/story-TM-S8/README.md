# TM-S8 统一产品执行闭环

Story: TM-S8  
Epic: TM  
Status: verifying  
OpenSpec: tm-s8-unified-product-chain

## 目标

用真实 Codex MCP 慢脑连接正在运行的 Yonder，由 Codex 提交首次计划/语义 replan，由 Yonder 内置 Jev 快脑在已验证计划片段内选择并连续推进，分别完成 CUA、BUA、Office 与 Command 的产品级任务闭环。不得用测试宿主、直接 Adapter、私有脚本逐步调用或手工改库替代产品入口。

## 文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前结论

四类底层能力及统一启动已有 macOS 子范围证据，但尚无一份同时证明真实慢脑 MCP 接入、Yonder GUI 可见、完整编排、确认、执行、Observe、审计与终态的统一产品证据。本 Story 专门关闭该差距；Windows 按用户决定暂缓，不据此标记双平台 Done。

2026-09-28 已完成统一执行浮窗的 macOS 增量验收：正式 Gateway 创建的 BUA 与 CUA 任务在运行期间均显示顶部浮窗、同步当前步骤，并在终态关闭；BUA 不显示桌面接管，CUA 保留接管触点。四类完整产品闭环及 Windows 证据仍按任务清单推进，不因本次增量验收提前标记 Story Done。

同日按正式 `Codex MCP → Gateway → 计划片段 → Jev → Yonder CUA` 路径复验，发现首动作前交回后的恢复阻断：Jev 交回会正确把任务保持为 `running` 并写出慢脑意图，但尚未取得 Admission、也没有历史 attempt；慢脑经 Gateway 提交的新片段可接受，执行却固定返回 `-32012`。该失败已形成[macOS 结构化验证记录](../../../../openspec/changes/tm-s8-unified-product-chain/verification-fast-slow-gateway-macos.md)，修复并重跑前不得宣称快慢脑产品链路通过。

2026-10-08 已关闭上述 replan 恢复边界：TM-S9 的事件驱动执行入口会先为同一 `running` 任务恢复 Runtime 投影，再在新片段首个副作用前幂等取得 Admission；精确回归覆盖“首动作前交回、无 attempt、无 Admission、同任务提交新片段并连续执行两个槽位”。详见[macOS 独立验证记录](../../../../openspec/changes/tm-s8-unified-product-chain/verification-replan-recovery-macos.md)。该结论只关闭 `-32012` 实现阻断；真实 CUA/BUA/Office/Command 产品闭环及快慢脑配对样本仍未完成，Story 继续保持 `verifying`。
