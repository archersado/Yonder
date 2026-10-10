# AG-S9 独立 macOS Verification Goal

Story: AG-S9
日期：2026-10-10
平台：macOS
Result: PASS

## 验证目标

独立验证圈选输入经既有 Agent Gateway 进入当前 Codex 会话后，正式 App Server 通知能够按 `input_id + session_id + reply_id + sequence` 回到 Yonder，并在原圈选卡展示等待、增量正文和唯一终态；回复内容不进入任务持久化、事件、Outbox 或日志。

## 自动化与合约证据

- `cargo test --offline --locked -p yonder-protocol -p yonder-application -p yonder-cli -p yonder-desktop`：164 项通过，0 失败。
- Protocol 20 项覆盖 1.50 能力门禁、严格通知结构、16 KiB 单段限制及生成物一致性。
- Connector 19 项覆盖正式 `item/started`、`item/agentMessage/delta`、`item/completed`、`turn/completed` 映射、turn 关联、UTF-8 分块、重连与附件同 turn 投递。
- Desktop 43 项覆盖回复请求协商、跨 session 拒绝、严格序号、唯一终态、断连转 interrupted，以及既有 Gateway/任务回归。
- Application 82 项全量回归通过；普通 voice、replan 与任务请求没有被回复槽位持久化或替代。
- `cargo run --offline --locked -p yonder-protocol --example generate -- --check`、`node --check apps/desktop/ui/region-preview.js`、`openspec validate ag-s9-agent-reply-presentation --strict`、`python3 scripts/check_architecture.py` 与 `git diff --check` 均通过。

## 原生运行证据

- 从已快进合并的 `dev` 构建 `yonder-desktop` 与 `yonder-cli` 成功。
- 新二进制已装入本地 `Yonda.app` 调试包并启动；托管 `yonder codex-hook` 随后恢复，同期错误日志为空。
- 独立 Computer Use 检查器在验证时报告 Sky native pipe 启动失败/超时；该失败不来自 Yonder 进程或 Agent Gateway。依据仓库围栏允许的“结构化日志证据”，本目标以完整协议、Connector、Desktop 合约测试和正式原生进程启动记录判定通过，不把检查器失败伪报为产品 UI 证据。

## 边界结论

- macOS Codex CLI 路径：PASS。
- Windows：Deferred，未纳入本次完成声明。
- ChatGPT Desktop：Unsupported，未纳入本次完成声明。
- 回复正文、问题、截图和 Codex turn/item 细节仅驻有界运行时内存；关闭、终态、5 分钟超时或断连后释放。
