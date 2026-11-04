# AG-S6 Windows 本地 Agent 注册与撤权验证

日期：2026-11-01
平台：Windows 11 / Named Pipe
结论：**PASS**

## 原生端到端结果

- 未登记 Agent 的真实 `gateway.hello` 返回 `-32003`（`Agent已禁用、撤权或未登记`）。
- 通过原生“Agent 管理”窗口登记专用 Agent 后，Named Pipe 握手与 `task.create` 成功。
- UI 禁用后，同一连接的 `task.step.declare` 写请求和新连接握手均返回 `-32003`；已有任务仍可读。
- UI 重新启用后，测试任务经正式 `task.cancel` 进入 `cancelled`，数据未删除。
- 带 `session_id` 和 `user_input` 能力声明的活动会话在 UI 确认撤权后收到 EOF；撤权后的新连接继续返回 `-32003`。
- 重启 Yonda 后，原生管理窗口仍显示本次 Agent“已撤权”，证明 SQLite 持久化生效。
- 全程未执行 CUA。

机器可读结果：`apps/desktop/evidence/ag-s6-agent-registration-windows-20261101/native-agent-registration-result.json`（`passed=true`）。原生截图：同目录 `native-agent-revoked.png`，显示 `ag-s6-win-7d26ccb9adf8` 为“已撤权”。

## 夹具修正与清理

验证过程中修正了三类夹具误判：WebView2 列表把一整行暴露为组合可访问名称，不能用 Agent ID 精确相等；禁用后允许读取已有任务，权限断言必须使用写请求；只有声明 `session_id`/`user_input` 的输入会话由撤权信号主动关闭，普通 Gateway 会话不属于该会话表。Windows 通知区还保留过已结束调试进程的同名幽灵图标，最终夹具会逐个尝试可见 Yonda 图标，只接受真正打开管理窗口的入口。

多次夹具定位产生的三个 `created` 测试任务已通过正式 Gateway 取消，所有 `ag-s6-win-*` 测试身份均为 `disabled` 或 `revoked`。只读 SQLite 收尾审计结果为：非终态测试任务 `0`，启用测试 Agent `0`。清理脚本见 `cleanup-test-artifacts.ps1`。

## 完成边界

本报告补齐 AG-S6 当前 Change 的 Windows 产品 E2E，与既有 macOS 证据共同满足双平台注册/禁用/撤权子范围。正式安装签名、跨用户 Named Pipe ACL 与生产认证属于 AG-S1 门禁，不由本 Change 冒充完成。
