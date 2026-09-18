# AG-S1 本地 CLI/MCP 独立 Verification Goal

日期：2026-09-15。关联AG-S1 AC8–12、Accepted AD-AG-05与`openspec/changes/ag-s1-local-cli-mcp/`。环境为macOS真实Yonda预览进程；Windows按用户要求暂缓。本Goal只验证本轮macOS增量，不Archive完整Story。

## 结果

PASS。最终锁定离线回归共33项通过（protocol 5、application 6、adapters 16、desktop 5、CLI 1），架构关联检查及`git diff --check`通过。预览包同时含`yonder-desktop`与无App外壳的`yonder` CLI。当前工具链未安装rustfmt，因此`cargo fmt --check`未运行成功；未自动安装工具链组件，不把该项写成通过。

正式桌面PID38726为唯一Yonda桌面进程；`~/Library/Application Support/com.yonder.desktop`权限0700，`agent.sock`权限0600且由桌面进程持有。`lsof -nP -a -p 34241 -iTCP`无输出，上一进程SIGTERM留下的陈旧端点由新进程安全覆盖；重启后MCP联调再次通过。

[结构化MCP证据](../../../apps/desktop/evidence/local-cli-mcp-20260915/result.json)证明initialize 2025-06-18、7项工具发现、`codex-cli`归属任务创建、读回、取消并保留全部通过。[真实Codex输出](../../../apps/desktop/evidence/local-cli-mcp-20260915/codex-result.txt)证明Codex CLI实际调用`yonder/task_create`与`task_get`，创建任务`task_d884324efb3bc4817836cb8ce4b1b877`，正式SQLite只读核对为`codex-cli / Codex 本地连接验证 / created / sequence 1`。

首次真实Codex会话使用`approval: never`，工具被Codex自身审批策略阻止，Yonder未收到请求；随后仅对该临时验证设置Yonder工具为approve后通过，没有修改长期审批策略。全局`codex mcp get yonder`显示stdio入口enabled。Windows Named Pipe、细粒度Agent注册/撤权与正式安装签名仍未验证，相关任务保持未勾选。
