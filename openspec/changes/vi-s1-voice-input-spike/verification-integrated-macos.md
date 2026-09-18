# VI-S1 macOS 桌宠集成验证

日期：2026-09-17  
结论：通过当前macOS Spike样本；Story仍受Windows样本与ADR门禁约束。

## 独立验证目标

验证语音入口不再依赖独立测试App，且真实桌面上的入口、收音卡、原生采集和小龙状态属于同一Yonda进程。

## 证据

- `cargo build -p yonder-desktop`通过，Objective-C原生Adapter链接`AVFoundation`与`Speech`。
- ego CUA从Yonda小龙的“语音输入”按钮进入，收音卡显示“正在聆听（最长20秒）”、转写区和停止按钮。
- WindowServer结构化取证显示同一PID同时拥有`200×200`小龙窗口与`310×220`语音卡；二者水平间隔8点并处于同一Y坐标。
- 原生回调以事件驱动`requesting/listening/processing/reviewing/failed/cancelled`；`requesting/listening/processing`覆盖投影为`voice_listening`临时视觉态，结束后恢复任务状态。该视觉态使用麦克风提示，不显示任务“收到请求”的✓。
- 关闭卡片、Esc或进程退出均调用取消并释放采集；启动时显式隐藏任务卡和语音卡，避免遗留双面板。

## 未覆盖

Windows原生Adapter、设备切换、运行中撤权和双平台统一样本尚未完成，因此AD-VI-01保持Proposed，不Archive。
