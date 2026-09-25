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

2026-09-25：macOS 文件组合增量及其[独立 Verification Goal](../../../../openspec/changes/do-s2-macos-file-runtime/verification-goal.md)已通过。Application 已组合受控 File Port 与 OOXML Document Port，覆盖三格式读取、默认另存、源快照保护、暂存结构复验、可信本机覆盖和宿主锁拒绝；全仓 127 项 Rust、70 个活动 OpenSpec、33 项 Python 测试及架构/协议门禁通过。Agent Gateway、任务步骤/Observe/完成链路和 Windows 仍未接通，完整 Story 保持 `verifying`，不 Archive/Done。
