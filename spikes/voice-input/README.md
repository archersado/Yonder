# VI-S1 单轮语音输入 Spike

当前只读探针不会请求权限或开启麦克风。

```bash
swift macos-capability-probe.swift
```

真实收音探针必须由用户在窗口内点击“开始测试”，最多20秒且不落盘：

```bash
./build-macos-explicit-probe.sh
open "/private/tmp/yonda-voice-spike/Yonder Voice Spike.app"
```

Windows探针在Windows 11 PowerShell中运行：

```powershell
powershell -ExecutionPolicy Bypass -File .\windows-capability-probe.ps1
```

Windows显式收音候选探针先编译，再由测试者在控制台输入`START`授权。输入其他内容会立即退出且不打开麦克风：

```powershell
powershell -ExecutionPolicy Bypass -File .\build-windows-explicit-probe.ps1
& "$env:TEMP\yonder-voice-spike\windows-explicit-probe.exe"
```

当用户已经在当前Agent会话中明确授权，但宿主无法把交互式控制台输入转发给原生程序时，可使用等价的显式参数入口；不得在没有当轮用户授权时调用：

```powershell
& "$env:TEMP\yonder-voice-spike\windows-explicit-probe.exe" --authorized-start
```

探针最多收音12秒，通过Windows语音合成播放公开测试句，只输出计数、布尔状态与HRESULT；不打印或保存PCM、转写正文。

真实采集阶段必须由测试者在对应UI点击开始或在控制台输入`START`，并使用公开统一测试音频；证据不得保存PCM或转写正文。
