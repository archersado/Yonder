# CM-S1 结构化命令执行

Story: CM-S1
Epic: CM
Status: implementing
OpenSpec: cm-s1-macos-command-runtime

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

AD-CM-01 已接受 macOS-only Runtime：标准库结构化参数、stdout/stderr 各 64 KiB、超时/取消/超限完整进程组停止路线可进入产品 Port/Adapter。AD-CM-02 已接受 Agent 命令“提议—本机批准—一次执行”契约；Rust 协议、CLI/MCP 映射、Gateway 提议登记、有界内存 Registry、TaskHost 本机入口及 Task Space 批准卡已实施。Windows Job Object 按用户决定暂缓并保持 unavailable。

当前待实施的是复用 TM-S7 启动事务的一次性 Command 执行用例、Gateway 执行接线、unknown 不重试等安全回归和独立 macOS 原生验证；在这些项目完成前不开放可执行能力，也不将完整 Story 标记 Done。

## OpenSpec 与验证

[Spike Change](../../../../openspec/changes/cm-s1-command-executor-spike/proposal.md)。该Change只验证技术路线，不代替后续产品实施Proposal。

[macOS独立验证](../../../../openspec/changes/cm-s1-command-executor-spike/verification-macos.md)已通过；完整双平台Spike与Story保持未完成。

产品 Runtime 增量：[cm-s1-macos-command-runtime](../../../../openspec/changes/cm-s1-macos-command-runtime/proposal.md)。

Agent 本机批准增量：[cm-s1-agent-command-approval](../../../../openspec/changes/cm-s1-agent-command-approval/proposal.md)。Task Space 浏览器交互夹具已覆盖完整预览、批准一次与撤销；该夹具不替代后续 macOS 原生验证。
