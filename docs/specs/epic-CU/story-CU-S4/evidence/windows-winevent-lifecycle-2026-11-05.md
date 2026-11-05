# Windows WinEvent 窗口生命周期探针（2026-11-05）

## 方法

在 Yonder 自建 WinForms fixture 的第一个窗口已存在时，由独立 PowerShell Host 进程调用 `SetWinEventHook(EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, ..., idProcess = fixture PID, WINEVENT_OUTOFCONTEXT)`。注册线程用 `Application.DoEvents` 持续处理消息。fixture 随后销毁第一个窗口、创建第二个窗口、销毁第二个窗口。回调只记录 `OBJID_WINDOW / CHILDID_SELF` 的事件种类和 HWND；没有读取窗口内容或派发输入。

## 结构化结果

| 项 | 结果 |
|---|---|
| fixture PID | `30028` |
| 第一个 HWND | `5117068` |
| 第二个 HWND | `5182604` |
| 收到的窗口事件顺序 | `DESTROY(5117068) → CREATE(5182604) → DESTROY(5182604)` |
| Hook / fixture 退出 | 正常 |

首个窗口在注册前创建，因此没有预期收到它的 `CREATE`。这次实验验证了受控 WinForms 生命周期事件在独立进程中的可见性与顺序；没有触发 HWND 复用，也没有覆盖 WPS、文件资源管理器或不同完整性级别。

另一次受控屏障实验将 Hook 扩至当前桌面的所有进程。在目标第二个窗口创建并发布句柄后，Host 创建自己的标记窗口，以该窗口由系统生成的 `EVENT_OBJECT_CREATE` 回调作为屏障。本次 fixture PID `60564`，相关回调依次为 `DESTROY(3735798) → CREATE(3801334) → CREATE(859456，Host 标记)`；`barrier_after_target_events = true`，两个进程均正常退出。此前一次相同实验也得到相同的相关事件顺序。它验证了受控情况下的顺序，尚不足以证明跨应用、队列拥塞或观察器故障下的屏障可靠性。

## 下一轮门槛

WinEvent 的 out-of-context 回调异步到达，调用方必须维持消息循环。仅观察当前已处理的回调数，不能证明更早生成的事件已排空。正式身份 oracle 需要：

1. 在首次快照前同时启动 Host 与 Worker 的生命周期观察，并证明观察器就绪；丢失、重启或队列状态不可判时拒绝。
2. 用可验证的事件屏障，证明身份采样前生成的目标窗口销毁/创建事件都已被处理；不能只靠延时。Host 自有窗口的系统 `CREATE` 事件已在受控实验中形成候选屏障，正式实现还须验证全局事件顺序、队列拥塞、Hook 生命周期与 Worker 独立观察的一致性。
3. 对同进程窗口重建、实际 HWND 复用、观察器故障与目标应用样本验证零误投递；派发动作前后均核对，并单独解决最终核对到物理副作用之间的竞态。

在这些条件通过前，不生成正式 `window_token_sha256`，不开放原生派发。

参考：[SetWinEventHook 的异步与消息循环要求](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook)、[系统生成窗口 CREATE/DESTROY 事件](https://learn.microsoft.com/en-us/windows/win32/winauto/event-constants)、[out-of-context 回调顺序](https://learn.microsoft.com/en-us/windows/win32/winauto/out-of-context-hook-functions)。
