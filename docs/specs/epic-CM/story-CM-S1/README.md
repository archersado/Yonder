# CM-S1 结构化命令执行

Story: CM-S1
Epic: CM
Status: verifying
OpenSpec: cm-s1-agent-command-approval

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

AD-CM-01 已接受 macOS-only Runtime：标准库结构化参数、stdout/stderr 各 64 KiB、超时/取消/超限完整进程组停止路线可进入产品 Port/Adapter。AD-CM-02 已接受 Agent 命令“提议—本机批准—一次执行”契约；Rust 协议、CLI/MCP 映射、Gateway 提议登记、有界内存 Registry、TaskHost 本机入口及 Task Space 批准卡已实施。Windows Job Object 按用户决定暂缓并保持 unavailable。

复用 TM-S7 启动事务的一次性 Command 执行用例、Gateway 执行接线及真实 macOS Command Adapter 已实施；协议替换、重放、撤权、超时、unknown 不重试与无命令正文边界已有自动化回归。独立 macOS 原生批准界面证据与 Verification Goal 已通过；Windows Job Object 与 Windows 原生 E2E 按用户决定延期，因此完整 Story 保持 `verifying`，不标记 Done。

## OpenSpec 与验证

[Spike Change](../../../../openspec/changes/cm-s1-command-executor-spike/proposal.md)。该Change只验证技术路线，不代替后续产品实施Proposal。

[macOS独立验证](../../../../openspec/changes/cm-s1-command-executor-spike/verification-macos.md)已通过；完整双平台Spike与Story保持未完成。

产品 Runtime 增量：[cm-s1-macos-command-runtime](../../../../openspec/changes/cm-s1-macos-command-runtime/proposal.md)。

Agent 本机批准增量：[cm-s1-agent-command-approval](../../../../openspec/changes/cm-s1-agent-command-approval/proposal.md)。[独立 Verification Goal](../../../../openspec/changes/cm-s1-agent-command-approval/verification-goal.md) 已以原生 macOS `WKWebView` 证据和真实 TaskHost/SQLite/Gateway/Command Adapter 集成测试组合通过；界面夹具未连接正式任务库，证据边界已在 Goal 中明示。完整 Story 因 Windows 延期暂不 Archive。
