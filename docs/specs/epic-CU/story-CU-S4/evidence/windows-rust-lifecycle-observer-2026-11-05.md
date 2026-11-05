# Windows Rust 生命周期观察器验证（2026-11-05）

## 范围

隔离 Spike harness 新增 `host_oracle::windows_lifecycle::LifecycleObserver`。它在专用线程注册全桌面 out-of-context `SetWinEventHook`，只收集窗口 `CREATE/DESTROY`；使用隐藏的 Host 自有 `STATIC` 窗口生成系统事件作为屏障。事件数组上限 4096；溢出、回调重入、Hook 失败、标记未到达或观察器断连均返回错误。当前接口只返回目标 HWND 的创建/销毁计数与屏障到达状态，不生成 `window_token_sha256`，不参与 Worker 动作派发。

## 验证

固定 Rust 1.89.0、Windows `x86_64-pc-windows-msvc`，离线运行：

- `cargo.exe +1.89.0 test --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked --all-targets --offline --quiet`：21 个单元测试、1 个 Worker containment 测试及 5 个 Worker 合约测试全部通过。
- `cargo.exe +1.89.0 clippy --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked --all-targets --offline -- -D warnings`：通过。

原生测试在 Hook 启动后创建和销毁一个隐藏 Win32 窗口，两次屏障都收到标记；第一次见到目标创建事件，第二次的目标销毁计数递增。这验证了 Rust 封装在当前桌面会话中可运行；跨进程事件可见性仍由前一份 PowerShell fixture 证据支持，尚未完成 Rust Host/Worker 双进程一致性验证。

随后把观察器接入 `native-read-probe`，在固定 `AutomationId = YonderProbeButton` 的独立 WinForms 进程上，于 snapshot 前后各运行一次屏障并比较事件计数。PID `61504` 的结果：`complete = true`、`nodes_observed = 8`、`selector_matches = 1`、`host_identity_matches_adapter = true`、`lifecycle_barrier_observed = true`、`lifecycle_target_create_count = 0`、`lifecycle_target_destroy_count = 0`、`lifecycle_changed_during_snapshot = false`、`window_generation_proven = false`。窗口在 Hook 启动前已存在，所以零创建计数符合预期。测试窗口正常退出，私有 RefStore 目录已清理。

首次使用 WSL 创建的 `D:` 盘私有目录时，上游因 `private file DACL grants an untrusted principal` 拒绝 snapshot；接入前版本在同一窗口也复现。改用 Windows 在用户 Temp 下新建、由上游保护 ACL 的目录后成功。这是私有状态目录权限问题，不是 WinEvent 观察器故障；后续原生样本必须由 Windows 端创建私有状态目录。

额外原生故障测试在 Hook 就绪后，连续创建并销毁 4096 个隐藏 Win32 窗口，超过观察器的 4096 条事件上限。其后的屏障返回明确的事件溢出错误，未提供可用身份样本；测试约 34 秒通过。该测试证明当前实现对本地缓冲耗尽会 fail closed，但不覆盖操作系统在 Hook 回调之前丢失通知、观察器进程崩溃或高完整性目标。

## 未通过的门禁

尚未验证实际 HWND 复用、队列拥塞与 Hook 故障恢复、目标应用覆盖、Host/Worker 同步、最终核验与副作用之间的竞态。观察器启动与屏障成功不是正式窗口代际证明；原生动作仍关闭。
