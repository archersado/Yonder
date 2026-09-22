# AG-S6 本地 Agent 注册与撤权 Verification Goal

状态：macOS PASS（2026-09-23）；Windows 按用户决定暂缓，完整 Story 不 Archive。

## 验证范围

- 使用真实 SQLite 数据库完成 schema 16→17 迁移，并重启后保留 `agent_registry` 状态。
- 在 macOS 本地设置入口登记 Agent，列表展示状态与最近连接时间。
- 未登记 Agent 的握手/写请求返回稳定权限拒绝。
- 已登记 Agent 可以握手并创建任务。
- 禁用或撤权后，该 Agent 不能新建任务、提交步骤、控制任务或接收输入。
- 撤权后活动 UDS 会话立即断开；已有任务保持可见，用户仍可查看和终止。

## macOS 结果

- SQLite `user_version` 为 `17`；注册状态重启后仍保留。
- UI 成功登记 `ag_s6_verification_agent`，并显示最近连接时间。
- 未登记 Agent 通过真实 UDS 访问被拒绝：`Agent已禁用、撤权或未登记`。
- 已登记 Agent 通过真实 UDS 成功创建任务 `task_4ab44d5a957794349a9bce64c3c98664`。
- 禁用与撤权后，`task.create`、`task.step.declare`、`task.control` 均返回 `-32003`。
- 使用声明 `session_id` 与 `user_input` 的真实 UDS 会话执行撤权后，客户端收到 EOF，`SOCKET_CLOSED`；前后 `lsof` 显示活动连接关闭。
- 既有任务在 Task Space 中仍可见，并可通过 UI 取消；SQLite 状态从 `created` 变为 `cancelled`。

## 证据

- 结构化结果：`apps/desktop/evidence/ag-s6-agent-registration-20260923/result.json`
- Agent 管理截图：`apps/desktop/evidence/ag-s6-agent-registration-20260923/agent-management.png`
- Task Space 截图：`apps/desktop/evidence/ag-s6-agent-registration-20260923/task-space.png`
- 会话关闭日志：`apps/desktop/evidence/ag-s6-agent-registration-20260923/disconnect-session.log`

## 最终回归

- `cargo test --offline --locked -p yonder-desktop`：11项通过、0失败。
- `cargo test --offline --locked -p yonder-application`：19项通过、0失败。
- `cargo test --offline --locked -p yonder-adapters`：34项通过、0失败。
- `node --check apps/desktop/ui/agent-settings.js`：通过。
- `openspec validate ag-s6-local-agent-registration`：通过。

普通连接退出使用`unregister`仅移除会话；禁用或撤权才通过`disconnect_agent`发送关闭信号。路由测试已按该边界修正，不把自然断开伪装为强制关闭。

Windows 证据缺失时，不宣称完整 AG-S6 双平台通过，也不 Archive。
