# TM-S1 任务展示元数据独立 Verification Goal

日期：2026-09-22。关联 Story TM-S1、Partially Accepted AD-TM-01（AC10 子范围）与 `tm-s1-task-presentation-metadata`。Windows 按用户要求暂缓。

## 结果

PASS（macOS 子范围，协议/SQLite/Gateway/原生展示）。工作区锁定测试、协议生成检查、架构门禁、JavaScript 语法检查和 `git diff --check` 均通过。

- 协议 1.19 及 Rust 派生 JSON Schema/TypeScript 同步完成；旧协议会话剥离新增展示字段。
- SQLite schema 14→15 迁移先备份旧库，历史任务映射 `source=legacy`，其余展示字段为 `null`；迁移后手工降级重试幂等请求返回 `IdempotencyConflict`，不静默伪造成功。
- `source/current_step/observation/next_intent` 按作者写入；接受写入同事务更新当前值、递增任务 sequence、追加事件并写 Outbox，失败整体回滚。
- `task.get` 与 `task.step.get` 返回完整快照，`task.list` 仅返回有界摘要；详情新增“来源 / 当前步骤 / 观察摘要 / 下一步意图”，全部按纯文本渲染。
- macOS 原生验证通过，证据见 [result.json](../../../apps/desktop/evidence/tm-s1-presentation-20260922/result.json) 与 [tm-s1-native.png](../../../apps/desktop/evidence/tm-s1-presentation-20260922/tm-s1-native.png)。

## 完成边界

本 Goal 关闭 `tm-s1-task-presentation-metadata` 的 AC10 子范围，可以归档该 Change。Windows 证据暂缓不阻塞本子范围；历史保留、产物版本、Resume 和完整 TM-S1 其余验收仍由后续 Story/Change 承接，完整 TM-S1 不因本记录 Done。
