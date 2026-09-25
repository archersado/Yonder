# FI-S1 规范文件身份与受控操作

Story: FI-S1
Epic: FI
Status: design-review
OpenSpec: fi-s1-file-identity-spike

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

Proposed AD-FI-01已建立。macOS临时目录样本已通过文件身份、别名归并、授权根、重复写锁和原子提交验证；WPS真实打开/关闭DOCX的文件引用与系统锁对照也已通过。2026-09-24 已补齐可移交的Windows统一样本及Office/WPS锁探针，但当前macOS环境不能生成Windows实机证据；Windows结果未通过独立复核前不接受ADR、不接产品Gateway。

## OpenSpec 与验证

[Spike Change](../../../../openspec/changes/fi-s1-file-identity-spike/proposal.md)。只验证技术路线，不代替产品实施Proposal。

[Windows运行包就绪记录](../../../../openspec/changes/fi-s1-file-identity-spike/windows-readiness.md)不是Windows PASS证据。
