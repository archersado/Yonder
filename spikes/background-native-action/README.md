# CU-S3 后台原生动作 Spike

macOS 隔离样本使用 `NSWorkspace.OpenConfiguration.activates = false` 启动临时 fixture，并以就绪标记完成 Observe。探针同时确认前台应用未改变、未新增 trycua Worker、不支持动作没有隐式兜底，并输出 Task Space 时间线所需的最小事实。

```bash
./run-macos.sh evidence/macos-20260918/result.json
./run-macos.sh --real-app /System/Applications/Calculator.app evidence/macos-real-app-20260918/result.json
./inspect-app-intents-macos.py evidence/app-intents-macos-20260918/result.json
./run-eventkit-macos.sh evidence/eventkit-reminder-macos-20260918/result.json
```

fixture 只在临时目录构建和运行，不进入产品包。Windows 按用户决定暂缓。

macOS子范围已通过，结果见`evidence/macos-20260918/result.json`；独立结论见OpenSpec的`verification-macos.md`。

EventKit样本先运行独立`authorize`模式，再由同一已授权临时App运行`background`模式；只有后者创建并删除固定临时提醒，不输出任何提醒内容。
