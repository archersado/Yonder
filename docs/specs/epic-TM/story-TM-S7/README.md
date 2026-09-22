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

本 Story 统一 CUA、BUA、Document 与 Command 首次副作用前的任务启动语义。CUA/BUA 已有 `start_attempt` 路径；Document 和 Command 尚未接入 Gateway 或任务执行用例，不能把已登记的 `created` 任务显示为执行中。

Accepted AD-TM-13 定义共享启动边界。CM-S1 的双平台进程树 Spike、DO-S2 的 FI-S1 文件身份/锁门禁，以及各能力的独立 Proposal 通过前，不实施对应执行入口。

2026-09-21：AD-TM-13 已定案 Application 统一启动边界；`start_execution` 已迁移 CUA/BUA 的 created 与 running 分支，并用 SQLite 回归覆盖两个分支。Application/Adapter 全量测试 39 项 PASS，[独立 Verification Goal](../../../../openspec/changes/tm-s7-unified-execution-start/verification-goal.md) 已建立。Document/Command 仍不开放，完整 Story 不 Archive。

2026-09-22：macOS Browser Gateway补证通过。隔离HOME真实桌面进程中，created任务在首条`browser.execute`前进入`running`，真实ego-lite引用建立并在同一引用上completed。Document/Command与Windows仍保留门禁，完整Story不Archive。

2026-09-23：启动事务已与 AG-S1/EX-S2 计划片段候选对齐，见[架构设计](architecture-design.md)。该对齐不改变现有 `start_execution` 契约，也不授权新增协议字段。

## OpenSpec 与验证

[统一启动 Change](../../../../openspec/changes/tm-s7-unified-execution-start/proposal.md)。该 Change 只建立 Application 级共享契约、现有 CUA/BUA 回归与 Document/Command 接线门禁；不提前实现尚未获准的 Command 或文件写入能力。
