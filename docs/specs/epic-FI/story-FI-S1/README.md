# FI-S1 规范文件身份与受控操作

Story: FI-S1
Epic: FI
Status: implementing
OpenSpec: fi-s1-macos-file-runtime

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

AD-FI-01 已接受 macOS-only Runtime：临时目录样本和 WPS 真实打开/关闭 DOCX 已通过文件身份、别名归并、授权根、重复写锁、宿主锁与原子提交验证。Windows 统一样本与 Office/WPS 锁探针已就绪但按用户决定延期，产品路径保持 unavailable。Agent Gateway 仍等待文件授权引用、任务生命周期及可信覆盖/删除确认设计。

## OpenSpec 与验证

[Spike Change](../../../../openspec/changes/fi-s1-file-identity-spike/proposal.md)。只验证技术路线，不代替产品实施Proposal。

[Windows运行包就绪记录](../../../../openspec/changes/fi-s1-file-identity-spike/windows-readiness.md)不是Windows PASS证据。

产品 Runtime 增量：[fi-s1-macos-file-runtime](../../../../openspec/changes/fi-s1-macos-file-runtime/proposal.md)。
