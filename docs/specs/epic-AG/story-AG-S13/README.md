# AG-S13 Agent 输入会话目标选择

Story: AG-S13  
Epic: AG  
Status: verifying  
OpenSpec: ag-s13-agent-session-target-selection

关联变更：[ag-s13-agent-session-target-selection](../../../../openspec/changes/ag-s13-agent-session-target-selection/proposal.md)。

## 目标

当同一 Yonder 进程连接多个可接收输入的 AgentSession 时，用户能在 Agent 管理中选择本轮语音/圈选输入的接收会话；单会话仍自动选择，不要求额外操作。

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)
- [Accepted AD-AG-11 增量](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-11-AGENT-REPLY-PRESENTATION.md)

## 来源与当前阻断

2026-10-14 真实 VI-S3 验证中，Yonder 已连接多个 Codex CLI 会话，圈选卡正确拒绝猜测并提示“请选择接收输入的 Agent”，但产品没有任何会话选择入口，导致用户无法解除阻断。本 Story 只补齐该运行时选择闭环，不建立聊天列表、历史会话或持久化路由。

2026-10-14 已完成 Hub 状态机、Agent 管理会话列表/选择以及圈选卡直达入口并 ff-only 合入 `dev`。独立验证已完成自动化、正式 macOS 构建和真实连接态检查；多会话选择后的真实回复朗读/停止仍待当前客户端重连后复跑，因此保持 `verifying`。
