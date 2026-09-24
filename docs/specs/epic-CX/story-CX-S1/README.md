# CX-S1 原生上下文与浏览扩展验证

Story: CX-S1
Epic: CX
Status: done
OpenSpec: cx-s1-macos-context-spike

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

Windows范围已由Accepted AD-E0-05验证。2026-09-24解除macOS延期，进入限时技术Spike；Proposed AD-CX-03只授权验证`NSWorkspace + AXObserver`事件驱动窗口元数据和Chrome用户级Native Messaging，不授权产品采集、索引、检索、Recording或持久化接线。

Spike统一样本必须输出无正文结构化证据；Accessibility权限不足时明确返回`capability_unavailable`，不得轮询窗口、读取浏览器History数据库或用截图兜底。Google Chrome实连可在当前macOS环境验证；Microsoft Edge未安装，只能记录未验证，不能据此宣称双浏览器macOS完成。

2026-09-24首轮失败定位为测试驱动与AX初始化竞态，修正为外部签名`.app`夹具、common-mode RunLoop Source及单次500ms有界注册重试后连续三轮通过；Google Chrome 153当前用户级Native Messaging三条消息实连和隐私拒绝亦通过。Microsoft Edge未安装，保持未验证且不得由Chrome证据替代。见[验证记录](../../../../openspec/changes/archive/2026-09-24-cx-s1-macos-context-spike/verification-macos.md)。AD-CX-03在macOS原生事件与Google Chrome范围接受；本Story不接产品运行时。

## OpenSpec 与验证

openspec/changes/e0-validate-windows-context/

macOS增量：[cx-s1-macos-context-spike](../../../../openspec/changes/archive/2026-09-24-cx-s1-macos-context-spike/proposal.md)（已归档）。

[Change](../../../../openspec/changes/e0-validate-windows-context/proposal.md)；独立验证在该 Change 内维护，记录存在不代表通过。

[原 Story 正文与历史验证](legacy-record.md)。旧编号仅作追溯，不用于新 PR。
