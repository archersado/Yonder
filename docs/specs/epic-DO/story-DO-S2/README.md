# DO-S2 OOXML Document Port

Story: DO-S2  
Epic: DO  
Status: verifying  
OpenSpec: do-s2-macos-file-runtime

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

DO-S1与Accepted AD-E0-04已确定Rust进程内实现。Accepted AD-DO-01的内存语义转换子范围已通过DOCX/XLSX/PPTX统一样本并归档；FI-S1 macOS-only Runtime 已提供规范绝对路径、文件身份、源快照保护、同文件写互斥、宿主锁和原子提交，可进入产品文件组合。Windows 与 Agent Gateway 继续后补。

## OpenSpec 与验证

[已归档语义转换 Change](../../../../openspec/changes/archive/2026-09-24-do-s2-ooxml-semantic-transform/proposal.md)及其[独立 Verification Goal](../../../../openspec/changes/archive/2026-09-24-do-s2-ooxml-semantic-transform/verification-goal.md)仅覆盖内存字节转换。

macOS 文件组合增量：[do-s2-macos-file-runtime](../../../../openspec/changes/do-s2-macos-file-runtime/proposal.md)。该增量不代替后续 Gateway/任务闭环或 Windows 验证。

Agent 文档 Gateway 增量：[do-s2-agent-gateway](../../../../openspec/changes/do-s2-agent-gateway/proposal.md)。

2026-09-25：macOS 文件组合增量及其[独立 Verification Goal](../../../../openspec/changes/do-s2-macos-file-runtime/verification-goal.md)已通过。Application 已组合受控 File Port 与 OOXML Document Port，覆盖三格式读取、默认另存、源快照保护、暂存结构复验、可信本机覆盖和宿主锁拒绝。

2026-09-26：macOS Agent 文档 Gateway 增量及其[独立 Verification Goal](../../../../openspec/changes/do-s2-agent-gateway/verification-goal.md)已通过。归属 Agent 可经统一 Gateway 使用同任务 `read` + `create-new` 文件授权执行唯一文本替换与默认另存；协议、CLI/MCP、TaskHost、TM-S7 attempt 和无路径安全响应已接通。任务的后续 Observe/推进/完成继续使用既有 Gateway 方法。Windows 原生验证仍未接通，完整 Story 保持 `verifying`，不 Archive/Done。
