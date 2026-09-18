当前归属 Story：CU-S1；规划：`docs/specs/epic-CU/story-CU-S1/README.md`。旧编号保留历史追溯。

状态：Frozen（选型历史，不再新增实施）。唯一结论trycua 0.25.0已进入正式`apps/desktop/cua`依赖清单和`crates/adapters`产品链路；后续宿主权限、派发、Observe、停止、工作定位与平台验证由CU-S2/TM-S3及其独立Change承接。本Change保留原始对照和失败证据，不Archive。

# 提案：对照 CUA Driver

## 为什么

CUA Driver 决定跨平台输入、观察与故障恢复能力，必须在产品 Story 前实测定案。

## 变更

建立无 Agent 的统一黑盒 Harness，对照 Qwen CUA SDK 0.20.5 与 trycua CUA Driver 0.25.0。旧 Qwen open-computer-use 仅记录为历史基线。

## 影响

- Story：E0-S2
- 架构影响：architecture-change（候选尚未定案）
- 决策输出：AD-E0-02
- 产品代码影响：无，仅 Spike
- 协议影响：形成 Yonder CUA Driver Port 的最小能力要求
- 迁移影响：无

macOS接管只读停止补充Spike关联CU-S1及docs/specs/epic-CU/story-CU-S1/，按TAKEOVER-STOP-PLAN定期限/统一样本/门禁。仅复用trycua，不授权CU-S2产品输入或Recording，不更改既有Windows选择与历史证据。

2026-09-14补充FOCUS-SDK原生工作定位Spike，关联TM-S3/CU-S2三份设计与AD-TM-07/AD-CU-02；期限及样本见TAKEOVER-STOP-PLAN.md。只验证SDK契约与隔离AX/AppKit精确前置/恢复/失效拒绝，不授权产品任务接管，不引入附加App；独立verification-focus-macos.md，Windows暂缓。

2026-09-14：沿AD-CU-03工作定位Spike扩展身份复核，设计依据WORK-IDENTITY-PLAN与TM-S3/CU-S2三份设计/Proposed AD-TM-08。只扩展隔离夹具及原生探针保留模式，固定SDK0.25.0无截图Observe，无产品协议/持久化迁移。独立verification-work-identity-macos.md已PASS macOS隔离子范围；Windows、正式宿主权限、多Space/显示器、步骤停止及Recording门禁保留。
