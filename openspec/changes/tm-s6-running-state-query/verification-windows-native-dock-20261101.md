# TM-S6 Windows 真实运行态收起门禁验证

日期：2026-11-01
结论：**PASS（桌面消费者与长时原生门禁）**

## 真实链路

- Windows Desktop 经 Named Pipe Gateway 创建真实 ego-lite BUA 任务并进入 `running`。
- 从运行态展开窗口开始连续观察 **190.896 秒**；超过 180 秒空闲阈值后仍为 200×200，没有错误收起。
- 随后通过正式步骤推进、声明和 `browser.execute(finish)` 将任务收口为 `completed`，外部 Task Space `finished=true`。
- 完成后重新进入可收起周期，从 finish 响应回包起 **166.53 秒**观察到 112×56 底部停靠；宿主物理日志为 168×84，符合本机 150% DPI。
- finish 的终态展示/空闲重新计时发生在外部完成请求返回之前，因此从响应回包计时小于180秒；运行期间的旧空闲时间没有直接导致完成后立即收起。
- ego-browser `listTaskSpaces()` 最终返回 `[]`；测试 Agent 已禁用。全程未执行 CUA。

机器可读结果与可重复夹具：

- `apps/desktop/evidence/tm-s6-running-dock-windows-20261101/native-running-dock-result.json`
- `apps/desktop/evidence/tm-s6-running-dock-windows-20261101/verify-running-dock.ps1`

宿主诊断在 running 期间记录 30/60/90/120/150 秒进度，最终只在任务完成后记录 `桌宠停靠：edge=bottom`。夹具的首次短运行在开始长时计时前暴露 PowerShell 参数名冲突；异常清理已将 `ego:17` 正式 finish。最终通过会话使用 `ego:18`，不存在遗留 Task Space。

## 完成边界

本报告证明 Windows Desktop 已消费全局运行态/准入查询并执行保守收起门禁。PR 隔离审阅和 Archive 仍是流程项；多显示器断开属于 CX-S2，不由本 Goal 关闭。
