# Windows 目标覆盖范围只读核验（2026-11-05）

## 门禁原因

Windows out-of-context `SetWinEventHook` 的全局范围是**当前桌面**；普通完整性级别的进程不能读取提升权限应用的 UI。如果目标不在观察器可覆盖范围，零生命周期事件不能作为“窗口未变化”的证据。依据：[SetWinEventHook](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook)、[UIAccess 安全边界](https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-securityoverview)。

## 实现与现有证据

隔离 Host `capture_window` 在 PID/HWND、进程创建时间核验前，增加只读检查：目标与本进程会话相同；目标和调用线程桌面均为输入桌面且名称相同；目标进程完整性 RID 不高于本进程。观察器启动时分别在调用线程和 Hook 线程读取输入桌面名称，两者不同或不可读即拒绝；每次屏障前后在 Hook 线程复核输入桌面，切换时拒绝。调用 Windows `GetThreadDesktop`、`GetUserObjectInformationW`、`ProcessIdToSessionId`、`OpenProcessToken` 与 `GetTokenInformation(TokenIntegrityLevel)`；不写入目标进程或窗口。

- 原生单元测试创建本进程隐藏 Win32 窗口，新的 `capture_window` 接受该受控目标。
- 负向原生测试创建另一个未切换到前台的 Windows desktop，把受控线程绑定到该 desktop；`verify_desktop_scope` 明确返回 `target is outside input desktop`，测试结束后线程与 desktop 句柄正常释放。
- 独立 WinForms fixture 上，Rust 诊断观察器通过 `ready` 和屏障，`owner_matches_initial = true`；fixture 正常退出。该结果证明同会话、同桌面、同权限的一个外部窗口仍可通过只读核验。
- `native-read-probe` 和诊断观察器在身份核验成功时输出 `target_scope_verified = true`；它不等于窗口代际证明。
- 当前构建在新的独立 WinForms fixture 上复测：`target_scope_verified = true`、`marker_observed = true`、`owner_matches_initial = true`、`window_generation_proven = false`；fixture 正常退出。

## 未通过的门禁

尚无提升权限目标、锁屏或 UAC 安全桌面的实际拒绝日志；运行过程中桌面变化和 Hook 丢失的拒绝路径尚未实测。负向测试只覆盖 Yonder 自建的非输入 desktop。正式 Host/Worker 派发门禁继续关闭，跨权限或跨桌面目标必须 fail closed。
