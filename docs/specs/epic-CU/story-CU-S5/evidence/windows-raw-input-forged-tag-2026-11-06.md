# CU-S5 Windows Raw Input 与伪造标记诊断（2026-11-06）

## 范围

本次只运行自建测试窗口与独立子进程，不接 CU-S4 Worker、Agent Gateway 或真实应用，不执行 CUA 动作。测试窗口注册键盘 Raw Input；每轮只有确认该窗口处于前台才发送固定 `F24` key-up。父进程与子进程各发送 10 次，子进程核对窗口类名和父 PID，两者故意使用相同 `dwExtraInfo` 测试值。窗口只统计 `WM_KEYUP`、`WM_INPUT`、`RAWKEYBOARD` 类型、设备句柄空值和标记匹配数，不保存其他键值或正文。任一前台核验失败时零注入或停止后续轮次；`admission_ready` 始终为 false。

## 构建与原生结果

- Windows x86_64 MSVC，Rust 1.89.0。`cargo.exe +1.89.0 build --manifest-path spikes/agent-desktop-driver-v2/input-visibility/Cargo.toml --locked --offline` 成功；`cargo.exe +1.89.0 clippy --manifest-path spikes/agent-desktop-driver-v2/input-visibility/Cargo.toml --locked --offline -- -D warnings` 通过。
- `Cargo.lock` SHA-256：`39db0a8c2ecbc0f76aab1db4ef898ea3cf6b63863cda5ca375651d9c3af14717`；最终格式化并复跑的诊断 exe SHA-256：`a10471ac920f8c309f1ab6a8fc7bdb9cd49dc58ff6b14d7bf09862a9629cf3bb`。它是开发测试制品，不是已签名分发制品。
- 从 PowerShell 启动一次有效样本：`current_sent_count=10`、`other_sent_count=10`、`f24_keyup_messages=20`、`raw_input_messages=20`、`raw_read_errors=0`、`raw_keyboard_messages=20`、`f24_raw_messages=20`、`f24_raw_null_device=20`、`f24_raw_tag_match=20`、`admission_ready=false`。
- 直接从 WSL 启动或部分 PowerShell 启动时，自建窗口无法取得前台，输出 `foreground_unavailable`、`sent=false`。这是测试环境条件未满足，探针按门禁没有发送事件。

早期单次诊断曾把 `GetRawInputData` 返回的实际字节数与传入缓冲区容量比较，误计为读取失败并推导“Raw Input 不可见”。修正为检查失败码及键盘结构最小长度后，原生有效样本表明 Raw Input **能看见**本次 `SendInput` 注入。此前的相反推断已作废，不用于架构判定。

## 结论与边界

在这一 Windows 自建窗口样本中，当前进程与独立子进程注入的事件在已读取的 Raw Input 字段上均呈现空设备句柄和同一个可由发送方设置的标记。因而**仅凭 `dwExtraInfo`/Raw Input 可见字段无法证明事件属于当前 attempt/Worker**；把相同标记一律当作 `driver_current_attempt` 会误分独立子进程。此结果并不否定 Raw Input 作为事件来源，也不证明所有设备、注入 API、权限和桌面场景的覆盖。真实用户事件、鼠标、UIAccess、观察器失效和 macOS 均未在此样本验证，V2-INPUT-02 完整验收未通过。

微软的 [Raw Input 文档](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input)说明设备输入通过 `WM_INPUT` 交付；[INPUT_MESSAGE_ORIGIN_ID 文档](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ne-winuser-input_message_origin_id)把某些 UIAccess 注入也归入 `IMO_HARDWARE`。这些文档没有提供当前 Worker 的不可伪造归属。原生结果和源码共同排除“只给 `SendInput` 打标记即可开放派发”的 v2 方案。
