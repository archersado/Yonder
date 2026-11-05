# Proposal：agent-desktop v2 交付门禁 Spike

状态：Proposed。期限：2026-11-05 至 2026-11-11。关联 [CU-S5](../../../docs/specs/epic-CU/story-CU-S5/README.md) 与 [AD-CU-08](../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-08-AGENT-DESKTOP-V2-DELIVERY-PROOF.md)。

## Why

CU-S4 证实旧限定引用可以绑定同窗替代按钮；Windows 输入低级钩子不能证明持续存活。当前候选不能安全开放动作。需要在原五工作日期限内，用隔离 v2 源码与双平台样本证明两个门禁或淘汰候选。

## What Changes

- 新建独立 v2 源码/补丁/manifest/样本，不改 v1 输入和证据。
- 固定最终交付路径和输入观察来源的源码清单，实施只读 fixture 与失效样本。
- 两门禁经双平台原生证据与独立验证后，才另行更新 ADR 讨论隔离 fixture 真实动作；当前动作继续拒绝。
- 到期输出可复核的 Accepted/Rejected 建议，不修改产品 Gateway、Adapter、协议、数据库或 UI。

## Architecture Impact

`architecture-change`，但限于 Spike 候选实现。模块 CU；依赖方向、产品协议和迁移无变化；不形成第二产品 Driver 栈。
