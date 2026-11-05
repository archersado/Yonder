# CU-S5 v2 固定源码只读门禁（2026-11-05）

## 环境与输入

当前主机为 WSL，读取 Windows 挂载盘上的 Yonder 仓库；没有 macOS 原生环境。本次没有执行桌面动作。运行 `python3 spikes/agent-desktop-driver-v2/source_gate.py`，先核对 v1 固定 revision、唯一补丁 SHA-256 和 tracked source tree SHA-256，再检查已知源码接缝。结果的 `inspection_scope=known_seams_only`，不是穷尽调用图或 v2 补丁证据。

固定上游 commit：`d3dc190afbf34cd2377252398ddd5c30efbd093a`；v1 patch SHA-256：`3f625a14254d66126f931cb479fb67b698e0ea2a5a2e4a8caad1f8f9e3149023`；已准备源码树 SHA-256：`78f37b45808177601ea6f5b3af1fcd9095ead4ecb9dcd405c3b7abe0e20f9872`。三者与 v1 manifest/stamp 一致；v1 文件未修改。

## 结果

| 已知接缝 | 探针结果 | 含义 |
|---|---|---|
| action 重新从 RefStore 读取并解析保存引用 | true | 同窗替代元素有重绑定路径，不能以调用前身份核验放行动作 |
| action 自动等待可再次尝试解析 | true | stale 拒绝不一定是终态，v2 必须禁用重试或证明不会绑定替代目标 |
| 单次 dispatch 丢弃返回的落点 handle | true | 当前公开结果不保留交付对象作为 Host 验证依据 |
| Windows 鼠标、键盘 `dwExtraInfo = 0` | true、true | 没有当前 attempt/Worker 标记 |
| macOS 鼠标、键盘源码未设置 `EventSourceUserData` | true、true | 合成事件没有当前 attempt/Worker 标记 |

判定：`identity_gate_passed=false`、`input_gate_passed=false`、`native_dispatch_admitted=false`。这是一份**阻断证据**，不是候选淘汰的最终结论；v2 补丁和双平台原生样本仍待实施。`python3 -m py_compile` 与 `git diff --check` 通过。

Windows 官方文档说明低级鼠标/键盘钩子超时可能被[静默移除](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc)，应用无法确认其持续安装；[Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input) 能以 `WM_INPUT` 取得设备来源，但不能据此推断它覆盖所有其他程序注入事件。Apple 的 [CGEvent `eventSourceUserData`](https://developer.apple.com/documentation/coregraphics/cgeventfield?changes=_2_6_8) 仅定义用户数据字段；是否能安全归属当前 Worker 需要原生验证。[Event Tap 超时禁用](https://developer.apple.com/documentation/coregraphics/cgeventtype/tapdisabledbytimeout?changes=_2_1&language=objc) 也必须纳入观察器失效样本。
