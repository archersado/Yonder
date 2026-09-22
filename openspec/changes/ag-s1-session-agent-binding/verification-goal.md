# AG-S1 本地会话身份绑定独立 Verification Goal

日期：2026-09-22。关联AG-S1首帧绑定增量、Accepted AD-AG-05与`openspec/changes/ag-s1-session-agent-binding/`。环境为macOS Story worktree真实`yonder-desktop`进程；Windows按用户要求暂缓。本Goal只验证本轮macOS增量，不Archive完整Story。

## 结果

PASS。相关单测`yonder_application::gateway::tests::local_first_hello_binds_the_declared_agent_id`与`yonder-cli`MCP工具构造单测通过；`yonder-desktop`和`yonder`在Story worktree构建完成。

验证脚本以隔离HOME启动真实桌面进程，生产路径为`HOME/Library/Application Support/com.yonder.desktop/agent.sock`；数据目录权限0700、UDS权限0600。生产UDS验证证明：

- 首帧`gateway.hello`的有效`agent_id`绑定连接。
- 绑定后同一连接携带不同`agent_id`读取任务或重握手均被`-32003`拒绝。
- `agent-a`和`agent-b`分别经不同连接创建任务，跨连接越权读取被拒绝，任务归属隔离。
- `yonder mcp`缺省`YONDER_AGENT_ID`与无效`bad id`均启动失败；有效身份可完成MCP `initialize`。

[结构化证据](../../../apps/desktop/evidence/ag-s1-session-binding-20260922/result.json)由[可重复验证脚本](../../../apps/desktop/evidence/ag-s1-session-binding-20260922/check.py)生成，包含两个任务ID与全部断言结果。该样本验证OS用户私有UDS内的连接绑定与归属隔离，不代表细粒度认证；正式安装签名与Windows Named Pipe仍另行验证。
