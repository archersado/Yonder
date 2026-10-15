# AG-S13 Agent 输入会话单活抢占

Story: AG-S13
Epic: AG
Status: implementing
OpenSpec: ag-s13-agent-session-target-selection

关联变更：[ag-s13-agent-session-target-selection](../../../../openspec/changes/ag-s13-agent-session-target-selection/proposal.md)。

## 目标

Yonder 本地输入通道只保留一个 AgentSession。新的已认证会话连接时抢占旧会话，并立即成为圈选、语音和回复回传的唯一目标；产品不要求用户理解或选择 Codex 会话。

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)
- [Accepted AD-AG-11 增量](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-11-AGENT-REPLY-PRESENTATION.md)

## 来源与修订

2026-10-14 首轮方案允许多个在线 AgentSession 并要求用户选择目标；实现后真实验证证明该交互暴露了不应存在的内部连接复杂度。

2026-10-15 主人明确连接应为单活抢占。此前会话列表、显式选择和 `session_ref` UI 方案全部作废；本 Story 返回实施阶段，以后连抢占和旧连接安全退出完成闭环。

同日已合入回复订阅修复 `f29f3e1`：共享 daemon 中即使 thread 已处于 loaded，当前 Connector 仍须通过同一 proxy 执行 `thread/resume(excludeTurns=true)`，取得同 thread 回执后才向 Yonder 注册，避免输入可投递但本客户端收不到回复通知。独立复验覆盖 Connector 18 项与 Desktop AgentInputHub 11 项回归，全部 PASS；真实文字回传、TTS 朗读/停止及双连接抢占仍由正式 macOS GUI Verification Goal 验证，未据隔离测试宣称 Story 完成。
