# macOS 无录音能力探测

日期：2026-09-17  
结论：PASS（仅只读能力清单）

## 命令

```bash
swift spikes/voice-input/macos-capability-probe.swift
```

## 结果

- AVFoundation与Speech框架可加载。
- `zh-CN`识别器存在、当前可用，并报告支持本机识别。
- 麦克风与语音识别权限均为`not_determined`。
- 探针未请求权限、未打开麦克风、未采集或保存音频。

结构化证据见`spikes/voice-input/evidence/macos-20260917/capability.json`。本结果不覆盖显式授权、真实测试音频、部分/最终文本或停止释放，不能接受AD-VI-01。
