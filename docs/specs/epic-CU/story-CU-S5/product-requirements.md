# CU-S5 产品需求

## 问题与范围

CU-S4 已发现旧限定引用可在同窗重建后绑定替代元素，且 Windows 低级输入钩子可能静默失效。用户 2026-11-05 选择继续 v2。本 Story 在 CU-S4 原期限内判定固定 `agent-desktop` 能否通过**最终交付身份**与**人类输入抢占**两个门槛。它只产出可复核的技术结论，不修改产品 Adapter、Gateway、任务状态或 trycua 基线。

## 需求来源分类

- 原始需求：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)“两条执行路径、MVP 主干链路”要求模型无关 CUA；[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)“执行原则、验证方案”要求用户接管、每步 Observe 与故障不误报。
- 后续用户变更：2026-11-05“继续做 v2”，允许在固定候选上进一步验证交付与输入门禁。
- 已接受或待决架构约束：[AD-CU-07](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-07-AGENT-DESKTOP-DRIVER-SPIKE.md) 保持整体淘汰、统一样本和原截止日；[AD-CU-08](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-08-AGENT-DESKTOP-V2-DELIVERY-PROOF.md) 只授权隔离机制验证。
- 待审设计建议：保留 snapshot 原生元素对象，禁用动作时谓词重新解析；Windows `dwExtraInfo` 与 macOS `eventSourceUserData` 可试作归属载体，但须验证接收和防伪，不能直接视为通过。

## 验收条件

| ID | 条件 | 来源 |
|---|---|---|
| V2-01 | v1 源码/补丁/manifest/证据不变；v2 固定同一上游 revision，并独立记录 patch set、源码树、依赖锁与样本哈希。 | AD-CU-07 补丁扩围 |
| V2-02 | Windows/macOS 均证明一次动作从 snapshot 到副作用交付使用同一原生对象；同窗元素替换、整窗重建、完整 HWND 复用/AX 对象替换零误投递。无法证明则拒绝。 | ADS-06、AD-CU-08 |
| V2-03 | 真实用户、其他程序注入及当前 Worker 合成输入按统一分类零漏分/零误分；observer 丢失、权限/桌面变化时冻结下一派发。 | ADS-04、AD-CU-08 |
| V2-04 | `dispatch` 在 V2-02/03 双平台门禁和独立复核前保持 `unsupported/not_delivered`；不执行真实用户任务。 | AD-CU-08 |
| V2-05 | 沿用 CU-S4 固定应用、Observe、取消、故障、清理与分发样本；不能仅靠缩小动作/应用范围通过。到期输出 Accepted/Rejected 结论，缺环境不称通过。 | ADS-03～10 |

## 验收映射

| 来源章节或证据 | 对应验收 |
|---|---|
| 产品简报“两条执行路径、MVP 主干链路”；补充材料“执行原则、验证方案” | V2-02、03、05 |
| 用户 2026-11-05“继续做 v2” | V2-01～05 |
| AD-CU-07“受限补丁扩围决策门槛、Spike 门槛” | V2-01～05 |
| CU-S4 [Windows 同窗替换诊断](../story-CU-S4/evidence/windows-identity02-read-only-stale-ref-2026-11-05.md) | V2-02 |
| CU-S4 [输入钩子诊断](../story-CU-S4/evidence/windows-input-hook-diagnostic-2026-11-05.md) | V2-03 |
