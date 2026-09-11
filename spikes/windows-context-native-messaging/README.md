# Windows 上下文与 Native Messaging Spike

运行协议自检：

```bash
npm test
```

从 WSL 验证当前 Windows 前台窗口（只输出标题是否存在与长度）：

```bash
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$(wslpath -w "$PWD/foreground-window.ps1")"
```

本 Spike 不读取浏览器 History 数据库，不保存窗口标题，也不实现后台操作采集。

扩展 ID 固定为 `mofdddjaniddgalgegfdjegiegpneokc`。点击扩展按钮才开始 Record（徽标显示 `REC`），再次点击停止；Service Worker 重启后默认关闭。扩展不申请 `history` 权限，并在发送前拒绝 `tab.incognito`。

Windows 实机编译与注册：

```powershell
.\build-host.ps1 -OutputPath "$env:LOCALAPPDATA\Yonder\yonder-context-spike.exe"
.\register-host.ps1 -HostPath "$env:LOCALAPPDATA\Yonder\yonder-context-spike.exe" -Browser Both
```

验证结束后运行 `.\unregister-host.ps1` 撤销注册。

浏览器实连需要在 `chrome://extensions` 或 `edge://extensions` 打开开发人员模式，选择“加载解压缩的扩展”，目录为：

```text
C:\Users\gongjian\AppData\Local\Yonder\context-spike\extension
```

加载后点击一次扩展按钮；显示绿色 `REC` 且 `native-host-events.log` 新增计数即为连接成功。

Windows 操作 Hook 验证程序默认不运行；启动后仍需用户按 Enter 才开始，最长 10 秒，只输出键鼠事件数量：

```powershell
.\build-operation-hook.ps1 -OutputPath "$env:LOCALAPPDATA\Yonder\operation-hook-spike.exe"
& "$env:LOCALAPPDATA\Yonder\operation-hook-spike.exe" 10
```

协议依据：[Chrome Native Messaging](https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging) 与 [Microsoft Edge Native Messaging](https://learn.microsoft.com/microsoft-edge/extensions-chromium/developer-guide/native-messaging)。
