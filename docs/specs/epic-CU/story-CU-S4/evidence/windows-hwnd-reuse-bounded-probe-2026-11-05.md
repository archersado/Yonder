# Windows HWND 有界重建探针（2026-11-05）

## 方法

在 `x86_64-pc-windows-msvc` 上运行 `native-hwnd-reuse-probe`。探针只在自身进程中创建和销毁隐藏 `STATIC` 窗口，同时运行独立 WinEvent 观察线程；没有向其他应用派发输入。初始窗口先通过 Host 只读范围核验和事件屏障，然后最多重建 512 次。输出只含哈希、计数与布尔值。

## 实测

- 512 次重建完成；初始窗口的 `CREATE`/`DESTROY` 分别由屏障确认，`creates_before = 1`、`destroys_before = 0`、`creates_after = 1`、`destroys_after = 1`。
- `slot_reused_with_changed_handle = true`：至少一次新窗口与初始窗口的低 16 位相同，但完整 HWND 不同。
- `reuse_observed = false`：这 512 次内未见完整 HWND 值复用，因此没有同完整 HWND 的 Host 身份字段比对证据。
- 曾将上限临时提高到 1500 次，WinEvent 队列达到容量，屏障返回 `WinEvent queue overflowed or reentered` 且进程退出码为 1；未把该次运行用于代际结论。正式探针仍采用 512 次上限。
- 另运行显式 `--seek-exact` 模式：在同一自建进程中最多创建并销毁 70,000 个隐藏窗口，不启动生命周期观察器。该次完整达到上限、退出码 0；`slot_reused_with_changed_handle = true`、`reuse_observed = false`、`same_host_fields = false`、`window_generation_proven = false`。这只说明本次未命中完整 HWND 值复用；它没有动作路径与事件连续性证据，也不能证明完整值永不复用。此长循环不列入常规测试。

微软对 Windows NT 窗口句柄的[实现说明](https://devblogs.microsoft.com/oldnewthing/20070717-00/?p=25983)描述了表槽索引和随重用变化的标识位，同时明确该机制属于可变更的实现细节。本次观测与说明相符，但不能把低 16 位或当前完整 HWND 行为写成受支持的代际协议。

## 判定

`window_generation_proven = false`。该实验只覆盖当前 Windows、单个自建进程、隐藏窗口与有限重建次数；没有复现完整 HWND 回绕、没有 Host/Worker 双进程一致性、没有覆盖固定应用集或 `execute_by_ref` 的真实派发竞态。原生 Backend 继续 `unsupported/not_delivered`。
