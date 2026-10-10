# AG-S9 Agent 回复回传与圈选会话卡

Story: AG-S9  
Epic: AG  
Status: implementing  
OpenSpec: ag-s9-agent-reply-presentation

关联变更：[ag-s9-agent-reply-presentation](../../../../openspec/changes/ag-s9-agent-reply-presentation/proposal.md)。

## 目标

用户从 Yonder 圈选并向当前 Agent 提问后，不必切换到 Agent 客户端；Yonder 在原确认卡中显示等待、流式回答和明确终态。

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)
- [Accepted AD-AG-11](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-11-AGENT-REPLY-PRESENTATION.md)

## 范围

- 首批只覆盖 macOS Codex CLI 托管连接器与圈选卡。
- 回复为短生命运行时数据，不是任务状态、个人上下文或聊天历史。
- Windows 继续 Deferred；ChatGPT Desktop 继续 Unsupported。
