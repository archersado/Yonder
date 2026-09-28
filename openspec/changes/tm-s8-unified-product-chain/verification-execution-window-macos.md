# TM-S8 统一执行浮窗 macOS 增量验证

日期：2026-09-28  
平台：macOS  
基线：`dev` / `e18ccf7`  
结论：通过（Windows 仍按主人决定暂缓）

## 验证范围

验证所有 Gateway 执行能力共用的顶部规划/执行浮窗基础机制，并以正式 BUA、CUA 链路覆盖两种交互语义：

- 所有执行任务均显示当前步骤与已提供的后续计划；没有可信计划时明确显示“未提供后续计划”，不得伪造。
- 只有 CUA 桌面控制态显示“接管电脑”；BUA 等非桌面能力不得伪装成正在控制电脑。
- 任务到达完成、失败、取消或中断终态后关闭浮窗。
- SQLite 任务状态与事件仍为事实源，React 窗口不成为状态所有者。

## BUA 正式链路

通过已打包 `Yonda.app` 的 Unix Domain Socket Gateway，以 `codex-cli` 会话创建任务 `task_81d4451e3fac4a7460fa8619023bc88f`，并由 Yonder 创建 ego-lite Task Space `ego:140`。

1. 声明步骤“创建浏览器空间并打开 Example Domain”，打开 `https://example.com/`，页面标题为 `Example Domain`。
2. 推进并声明步骤“核验 Example Domain 页面”，调用 `browser.execute/observe`，任务保持 `running@8`。
3. macOS 原生可访问性与截图同时证明顶部窗口标题为“Yonder 正在执行任务”，当前步骤为“核验 Example Domain 页面”，规划区域为“未提供后续计划”，且不存在“接管电脑”按钮。
4. 声明“完成浏览器任务空间”并调用 `browser.execute/finish`，Task Space 标记 `finished=true`，任务到达 `completed@14`；随后顶部窗口关闭。

## CUA 正式链路

通过同一正式 Gateway 运行受控原生窗口任务 `task_153a60c6a81eb80925238ab1d1763f44`，使用协议 `1.12` 的单调用步骤语义：

1. `computer.step` 执行 `type_text` 并完成 Observe，原生目标确认收到输入；任务保持 `running@5`。
2. 持有桌面租约期间，macOS 原生可访问性证明顶部窗口标题为“Yonder 正在控制您的电脑”，当前步骤为“向当前工作窗口输入测试标记”，并显示“接管电脑”按钮。
3. 同一 CUA Worker 连续执行第二个动作，PID `55644` 被复用；任务完成后 Worker 正常退出，Observe 截图完成清理。
4. 最终任务为 `completed@10`，共 10 个事件，Recording 未启动；顶部控制浮窗随终态关闭。

结构化结果：

```json
{
  "protocol": { "major": 1, "minor": 12 },
  "computer_capability": "available",
  "native_target_matches": true,
  "attempt_phase": "observed",
  "one_call_step": true,
  "observe_screenshot_created_and_cleaned": true,
  "worker_exited_after_completion": true,
  "final_status": "completed",
  "final_sequence": "10",
  "events": 10,
  "recording_started": false,
  "passed": true
}
```

## 自动化回归

- `cargo test -p yonder-desktop --lib --locked`：16/16 通过。
- `cargo test -p yonder-desktop --bin yonder-desktop --locked`：11/11 通过。
- `node apps/desktop/check-cua-control.mjs`：通过；覆盖 CUA 标题与接管按钮、非 CUA 标题与禁用接管、当前步骤和计划显示。

## 未覆盖范围

- 本证据只关闭“所有任务共用顶部执行浮窗”的实现与 CUA/BUA 生命周期验证项。
- BUA 交回、Jev 快慢脑对照、Office 与 Command 完整产品任务仍保留在 TM-S8 清单中，未提前宣告完成。
- Windows 原生证据按主人决定暂缓。
