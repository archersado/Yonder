# EX-S2 macOS 计划片段执行 Verification Goal

状态：进行中（2026-09-27）。

## 目标

在 macOS 正式桌面组合根中，以同一已认证 Gateway 会话提交一个受限 CUA 计划片段，并执行一个槽位。证据必须同时证明：Jev 只返回已提交候选；动作经既有桌面租约与 Observe；成功才推进片段游标；低置信或手动交回会写入任务事件及 Outbox；不保存模型输入、截图、键入内容或完整 Agent Payload。

## 已通过的结构验证

- `cargo test -p yonder-application --lib`：42/42；其中 Gateway 用例确认 `task.plan.submit` 与 `task.plan.execute` 必须先协商协议 1.31，旧版协商或不可用组合根均会被拒绝；
- `cargo test -p yonder-adapters --lib`：65/65；其中 SQLite 用例确认片段不可变重放、CAS、槽位推进及交回均与事件/Outbox 同事务；
- `cargo test -p yonder-desktop --lib`：11/11。

这些结果只证明协议、Application、SQLite 与桌面组合根可构建并保持既有回归；不替代真实 Keychain 凭据、辅助功能授权、前台桌面目标和远端 Jev 的 macOS 原生样本。

## 已通过的 Gateway 原生样本

2026-09-27 以正式 debug bundle、隔离 `/tmp/ex2-*` HOME 和私有 stdio 运行 `apps/desktop/check-plan-fragment-submit-macos.py`。自动登记的 `local-test-agent` 成功完成协议 1.31 握手、任务创建和 `task.plan.submit`；结果为 `hello_ok=true`、`plan_capabilities=true`、`accepted=true`、`sequence_advanced=true`。样本没有调用 Jev、未派发 CUA 动作、未保存候选参数或用户内容，因此只证明 EX-S2 的 Gateway/CAS/SQLite/Outbox 提交子范围。

## 待运行原生样本

需在用户主动提供的可控测试窗口中，以无敏感正文的 `computer_click` 候选运行一次。输出仅保留 task/plan/sequence、选择类别、Observe 成败、Outbox 数量与权限状态；若缺少 Keychain API Key、辅助功能权限、网络或测试窗口，应记录为环境阻塞，不将 EX-S2 标为 PASS。Windows 仍按用户决定暂缓。

2026-09-27 已以 `apps/desktop/check-plan-fragment-execute-macos.py` 尝试运行固定输入的隔离窗口样本。fixture 可创建，但系统窗口聚焦校验未通过，脚本在任何 Gateway 执行请求前以 `fixture-focus-failed` 结束；没有调用 Jev、没有派发 CUA、没有记录敏感数据。按照前台目标身份保护，该失败是原生环境证据阻塞，不可用其他前台窗口替代。待 macOS 窗口聚焦能力恢复后重跑。
