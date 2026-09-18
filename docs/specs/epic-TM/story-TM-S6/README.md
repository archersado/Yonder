# TM-S6 全量运行状态查询

Story: TM-S6
Epic: TM
Status: verifying
OpenSpec: tm-s6-running-state-query

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

2026-09-12 完成独立核心范围工程审阅，依据 AD-TM-02，从 TM-S1 AC11 拆出数据库全量查询。TM-S1 整体仍待审；不把 NoRunningTask 当作收起许可。无 schema、传输或桌面改动，不依赖未定案元数据。当前工作区已有多 Story 未提交改动，沿用现有分支保留现场，未创建 PR；合并前必须隔离本 Story 变更。

## OpenSpec 与验证

openspec/changes/tm-s6-running-state-query/

[Proposal](../../../../openspec/changes/tm-s6-running-state-query/proposal.md)。实施后独立验证，不追认桌面或 Windows 原生验收。

[独立验证记录](../../../../openspec/changes/tm-s6-running-state-query/verification-goal.md)：本地核心通过，Windows 与 PR 审阅未完成，未 Archive。

2026-09-12：同一未归档 Change 扩展 AC5–8，汇总数据库与真实准入占用；AD-TM-02 与三份设计先行补齐，重新进入实施验证。

[执行占用汇总独立验证](../../../../openspec/changes/tm-s6-running-state-query/verification-activity.md)：18 项核心回归通过；数据库与准入占用的只读汇总完成，桌面协调和双平台闭环尚未完成。

本轮在同一 Change 补充 AC9–11 收起预约，按 AD-TM-02 先更新设计再实施；不解除 DS-S1 门禁。

[收起预约独立验证](../../../../openspec/changes/tm-s6-running-state-query/verification-rest-reservation.md)：20 项核心测试通过；同一准入实例内竞争已覆盖，原生接线仍待 DS-S1 门禁。
