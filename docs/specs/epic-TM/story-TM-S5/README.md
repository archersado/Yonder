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

2026-09-25 执行尝试开始历史投影已完成 macOS 子范围：协议 1.25 按原 Start 序号输出不可变 step/attempt/worker/host 身份，1.24 隔离有效；Task Space 只显示 step/attempt 的“已准备”文案，并与最终结果保持不同事件。正式隔离 Tauri 宿主、私有 Unix Socket、原生 UI 和独立 Verification Goal 均通过；Windows 按用户决定暂缓，Change 不 Archive。

历史待定项中的分页、用户确认、清单版本、审计配额、协议版本和迁移门禁已由后续设计、AD-TM-22 与关联 OpenSpec 收敛；取消任务继续按 AD-TM-06 保留数据，不在本 Story 引入自动清理或删除。

2026-09-24 的事件响应预算、缺口检测、历史 Observe、控制、定位与创建来源增量已有独立 macOS 验证；合并后协议扩展至 1.24、SQLite schema 19。AD-TM-22 的产物清单确认继续作为唯一确认语义，早期 AD-TM-14 实现不进入主干。

已完成与 TM-S1 的当前值/历史职责及事务边界设计复核，见 [联合复核](../TM-S1-TM-S5-DESIGN-REVIEW.md)。不代表字段契约已全部定案，也不是实施验证。

已补执行标识、可信作者/状态矩阵和重投递/迟到结果规则，并向 TM-S2/S3/S4 同步约束。2026-09-12 补齐事件最小事实、受控引用、Observe 轮次和双重分页预算，提出 8 KiB 事件/256 KiB 整响应的工程限额；2026-09-24 仅将该限额落实到既有 `task.events` 查询投影，其他产物与详情预算尚未定案。

已补保留/删除范围、去重随任务保留、产物版本和清单分页方案；任务审计不擅用上下文/附件 TTL，未同步/固定内容保持保护。尚未实施清理或删除。

AC11 全量忙碌/未知归 TM-S1/TM-S6，不属于本 Story。复核发现审计闭环首版只会在确认时生成空清单，尚未实现非空清单发布、版本变化和条目失效；因此 TM-S5 除 Windows 外仍有 AC05 的核心、Gateway/Adapter/UI 接线与完整 Story 验证门禁，不得以既有 macOS 子范围 PASS 提前 Archive。

2026-09-25 `tm-s5-artifact-manifest-core` 已通过独立验证：可信内部发布、不可变版本、引用可用性、同事务事件/Outbox 与受授权内部分页均已实现；确认会绑定当时最新清单，后续版本不改写旧确认。该 Change 不开放 Agent 发布或 UI 条目列表，只关闭 AC05 的核心存储/Application 子范围，完整 Story 继续保持 verifying。

2026-09-23 建立 [AD-TM-22](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-22-AUDIT-COMPLETENESS-AND-QUOTA.md)，收敛用户结果确认、不可变产物清单、审计容量门禁、协议 1.20 与 schema 18 迁移；该子范围进入 design-review，完整 Story 保持 implementing。

2026-09-25 完整审计闭环的实现与协议/存储/应用/前端回归、macOS 原生 UI 验证均通过，见 [独立 Verification Goal](../../../../openspec/changes/tm-s5-audit-completeness/verification-goal.md)。验证中修复 `task.step.get` 未返回审计投影的问题；Windows 仍暂缓，Story 保持 verifying，不 Archive。

产物清单核心版本化增量：[tm-s5-artifact-manifest-core](../../../../openspec/changes/tm-s5-artifact-manifest-core/proposal.md)。

2026-09-25 产物清单核心版本化 [独立 Verification Goal](../../../../openspec/changes/tm-s5-artifact-manifest-core/verification-goal.md) PASS：全仓 109 项 Rust 测试、75 项 OpenSpec、33 项 Python 测试及仓库门禁通过。Gateway/具体能力 Adapter/UI、交回与 Recording 审计、Windows 验证仍待完成，不 Archive。

## OpenSpec 与验证

三份设计和相关 ADR 明确后创建独立 Change。验证必须包含多步骤历史、终态/用户确认分离、失效产物、审计权限与回滚；真实 UI/Driver 证据不能由合成事件替代。

2026-09-24 建立 [tm-s5-result-confirmation](../../../../openspec/changes/tm-s5-result-confirmation/proposal.md)：只关闭 TM5-AC04 的本机终态结果审阅确认，不声称完成产物或保留策略。

2026-09-24 macOS 独立验证 PASS，见 [Verification Goal](../../../../openspec/changes/tm-s5-result-confirmation/verification-goal.md)：协议/SQLite/Outbox/幂等和原生待确认入口均通过，真实任务未被代替用户确认。Windows 按用户决定暂缓，完整 Story 保持 implementing。

