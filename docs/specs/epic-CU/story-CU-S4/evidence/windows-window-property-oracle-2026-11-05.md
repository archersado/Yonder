# Windows 窗口 property 代际探针（2026-11-05）

## 问题与方法

`WindowInfo.id = w-<HWND>` 与进程启动令牌不能区分同一进程内销毁后复用的 HWND。本探针仅在 Yonder 自建 WinForms fixture 上，使用独立 PowerShell Host 进程经 `SetPropW` / `GetPropW` 给第一个窗口附加一个固定测试标记，然后关闭窗口并新建第二个窗口。没有向产品应用写入 property，也没有派发 CUA 输入。

## 结构化结果

| 项 | 结果 |
|---|---|
| fixture PID | `62728` |
| 跨进程 `SetPropW` | 成功，`GetLastError = 0` |
| 第一个窗口读取标记 | 匹配 |
| 关闭后旧 HWND 有效性 | 无效 |
| 第二个窗口读取标记 | 不匹配 |
| 两个窗口的 HWND 是否相同 | 否 |
| fixture 退出 | 正常 |

这是单个、同完整性级别的受控样本。它证明了该环境下跨进程写入与读取可行，**没有**证明 HWND 复用时的行为、目标应用兼容性、权限覆盖或派发期间的原子性。

## 门禁结论

此路线不能作为当前冻结的只读窗口身份 oracle。`SetPropW` 会改变第三方窗口的 property，并受 UIPI 限制；微软要求调用方在窗口销毁前移除自己添加的 property，Host 崩溃时也无法保证这一点。即使派发前读取标记成功，读取与上游解析及副作用之间仍有窗口销毁/重建竞态。因此不填充 `target_identity.window_token_sha256`，不开放原生动作派发。

参考：[SetPropW 与 UIPI](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setpropw)、[Window properties 生命周期](https://learn.microsoft.com/en-us/windows/win32/winmsg/using-window-properties)、[Windows 窗口销毁语义](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features)。
