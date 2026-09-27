# EX-S2 macOS 计划片段执行 Verification Goal

状态：进行中（2026-09-27）。

## 目标

在 macOS 正式桌面组合根中，以同一已认证 Gateway 会话提交一个受限 CUA 计划片段，并执行一个槽位。证据必须同时证明：Jev 只返回已提交候选；动作经既有桌面租约与 Observe；成功才推进片段游标；低置信或手动交回会写入任务事件及 Outbox；不保存模型输入、截图、键入内容或完整 Agent Payload。

## 已通过的结构验证

- `cargo test -p yonder-application --lib`：41/41；
- `cargo test -p yonder-adapters --lib`：64/64；
- `cargo test -p yonder-desktop --lib`：11/11。

这些结果只证明协议、Application、SQLite 与桌面组合根可构建并保持既有回归；不替代真实 Keychain 凭据、辅助功能授权、前台桌面目标和远端 Jev 的 macOS 原生样本。

## 待运行原生样本

需在用户主动提供的可控测试窗口中，以无敏感正文的 `computer_click` 候选运行一次。输出仅保留 task/plan/sequence、选择类别、Observe 成败、Outbox 数量与权限状态；若缺少 Keychain API Key、辅助功能权限、网络或测试窗口，应记录为环境阻塞，不将 EX-S2 标为 PASS。Windows 仍按用户决定暂缓。
