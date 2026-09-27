# TM-S7 统一执行启动状态

Story: TM-S7  
Epic: TM  
Status: verifying  
OpenSpec: tm-s7-unified-execution-start

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

本 Story 统一 CUA、BUA、Document 与 Command 首次副作用前的任务启动语义。CUA/BUA、macOS Document 与 macOS Command 均已接入同一 `start_execution`；能力未获准、未批准或启动事务未提交时，已登记任务仍保持 `created`。

2026-09-23：已建立跨 Story 的 [Proposed AD-AG-09 计划片段 Gateway 契约](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-09-PLAN-FRAGMENT-GATEWAY-CONTRACT.md)，作为统一启动语义下的可选效率形态；该决定不创建第二启动路径，也未授权实施。

Accepted AD-TM-13 定义共享启动边界。CM-S1 的双平台进程树 Spike、DO-S2 的 FI-S1 文件身份/锁门禁，以及各能力的独立 Proposal 通过前，不实施对应执行入口。

2026-09-21：AD-TM-13 已定案 Application 统一启动边界；`start_execution` 已迁移 CUA/BUA 的 created 与 running 分支，并用 SQLite 回归覆盖两个分支。Application/Adapter 全量测试 39 项 PASS，[独立 Verification Goal](../../../../openspec/changes/tm-s7-unified-execution-start/verification-goal.md) 已建立。Document/Command 仍不开放，完整 Story 不 Archive。

2026-09-22：macOS Browser Gateway补证通过。隔离HOME真实桌面进程中，created任务在首条`browser.execute`前进入`running`，真实ego-lite引用建立并在同一引用上completed。Document/Command与Windows仍保留门禁，完整Story不Archive。

2026-09-23：启动事务已与 AG-S1/EX-S2 计划片段候选对齐，见[架构设计](architecture-design.md)。该对齐不改变现有 `start_execution` 契约，也不授权新增协议字段。

2026-09-27：Document 子范围已随 DO-S2 的 `task.document.execute` 接线进入统一启动事务：副作用前以同一 `start_execution` 写入 `created→running`、步骤、attempt、事件及 Outbox，随后才解析双授权并执行默认另存；macOS 宿主目标用例复验通过。

同日 Command 子范围随 CM-S1 协议 1.30、本机一次性批准和 `task.command.execute` 接入：先确认批准引用，再提交统一启动事务，提交后消费引用并派发真实 macOS Command Adapter。交叉审阅已锁定未批准请求保持 `created` 且不创建 attempt；批准执行、一次消费、unknown/超时不重试及无正文响应均通过回归。Windows仍延期，本 Story保持 `verifying`，不 Archive。

## OpenSpec 与验证

[统一启动 Change](../../../../openspec/changes/tm-s7-unified-execution-start/proposal.md)。该 Change 建立 Application 级共享契约；Document 与 Command 的参数、授权、Gateway 和 Adapter 实现分别由 DO-S2、CM-S1 的独立 Change 承接。当前[独立 Verification Goal](../../../../openspec/changes/tm-s7-unified-execution-start/verification-goal.md)已同步四类能力的 macOS 接线边界。
