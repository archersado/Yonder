# Verification Goal：TM-S9 活动任务列表 Runtime 覆盖

日期：2026-10-12
平台：macOS 26.5.1 arm64
Result: PASS（Windows Deferred）

使用正式 macOS `Yonda.app → MCP stdio → Agent Gateway` 创建并进入活动 Runtime 的无用户数据任务。在 SQLite projector 尚未覆盖最新终态的窗口读取 `task.get`、`task.events` 与 `task.list`，三者必须返回同一 sequence/status；`running_only` 不得保留已取消任务，`include_finished=false` 同样不得泄漏终态。

自动化覆盖页内覆盖、旧 Runtime 不倒退 Store、新任务分页游标稳定、Runtime 不可用时历史列表不变。Windows按用户决定暂缓，不外推。

## 自动验证

- `cargo test --offline --locked -p yonder-application`：83/83 PASS；覆盖页内 Runtime 覆盖、取消后筛选和分页游标保持。
- `cargo test --offline --locked -p yonder-adapters -- --test-threads=1`：86/86 PASS。
- `cargo test --offline --locked -p yonder-desktop --lib -- --test-threads=1`：33/33 PASS。
- `openspec validate tm-s9-task-list-runtime-overlay --strict`、`python3 scripts/check_architecture.py` 与 `git diff --check`：PASS。

## 正式产品链路验证

从 `dev@b976029` 重建并重启正式 debug `Yonda.app`，只通过安装包 MCP stdio、Agent Gateway 与 Application 创建任务 `task_e03f0f775dc1c3107c50a250029db628`。无用户数据 CUA 探针形成活动 Runtime 后，经 `task.control(cancel)` 在序号 5 停止。

- `task.get`：`cancelled@5`。
- `task.events`：连续返回 1..5，序号 5 为 `control_event(cancel, stopped)` 与 `cancelled`。
- `task.list(include_finished=true)`：同一任务为 `cancelled@5`。
- `task.list(running_only=true)`：不包含该任务。
- `task.list(include_finished=false)`：不包含该任务。

结构化证据见 `apps/desktop/evidence/tm-s9-task-list-runtime-overlay-macos-20261012/result.json`。验证未直连 Socket、SQLite、Runtime 或 Driver，也未记录截图、完整命令输出或 Agent Payload。

## 平台范围

本 Goal 只声明 macOS 正式链路通过。Windows 继续按用户决定 Deferred，不使用 macOS 结果替代双平台证据。
