# macOS 受控会话输入 Verification Goal

结论：子范围 PASS。

2026-09-19，在操作者显式确认后的 30 秒窗口内，原生 listen-only 探针收到 64 条受控会话事件；固定容量达到上限后立即形成 1 个 `gap` 并关闭采集。输出没有输入正文、坐标、截图、AX 文本或完整事件 Payload。

这只证明显式用户控制租约内的无正文事件边界与队列缺口可观察，不证明物理用户来源，也不授权产品持久化、自动 Replay 或桌宠录制状态。

结构化证据：[result.json](../../../spikes/recording-capture/evidence/controlled-session-macos-20260919/result.json)。
