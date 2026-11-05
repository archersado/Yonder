# Windows 双进程生命周期观察器实验（2026-11-05）

## 范围与方法

隔离 harness 的 `native-lifecycle-probe` 是只读诊断入口，不实现产品 Worker bridge 或动作。它在独立进程中启动 Rust `LifecycleObserver`，通过标准输入接收固定的 `barrier` / `stop` 诊断命令，只输出窗口事件计数、屏障状态和进程身份摘要。

受控 WinForms fixture 创建一个窗口；两个独立探针进程均就绪后，各自对该 HWND 运行屏障。随后 fixture 销毁旧窗口并创建新窗口；两个探针再次对旧 HWND 运行屏障。fixture 与两个探针均正常退出，没有派发 CUA 输入。

## 结构化结果

| 项 | 结果 |
|---|---|
| fixture PID | `52612` |
| 两个探针的进程启动摘要 | 相同 |
| 销毁前两个探针 | `creates = 0`、`destroys = 0`、`marker_observed = true`、`owner_matches_initial = true` |
| 销毁后两个探针 | `creates = 0`、`destroys = 1`、`marker_observed = true`、`owner_matches_initial = false` |
| 销毁前/后的两个探针结果 | 分别完全一致 |
| 新旧 HWND | 不同 |
| `window_generation_proven` | 两个探针均为 `false` |

21 个单元测试、1 个 Worker containment 测试、5 个 Worker 合约测试及 Clippy `-D warnings` 通过。结果证明这一个受控样本中，两个独立 Rust 观察器均看到了旧窗口销毁；没有证明 HWND 复用、事件丢失时的故障判定、队列拥塞、不同权限或真实应用覆盖。两个探针当前只输出计数，尚未参与同一动作帧的 Host/Worker 冻结身份比较。