2026-09-24 依据 Accepted AD-TM-15 建立 [tm-s5-event-response-budget](../../../../openspec/changes/tm-s5-event-response-budget/proposal.md)，推进 TM5-AC08 的编码字节上限及稳定续读；完整 Story 的产物与保留门禁继续开放。

2026-09-24 事件响应预算独立 [Verification Goal](../../../../openspec/changes/tm-s5-event-response-budget/verification-goal.md) macOS 本机 PASS：合成临界值、全仓测试、规格与架构门禁通过。Windows 原生证据仍暂缓；完整 Story 不 Archive/Done。

2026-09-24 依据 Accepted AD-TM-16 建立 [tm-s5-timeline-gap-detection](../../../../openspec/changes/tm-s5-timeline-gap-detection/proposal.md)：当前未裁剪历史若有首项、中间或尾部缺失，查询明确报错，不再伪装为完整页；完整产物与清理门禁不变。

2026-09-24 历史序号缺口检测独立 [Verification Goal](../../../../openspec/changes/tm-s5-timeline-gap-detection/verification-goal.md) macOS 本机 PASS：SQLite 故障注入、全仓 90 项测试与界面局部错误夹具通过。Windows 暂缓，完整 Story 仍为 implementing。

2026-09-24 依据 Accepted AD-TM-17 建立 [tm-s5-historical-observation](../../../../openspec/changes/tm-s5-historical-observation/proposal.md)：只读投影已持久化的可信观察事件，协议 1.21 与旧版隔离；不推断未发生的观察或产物。

2026-09-24 历史 Observe 增量已有 [独立 Verification Goal](../../../../openspec/changes/tm-s5-historical-observation/verification-goal.md)：SQLite/Gateway、全仓测试、macOS 原生 WebKit 隔离夹具及正式 Tauri 宿主原生可见 E2E 均通过。Windows 暂缓，完整 Story 仍为 implementing。

2026-09-24 依据 Accepted AD-TM-18 实施 [tm-s5-historical-control](../../../../openspec/changes/tm-s5-historical-control/proposal.md)：协议 1.22 将已提交控制请求和步骤边界停止按原事件序号只读投影；[独立 Verification Goal](../../../../openspec/changes/tm-s5-historical-control/verification-goal.md) 的 macOS Gateway 与正式 Tauri 宿主可见 E2E 通过。Windows 暂缓，定位/交回/录制和完整 TM-S5 仍不 Archive。

2026-09-24 依据 Accepted AD-TM-19 实施 [tm-s5-historical-focus](../../../../openspec/changes/tm-s5-historical-focus/proposal.md)：schema 19 与协议 1.23 将已提交定位中、成功或失败按原事件序号保存并只读投影；[独立 Verification Goal](../../../../openspec/changes/tm-s5-historical-focus/verification-goal.md) 的事务回滚、零回填迁移、macOS Gateway 与正式 Tauri 宿主 E2E 通过。Windows 暂缓，完整 TM-S5 的产物、保留、录制和交回仍不 Archive。

2026-09-24 依据 Accepted AD-TM-20 实施 [tm-s5-historical-creation](../../../../openspec/changes/tm-s5-historical-creation/proposal.md)：协议 1.24 将创建事务已保存的来源与归属 Agent 按 `#1` 事件只读投影；[独立 Verification Goal](../../../../openspec/changes/tm-s5-historical-creation/verification-goal.md) 的旧版隔离、越权拒绝、零回填、macOS Gateway 与正式 Tauri 宿主 E2E 通过。Windows 暂缓，完整 TM-S5 仍不 Archive。

首批删除按Accepted AD-TM-05，openspec/changes/tm-s5-terminal-delete/；完整时间线/产物/确认原需求仍联审，不以删除关闭全部Story。

用户取消清理语义，tm-s5-terminal-delete已withdrawn，不Done/Archive；数据保留变更由TM-S3/AD-TM-06承接。

2026-09-18 建立首批只读时间线增量 [tm-s5-readonly-timeline](../../../../openspec/changes/tm-s5-readonly-timeline/proposal.md)：仅在现有 Task Space 详情展示正式 `task.events` 的状态、步骤声明和执行结果，不新增协议、存储或事件类型。完整产物、确认、配额及清理设计门禁保持。

2026-09-18 既有事件分页增量 [tm-s5-timeline-pagination](../../../../openspec/changes/tm-s5-timeline-pagination/proposal.md) macOS 子范围 PASS：Task Space 每页读取 20 条，以最后实际事件序号继续；失败保留并可重试。正式 54 条历史任务已验证 `#1..#20` 追加到 `#40`。Windows、产物、确认及新 payload 字节预算仍保留门禁。

2026-09-18 首批只读时间线 macOS 子范围 PASS：ego-browser 与正式桌宠验证状态/声明/结果展示、局部失败和键盘入口；真实 Agent 步骤进入时间线。验证任务随后取消并保留数据。Windows及完整 TM-S5 继续保留门禁。
