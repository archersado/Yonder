# 固定上游 Windows 输入归属源码审计（2026-11-05）

## 审计范围

固定 `agent-desktop v1.1.0` commit `d3dc190afbf34cd2377252398ddd5c30efbd093a` 加 CU-S4 唯一 hit-test 补丁。审计已准备源码中的 `crates/windows/src/input/mouse/mouse_send.rs`、`crates/windows/src/input/keyboard/keyboard_send.rs` 和动作 Click 路径；没有改上游源码，也没有发送桌面输入。

## 发现

- 鼠标 `SendInput` 构造的 `MOUSEINPUT.dwExtraInfo` 固定为 `0`；键盘 `KEYBDINPUT.dwExtraInfo` 同样固定为 `0`。该固定版本没有把 Yonder 的 `attempt_id` 或 Worker 身份放进这些输入事件。
- [微软的鼠标低级钩子结构文档](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-msllhookstruct)与[键盘低级钩子结构文档](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct)说明，注入标志只表示事件由程序注入，`dwExtraInfo` 是随事件提供的附加信息。由此推断：仅按注入标志忽略事件，会把其他程序的注入也当作当前 Driver；仅按时间窗口或 `dwExtraInfo = 0` 归属，同样没有当前 attempt/Worker 的证据。
- Windows Click 的 UIA Invoke/Legacy 默认动作路径并不等同于上述 `SendInput` 事件流。是否产生可观察的低级输入事件、如何归属，仍需单独验证；不能用 `SendInput` 路径的结论替代。

## 对门禁的影响

`INPUT-02` 要求当前 attempt 的合成事件零误分，未绑定事件冻结派发。固定源码当前缺少可供观察器验证的输入归属标记；现有 Supervisor 的 `input_generation` 契约测试是模拟状态，不是 Windows 平台观察器。用户输入抢占和合成输入区分尚未完成，原生 `dispatch` 保持 `unsupported/not_delivered`。

下一步须在固定 revision 上设计并验证同一动作入口可用的归属机制，包括 UIA 语义动作与物理 `SendInput` 路径；若需要改变上游补丁范围或系统边界，先更新 AD-CU-07 与 OpenSpec，再实施。不能靠忽略全部 injected 事件或估计时间窗口通过门禁。
