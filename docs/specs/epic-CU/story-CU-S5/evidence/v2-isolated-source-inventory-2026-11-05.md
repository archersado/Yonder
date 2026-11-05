# CU-S5 v2 独立源码与发送出口清单（2026-11-05）

## 可复现输入

执行 `python3 spikes/agent-desktop-driver-v2/prepare_upstream.py`，从本地已固定的 CU-S4 Git 对象克隆同一上游 revision，再在**独立目录**应用按哈希绑定的 v1 hit-test 补丁。随后执行 `python3 spikes/agent-desktop-driver-v2/source_gate.py`。v2 基线 patch set SHA-256 为 `39cce93149b7d6267f3acf3b30819f138ca15d0d0a5581c16e3ec36761b11ec4`，准备后源码树 SHA-256 为 `78f37b45808177601ea6f5b3af1fcd9095ead4ecb9dcd405c3b7abe0e20f9872`，v2 样本定义 SHA-256 为 `593d153577f201831bdbd2e3103bff9b634dec740a4153ecbc769662c0bc73d1`。独立 [source manifest](../../../../../spikes/agent-desktop-driver-v2/source-manifest.json) 的阶段明确为 `baseline_source_prepared_no_v2_patch`；尚无 v2 行为补丁、制品或原生动作。

## 已知动作路径

- core `execute_by_ref` 读回 RefStore，经自动等待进入 `dispatch_resolved_landing`；该函数再次调用 `resolve_saved_ref`，滚动恢复时可能再次解析，最终把当次解析的 handle 传给 `execute_verified_action`。因此保留 snapshot 时的旧对象并非当前上游动作语义。
- Windows `Click` 使用同一 UIA 元素的 Invoke/Legacy 链；物理双击、右击、键盘等走其他终端，不能从 Click 的对象绑定推导它们安全。
- macOS `Select` 可先打开菜单，再按值搜索并操作新出现的子元素；只保留最初控件的 AX 对象，不足以证明被选择菜单项的交付身份。该结论是源码推断，尚无 macOS 原生证据。

## 合成输入出口

| 平台 | 源码文件 | 基线状态 |
|---|---|---|
| Windows | `input/mouse/mouse_send.rs`、`input/keyboard/keyboard_send.rs`、`system/shell/shell_surface_raise.rs` | 三处 `SendInput` 的 `dwExtraInfo` 均为 0 |
| macOS | `input/mouse/mod.rs`、`mouse_move.rs`、`mouse_drag.rs`、`mouse_scroll.rs`、`input/keyboard/keyboard_event.rs` | 五处 HID post 文件；键盘文件另有 `post_to_pid`，未设 attempt 归属字段 |

探针扫描非测试 Rust 文件并将这些文件集合与审阅清单比较，结果 `synthesis_seams_match_reviewed_inventory=true`。这只覆盖上述调用形式和固定源码，不证明别的系统 API 不能产生输入；补丁必须覆盖异常释放/清理路径。最终 `identity_gate_passed=false`、`input_gate_passed=false`、`native_dispatch_admitted=false`。

## 观察器判定

微软 [低级鼠标钩子文档](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc)明确指出超时后可静默移除且应用无法获知；[Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input)提供设备输入的 `WM_INPUT`，但没有据此证明覆盖所有程序注入。[输入来源枚举](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ne-winuser-input_message_origin_id)还把带 UIAccess 的程序注入归入 `IMO_HARDWARE`，因此仅靠该枚举也不能判定“真实用户”。这些 API 的组合是否满足 CU-S5 零漏分门槛仍未证明；本次没有模拟输入或操作桌面。
