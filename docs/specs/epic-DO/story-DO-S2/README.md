# DO-S2 OOXML Document Port

Story: DO-S2  
Epic: DO  
Status: verifying  
OpenSpec: do-s2-ooxml-semantic-transform

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

DO-S1与Accepted AD-E0-04已确定Rust进程内实现。Accepted AD-DO-01的内存语义转换子范围已通过DOCX/XLSX/PPTX统一样本和48项Workspace回归，并于2026-09-24归档；产品文件写入仍依赖FI-S1提供规范绝对路径、文件身份、同文件写互斥和锁冲突。

## OpenSpec 与验证

[已归档语义转换 Change](../../../../openspec/changes/archive/2026-09-24-do-s2-ooxml-semantic-transform/proposal.md)及其[独立 Verification Goal](../../../../openspec/changes/archive/2026-09-24-do-s2-ooxml-semantic-transform/verification-goal.md)仅覆盖内存字节转换。FI-S1通过后另建文件写入与Gateway Change，不复用选型Spike或本次子范围冒充产品文件闭环。
