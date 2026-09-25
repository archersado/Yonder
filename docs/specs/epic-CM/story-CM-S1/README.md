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

AD-CM-01 已接受 macOS-only Runtime：标准库结构化参数、stdout/stderr 各 64 KiB、超时/取消/超限完整进程组停止路线可进入产品 Port/Adapter。Windows Job Object 按用户决定暂缓并保持 unavailable。通用命令风险确认协议尚未定案，因此本增量不开放 Agent Gateway、CLI/MCP 或 Jev Command 候选。

## OpenSpec 与验证

[Spike Change](../../../../openspec/changes/cm-s1-command-executor-spike/proposal.md)。该Change只验证技术路线，不代替后续产品实施Proposal。

[macOS独立验证](../../../../openspec/changes/cm-s1-command-executor-spike/verification-macos.md)已通过；完整双平台Spike与Story保持未完成。

产品 Runtime 增量：[cm-s1-macos-command-runtime](../../../../openspec/changes/cm-s1-macos-command-runtime/proposal.md)。
