# 独立 Verification Goal：DS-S1 Windows BUA 展示状态延迟

日期：2026-11-01；平台：Windows。结论：**PASS（`browser.execute(create) → executing` 子范围）**。

## 方法与结果

- 当前 Windows Desktop 通过 Named Pipe Gateway 1.19 派发真实 `browser.execute(create)`，计时从请求写入开始，到 pet WebView 的 `applyPresentation` 同步配置后首次报告 `presentation-change/executing` 为止。
- executing 观测上界为 **14.225ms**，诊断出现时 Browser 请求仍在途，低于 Accepted AD-E0-01 的 300ms当前优化目标；完整 Browser create 请求耗时413.119ms。
- 随后通过真实 `browser.execute(finish)` 正常完成并关闭 `ego:7`。从 finish 请求写入到 success 诊断为347.383ms，其中包含外部 Task Space 完成耗时，不能作为纯 UI 应用延迟与300ms比较。
- finish 响应后873.994ms恢复到本机既有代表任务的 `paused`；该值包含终态展示脉冲的剩余时间，不是核心状态应用 SLA。
- 任务 `task_4a5ab80604737e1095a21faaf94fabf2` 最终为 `completed`，`reference_finished=true`；测试 Agent 已禁用，Yonda进程和 ego Task Space 均无遗留。`cua_executed=false`。
- 机器可读结果与夹具：`apps/desktop/evidence/ds-s1-windows-20261101/native-bua-presentation-latency-result.json`、`verify-native-bua-presentation-latency.ps1`。

## 首次失败与修复关系

首次同口径探针虽然在14.744ms内观察到 executing，但 Desktop 误选未激活的更高 ego Runtime，create 最终返回 `-32012`；该任务按安全语义进入 `interrupted/unknown`，没有 Browser 引用。BU-S1 修复及证据见 `../bu-s1-ego-lite-bridge-windows/verification-windows-active-runtime-20261101.md`。未知副作用没有自动重试；修复、重建并确认 active Runtime 后才重新创建独立任务。

## 边界

本结果关闭 Windows BUA executing 的直接状态应用延迟子范围，并记录真实 success 与恢复顺序；不把包含外部 Browser 完成或终态脉冲的时间冒充纯 UI 延迟。waiting_for_user、真实 pause/failed、完整可用状态、macOS同口径及九态完整原生链路仍未完成，DS-S1 总 Goal 不变。
