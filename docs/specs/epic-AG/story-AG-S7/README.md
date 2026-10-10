# AG-S7 Codex 会话连接生命周期 Spike

Story: AG-S7  
Epic: AG  
Status: design-review  
OpenSpec: ag-s7-codex-connector-lifecycle-spike  
Timebox: 2026-10-10 ～ 2026-10-16

## 目标

验证 Codex CLI、ChatGPT Desktop 内 Codex 会话与 Yonder 之间可受支持的当前会话绑定、进程托管和断线恢复路线，形成可实施的终局 ADR。目标产品体验是用户只启动 Yonder 与 Codex，不再手工维护 App Server、thread ID、Socket 或 `agent-bridge`。

本 Story 是技术路线 Spike，不授权产品实现。路线未通过双平台证据与 ADR 审阅前，不得实现自动发现最近会话、启动第二 Agent 或读取 Codex 内部数据库的替代方案。

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)
- [Proposed AD-AG-10](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-10-CODEX-CONNECTOR-LIFECYCLE.md)

## 当前结论与门禁

- 已知可行基线：Codex CLI 0.156.1 共享 App Server、显式 `thread_id` 与 WebSocket-over-UDS，已由 AG-S5 验证。
- 已知缺口：Yonder 重启后薄桥退出且无人重新拉起；普通 MCP 只能恢复任务方向，不能恢复慢脑回注。
- 未验证：ChatGPT Desktop 私有 `stdio://` App Server 是否存在受支持的当前会话连接器或生命周期 hook；不得从进程、最近会话或本地数据库猜测。
- Windows 对照样本按既有决定保持待办；缺少 Windows 证据时 Spike 不得宣称跨平台完成，AD-AG-10 不得转 Accepted。

关联变更：[ag-s7-codex-connector-lifecycle-spike](../../../../openspec/changes/ag-s7-codex-connector-lifecycle-spike/proposal.md)。
