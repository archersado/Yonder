# CM-S1 结构化命令执行

Story: CM-S1
Epic: CM
Status: design-review
OpenSpec: cm-s1-command-executor-spike

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

Proposed AD-CM-01已建立；标准库结构化参数、64 KiB输出截断和macOS完整进程组停止样本PASS。Windows Job Object按用户决定暂缓；ADR接受前不开放产品Gateway。

## OpenSpec 与验证

[Spike Change](../../../../openspec/changes/cm-s1-command-executor-spike/proposal.md)。该Change只验证技术路线，不代替后续产品实施Proposal。

[macOS独立验证](../../../../openspec/changes/cm-s1-command-executor-spike/verification-macos.md)已通过；完整双平台Spike与Story保持未完成。
