# AG-S11 Agent 回复 Markdown 渲染

Story: AG-S11
Epic: AG
Status: implementing
OpenSpec: ag-s11-markdown-agent-reply

关联变更：[ag-s11-markdown-agent-reply](../../../../openspec/changes/ag-s11-markdown-agent-reply/proposal.md)。

## 目标

Yonder 圈选回复卡把 Agent 返回的 Markdown 渲染为可读的标题、段落、列表、引用、代码与行内强调，同时保持回复的内存、安全和大小边界。

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 范围

- 首批只覆盖 macOS 圈选回复卡。
- 不改变 AgentSession、Gateway、SQLite、事件、Outbox 或日志。
- Windows 与其他回复载体继续 Deferred。
