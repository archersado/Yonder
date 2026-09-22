# TM-S1 任务状态与持久化用例

Story: TM-S1
Epic: TM
Status: implementing
OpenSpec: oct-s1-task-status

2026-09-18 增加 `tm-s1-agent-wait-for-user` 子范围：按 Accepted AD-TM-10 将安全步骤边界的 Agent 等待请求接入协议 1.17，持久化等待原因并驱动既有桌宠状态；不实现 Resume。

该 macOS 子范围已 PASS：协议/SQLite/准入合约、Yonder MCP、Task Space 时间线及正式库 schema 13→14 迁移均验证通过。Windows、Resume 与完整 TM-S1 门禁保留。

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

三份设计已按原始产品与架构扩展到 16 条验收条件；新增授权引用、资源失效、等待/恢复与历史一致性。TM-S5 承担时间线/产物/审计的专门查询与验收，二者共同审阅写入事务。仍为 design-review，不继续追加功能代码。

## 逐步拆解清单

- [x] 明确 TM/AG/DS/执行资源的职责与非目标
- [x] 定义创建、归属、迁移、分页、事件、恢复的验收场景
- [x] 明确状态文案、错误、加载/空态和未知信息的表现
- [x] 补齐原需求的信息归属和跨模块责任，新增 TM-S5 时间线/产物/审计 Story 三份设计
- [x] 当前值/历史事实、同事务与外部产物边界完成设计复核，见 [联合复核](../TM-S1-TM-S5-DESIGN-REVIEW.md)
- [x] 定义 task/step/attempt/request/sequence 职责、作者/状态允许矩阵和重投递/迟到结果规则；仍待 payload 与去重寿命定稿
- [x] 列出历史事件最小事实、观察轮次与受控引用，提出单事件/整页字节预算并完成文档边界走查；尚无实现测试
- [x] 补齐保留/删除/去重寿命及产物版本清单方案；区分源文档期限与工程建议，待整体 ADR 和跨模块审阅
- [x] AC10：确定名称、来源、步骤、观察/意图的字段与限额，更新主体、脱敏与持久化规则
- [x] AC11：确定全量忙碌/未知汇总及错误语义，不依赖 UI 分页
- [x] 先更新涉及的 ADR，再同步三份设计和跨模块接口
- [x] 设计审阅通过后明确新 Proposal 范围；旧 Change 仅保留历史承接，不扩展跨模块代码
- [ ] 建立实现后的独立验证目标，不把文档检查视为产品验收

AC10/AC11 设计已定案。2026-09-22 [`tm-s1-task-presentation-metadata`](../../../../openspec/changes/archive/2026-09-22-tm-s1-task-presentation-metadata/proposal.md) 已完成协议 1.19、SQLite schema 15、分作者用例与 Gateway 查询接线，全量 Rust 测试、生成物检查和架构门禁通过；macOS 原生展示与[独立 Verification Goal](../../../../openspec/changes/archive/2026-09-22-tm-s1-task-presentation-metadata/verification-goal.md)已通过，Change 已归档，Windows 按用户要求暂缓。历史保留和产物版本仍归 AD-TM-01 的 Proposed 部分。真实认证归 AG-S1、资源派发归 TM-S2、可见总览归 DS-S2；密钥与环绕菜单仍暂停。
原生验证证据：`apps/desktop/evidence/tm-s1-presentation-20260922/result.json` 与 `tm-s1-native.png`。

## OpenSpec 与验证

openspec/changes/oct-s1-task-status/

[Change](../../../../openspec/changes/oct-s1-task-status/proposal.md)；独立验证在该 Change 内维护，记录存在不代表通过。

2026-09-18：旧跨模块Change `oct-s1-task-status`已按AD-DEV-01冻结，保留已实施核心与验证历史。认证/Gateway、执行、文件、桌面、凭据与平台验证已迁入对应模块Story，不再向该Change追加实施。冻结不等于完整TM-S1验收或Archive。

[原 Story 正文与历史验证](legacy-record.md)。旧编号仅作追溯，不用于新 PR。

AC11 的数据库只读子项已独立拆为 [TM-S6](../story-TM-S6/README.md)，其通过不代表本 Story 或完整隐藏门禁通过。

2026-09-14 Agent名称子范围已实施并通过31项核心回归和macOS原生stdio Agent菜单验证，见openspec/changes/ag-s2-local-task-registration/verification-agent-names.md。旧数据备份迁移保留，新测试任务取消后保留；完整Story状态不变，Windows暂缓。
