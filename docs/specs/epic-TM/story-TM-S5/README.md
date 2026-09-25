# TM-S5 任务时间线、产物与审计

Story: TM-S5
Epic: TM
Status: verifying
OpenSpec: tm-s5-audit-completeness

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

2026-09-25 当前实现已覆盖协议 1.20、SQLite schema 18、本机用户确认、不可变产物清单与审计容量门禁，并通过协议、存储、应用、前端回归及 macOS 原生 UI 验证；Windows/完整 Story 验证仍待完成。与 TM-S1 同期审阅历史事件与快照事务，AG 提供授权，DS-S2 提供结果/时间线界面；TM-S2/S3/S4 提供实际执行和控制事实。录制原始时间线由 RC 管理，不复制进任务审计。

历史待定项中的分页、用户确认、清单版本、审计配额、协议版本和迁移门禁已由后续设计、AD-TM-22 与关联 OpenSpec 收敛；取消任务继续按 AD-TM-06 保留数据，不在本 Story 引入自动清理或删除。

已完成与 TM-S1 的当前值/历史职责及事务边界设计复核，见 [联合复核](../TM-S1-TM-S5-DESIGN-REVIEW.md)。不代表字段契约已全部定案，也不是实施验证。

已补执行标识、可信作者/状态矩阵和重投递/迟到结果规则，并向 TM-S2/S3/S4 同步约束。2026-09-12 补齐事件最小事实、受控引用、Observe 轮次和双重分页预算，提出 8 KiB 事件/256 KiB 整响应的工程限额；文档边界检查通过，不代表序列化实现已验证。

已补保留/删除范围、去重随任务保留、产物版本和清单分页方案；任务审计不擅用上下文/附件 TTL，未同步/固定内容保持保护。尚未实施清理或删除。

AC11 全量忙碌/未知归 TM-S1/TM-S6，不属于本 Story。TM-S5 当前只剩 Windows 与完整 Story 验证门禁；不得以 macOS 子范围 PASS 提前 Archive。

2026-09-23 建立 [AD-TM-22](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-22-AUDIT-COMPLETENESS-AND-QUOTA.md)，收敛用户结果确认、不可变产物清单、审计容量门禁、协议 1.20 与 schema 18 迁移；该子范围进入 design-review，完整 Story 保持 implementing。

2026-09-25 完整审计闭环的实现与协议/存储/应用/前端回归、macOS 原生 UI 验证均通过，见 [独立 Verification Goal](../../../../openspec/changes/tm-s5-audit-completeness/verification-goal.md)。验证中修复 `task.step.get` 未返回审计投影的问题；Windows 仍暂缓，Story 保持 verifying，不 Archive。

## OpenSpec 与验证

三份设计和相关 ADR 明确后创建独立 Change。验证必须包含多步骤历史、终态/用户确认分离、失效产物、审计权限与回滚；真实 UI/Driver 证据不能由合成事件替代。

首批删除按Accepted AD-TM-05，openspec/changes/tm-s5-terminal-delete/；完整时间线/产物/确认原需求仍联审，不以删除关闭全部Story。

用户取消清理语义，tm-s5-terminal-delete已withdrawn，不Done/Archive；数据保留变更由TM-S3/AD-TM-06承接。

2026-09-18 建立首批只读时间线增量 [tm-s5-readonly-timeline](../../../../openspec/changes/tm-s5-readonly-timeline/proposal.md)：仅在现有 Task Space 详情展示正式 `task.events` 的状态、步骤声明和执行结果，不新增协议、存储或事件类型。完整产物、确认、配额及清理设计门禁保持。

2026-09-18 既有事件分页增量 [tm-s5-timeline-pagination](../../../../openspec/changes/tm-s5-timeline-pagination/proposal.md) macOS 子范围 PASS：Task Space 每页读取 20 条，以最后实际事件序号继续；失败保留并可重试。正式 54 条历史任务已验证 `#1..#20` 追加到 `#40`。Windows、产物、确认及新 payload 字节预算仍保留门禁。

2026-09-18 首批只读时间线 macOS 子范围 PASS：ego-browser 与正式桌宠验证状态/声明/结果展示、局部失败和键盘入口；真实 Agent 步骤进入时间线。验证任务随后取消并保留数据。Windows及完整 TM-S5 继续保留门禁。
