# AG-S1 本地 CLI/MCP 独立 Verification Goal

日期：2026-09-15。关联AG-S1 AC8–12、Accepted AD-AG-05与`openspec/changes/ag-s1-local-cli-mcp/`。环境为macOS真实Yonda预览进程；Windows按用户要求暂缓。本Goal只验证本轮macOS增量，不Archive完整Story。

## 结果

PASS。最终锁定离线回归共33项通过（protocol 5、application 6、adapters 16、desktop 5、CLI 1），架构关联检查及`git diff --check`通过。预览包同时含`yonder-desktop`与无App外壳的`yonder` CLI。当前工具链未安装rustfmt，因此`cargo fmt --check`未运行成功；未自动安装工具链组件，不把该项写成通过。

正式桌面PID38726为唯一Yonda桌面进程；`~/Library/Application Support/com.yonder.desktop`权限0700，`agent.sock`权限0600且由桌面进程持有。`lsof -nP -a -p 34241 -iTCP`无输出，上一进程SIGTERM留下的陈旧端点由新进程安全覆盖；重启后MCP联调再次通过。

[结构化MCP证据](../../../apps/desktop/evidence/local-cli-mcp-20260915/result.json)证明initialize 2025-06-18、7项工具发现、`codex-cli`归属任务创建、读回、取消并保留全部通过。[真实Codex输出](../../../apps/desktop/evidence/local-cli-mcp-20260915/codex-result.txt)证明Codex CLI实际调用`yonder/task_create`与`task_get`，创建任务`task_d884324efb3bc4817836cb8ce4b1b877`，正式SQLite只读核对为`codex-cli / Codex 本地连接验证 / created / sequence 1`。

首次真实Codex会话使用`approval: never`，工具被Codex自身审批策略阻止，Yonder未收到请求；随后仅对该临时验证设置Yonder工具为approve后通过，没有修改长期审批策略。全局`codex mcp get yonder`显示stdio入口enabled。Windows Named Pipe、细粒度Agent注册/撤权与正式安装签名仍未验证，相关任务保持未勾选。

## 2026-09-22 MCP空闲生命周期补充验证

此前为清理宿主侧遗留进程加入“连续10分钟无stdin输入后退出”，并假设父进程会在下一次MCP调用时重新拉起。2026-10-12 正式Codex长会话证伪该假设：宿主保留已关闭transport，`task_list`返回`Transport closed`，即使Yonder Desktop和Codex输入连接器仍运行也无法恢复。该旧结论撤销，不再作为通过证据。

生命周期修正后，MCP随调用方stdio存活：单元回归证明接收器会阻塞等待迟到输入并在发送端关闭后返回EOF；源码不再包含业务空闲计时或`recv_timeout`。`cargo test --offline --locked -p yonder-cli`共18项通过，`openspec validate ag-s1-local-cli-mcp --strict`、架构检查与`git diff --check`通过。仍需在装入新二进制的新Codex MCP会话中补一次长会话调用与Yonder重启后后续调用样本；本修复不重放工具请求，Windows仍暂缓。

清理记录：2026-09-22修复前本机存在356个旧预览路径`yonder mcp`子进程；按完整命令精确匹配发送TERM后2秒计数为0。该清理只证明存量处置，不作为新生命周期逻辑的验证替代。
