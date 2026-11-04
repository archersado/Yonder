# AG-S6 本地 Agent 注册与撤权

Story: AG-S6
Epic: AG
Status: verifying
OpenSpec: ag-s6-local-agent-registration

关联 [Accepted AD-AG-08](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-08-LOCAL-AGENT-REGISTRATION.md)。本 Story 只定义 MVP 之后能力，不并入 AG-S2 当前实现。

设计：
- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

关联实施目录：[ag-s6-local-agent-registration](../../../../openspec/changes/ag-s6-local-agent-registration/proposal.md)，路径`openspec/changes/ag-s6-local-agent-registration/`。

2026-09-23：macOS Verification Goal 已通过，覆盖真实 SQLite 17 迁移、Agent 管理 UI、UDS权限拒绝、活动会话撤权断开、任务保留与取消、重启持久化。

2026-11-01：Windows 原生 Verification Goal 已通过，覆盖 Agent 管理 UI 登记/禁用/启用/撤权、Named Pipe `-32003` 权限拒绝、活动 `user_input` 会话撤权断开、任务正式取消及重启持久化；证据见 [Windows验证报告](../../../../openspec/changes/ag-s6-local-agent-registration/verification-windows-20261101.md)。测试残留均已收口。正式安装签名、跨用户 ACL 与生产认证仍属于 AG-S1 门禁。
