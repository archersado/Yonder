# TM-S7 Document 统一启动复验

日期：2026-09-27。关联 Story：TM-S7；Change：`tm-s7-unified-execution-start`；Document 执行实现：DO-S2 的 `task.document.execute`。

## 结论

PASS（macOS Document 子范围）。`execute_agent_save_as` 在解析或消费文件授权、调用 Document Port 及产生任何文件副作用之前，调用 Application 唯一的 `start_execution`。该事务为 `created` 任务提交 `created→running`、步骤、prepared attempt、sequence、事件与 Outbox；事务失败不会派发文档操作。执行后无论成功、可观察失败或未知，都复用 attempt 结果写入路径，不自动重试。

## 本机证据

| 命令 | 结果 | 覆盖范围 |
| --- | --- | --- |
| `cargo test -p yonder-application` | PASS，36/36 | 统一启动用例、资源准入和 Gateway 边界回归 |
| `cargo test -p yonder-desktop tests::document_execute_uses_dual_grants_and_default_save_as_without_paths -- --exact` | PASS，1/1 | macOS TaskHost 中双授权、首次启动、默认另存、源文件保留及响应不泄露路径 |
| `python3 scripts/check_architecture.py` | PASS | 架构与 Story/OpenSpec 关联 |
| `openspec validate tm-s7-unified-execution-start --strict` | PASS | Change 结构与规格一致性 |

## 边界

本记录不证明 Command 接线、Shell、风险确认、Windows 或完整 Story。Accepted AD-CM-01 仅授权 macOS 内部 Runtime，明确禁止在风险确认协议定案前开放 Agent Gateway、CLI/MCP 或 Jev Command 候选；因此 Command 保持待办，不能以本 Document 结果推断可用。
