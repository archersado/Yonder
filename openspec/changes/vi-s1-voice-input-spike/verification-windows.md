# Windows 验证：语音输入

日期：2026-10-23
结论：FAIL（能力清单通过；产品失败反馈未通过；显式音频未执行）

- 无录音能力探测确认 `Windows.Media.Capture` 与 `Windows.Media.SpeechRecognition` API 存在，且探测没有请求权限、没有打开麦克风。
- 桌宠“语音输入”可打开原生窗口；Windows Adapter 返回未实现后，窗口仍停留在“正在请求系统权限…”，没有显示“暂不可用”或失败原因，判定失败。
- 未请求麦克风权限、未采集音频，因而不构成显式授权、统一测试音频或停止释放验证。
- 能力证据：[`capability.json`](../../../spikes/voice-input/evidence/windows-20261023/capability.json)。
- UI Automation 结构化证据：[`result.json`](../../../apps/desktop/evidence/vi-s1-windows-20261023/result.json)；复现脚本：[`inspect-voice-fallback.ps1`](../../../apps/desktop/evidence/vi-s1-windows-20261023/inspect-voice-fallback.ps1)。

Windows 能力清单任务完成，但 VI-S1 的 Windows 显式音频任务与双平台门禁仍未通过。
