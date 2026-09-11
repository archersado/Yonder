# AD-E0-05 Windows 原生上下文

- 状态：Accepted（Windows 范围）
- Story：E0-S5
- OpenSpec：`e0-validate-windows-context`
- 日期：2026-09-11

## 决定

Windows 前台应用与窗口变化由 Rust Adapter 使用 WinEvent Hook/UI Automation 事件驱动采集；用户操作序列只在用户显式 Recording 期间安装低级键鼠 Hook，并在停止、异常或进程退出时无条件解除。

Chrome/Edge 使用同一 Manifest V3 扩展，经 Native Messaging 进入独立的 Rust 薄 Host；Host 只负责校验协议并通过 Windows Named Pipe 转发给 Yonder Core，不持有 Recording 状态，不引入 Node Runtime、HTTP/TCP 或第二套业务模型。

## 证据

- Windows 前台窗口原生探针成功取得应用、进程和窗口元数据，证据不含标题正文。
- Chrome 与 Edge 均通过精确扩展来源和 HKCU 注册完成 Native Messaging 实连。
- UTF-8、分片、多帧、1 MiB 上限与标准输出隔离检查通过。
- 浏览器 Recording 实测得到 `recording.started → tab.updated/tab.activated → recording.stopped`，停止后无标签事件、无遗留 Host 进程。
- Windows 低级 Hook 仅在用户按 Enter 后运行；10 秒内统计键盘 43、鼠标 572，结束后 `recording=false`、`hooks_released=true` 且进程退出。未记录键值或坐标。
- Chrome/Edge 均未授予隐私窗口权限；扩展不申请 `history` 权限，并在发送前再次拒绝 `tab.incognito`。

## 架构围栏

- Recording 状态属于 Application/SQLite；扩展会话状态仅是 Spike 控制手段，不进入正式状态模型。
- Native Host 是协议 Adapter，不得直接调用其他 Adapter或持久化上下文。
- 不读取 Chrome/Edge History 数据库，不使用通配 `allowed_origins`。
- 日志不得记录 URL、标题、键值、坐标、正文或完整 Payload。
- 密码框、安全桌面与排除应用的识别必须在正式 Adapter Story 中作为 fail-closed 合约；未识别时不得生成可回放动作。
- C#/Node Harness 仅用于 Spike；产品实现统一使用 Rust。
- macOS 按已确认范围延期，本 ADR 不构成 macOS 技术路线或原生 E2E 证据。
