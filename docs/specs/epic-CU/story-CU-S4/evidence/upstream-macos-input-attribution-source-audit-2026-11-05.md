# 固定上游 macOS 输入归属源码审计（2026-11-05）

## 审计范围

固定 `agent-desktop v1.1.0` commit `d3dc190afbf34cd2377252398ddd5c30efbd093a` 加 CU-S4 唯一 hit-test 补丁。只读审计已准备源码 `crates/macos/src/input/mouse/` 与 `crates/macos/src/input/keyboard/keyboard_event.rs`；当前环境为 WSL，没有 macOS 原生运行证据。

## 发现

- 鼠标与键盘事件分别由 `CGEvent::new_mouse_event`、`CGEvent::new_keyboard_event` 创建。当前构造函数设置 flags 等动作参数，但未设置 `EVENT_SOURCE_USER_DATA`，也未接收 Yonder 的 attempt/Worker 身份。
- [Apple 的 CGEventField 文档](https://developer.apple.com/documentation/coregraphics/cgeventfield?changes=_2_6_8)列出 `eventSourceUserData`，可存放最多 64 位由事件源提供的数据；[CGEventSource 文档](https://developer.apple.com/documentation/coregraphics/cgeventsource/userdata?changes=_2)说明事件可带此数据，并可按事件读取。它是待验证的归属候选字段，不等于当前源码已使用或 macOS Event Tap 已可靠接收到。
- AX 语义动作与 CGEvent 注入路径不同；其是否产生可观察的低级事件、如何归属，仍需 macOS 原生样本证明。

## 门禁状态

当前不能把 macOS 合成事件归为 `driver_current_attempt`，`INPUT-02` 没有原生证据。若扩展上游补丁，须对每条实际注入路径设置并验证当前 attempt/Worker 的标记；事件丢失、无法读取标记或与当前尝试不匹配时按 `unknown` 冻结。没有 macOS 测试环境时结果只能是 `environment_unavailable`，不能通过双平台门槛。
