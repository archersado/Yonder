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

真实采集阶段必须由测试者显式点击开始，并使用公开统一测试音频；证据不得保存PCM或转写正文。
