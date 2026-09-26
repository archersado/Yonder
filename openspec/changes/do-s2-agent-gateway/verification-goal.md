# Verification Goal：DO-S2 macOS Agent 文档 Gateway

状态：PASS（macOS Agent Gateway 子范围，2026-09-26）。Windows 按用户决定延期；完整 DO-S2 的跨平台验证仍保持 `verifying`，不 Archive/Done。

## 范围

- Story：DO-S2 的 DO-GW-01～04，以及 DO2-07 的 macOS Gateway 闭环。
- OpenSpec：`do-s2-agent-gateway`。
- 架构：Accepted AD-DO-01、AD-DO-02、AD-FI-01～04、AD-TM-13。
- 环境：macOS；验证只使用仓库合成 DOCX 与唯一临时目录，不访问用户文档或正式任务库。

## 验证矩阵

| 场景 | 可观察证据 | 结果 |
|---|---|---|
| 契约边界 | `task.document.execute` 要求双授权、64 位小写哈希和有界文本；路径字段、同一授权及不合法哈希拒绝 | PASS |
| 统一入口 | Gateway 仅在 1.29 握手和 TaskHost 组合 Document/File Runtime 后公布 `document.execute` | PASS |
| 默认另存 | macOS TaskHost 以同任务 `read` + `create-new` 授权执行 DOCX 唯一替换；原件逐字节保持不变，输出新建 | PASS |
| 安全投影 | Gateway 响应只含执行摘要、格式、hash 和字节数；测试确认源路径、输出路径和授权根均未出现 | PASS |
| 授权与状态 | 输出授权一次性消费；首次副作用经 TM-S7 attempt 启动/observed 结果记录；授权解析失败也收束 attempt | PASS |
| 既有防护复用 | Document/File Port 的源 hash、目标存在、锁、临时校验和 unknown 不重试由 DO-S2 文件 Runtime 回归覆盖 | PASS（复用） |
| Windows | 未运行 Windows 原生验证；不计为通过 | 延期 |

## 命令与结果

- `cargo run -p yonder-protocol --example generate`：通过，Rust 协议生成物已同步。
- `cargo test -p yonder-protocol -p yonder-application -p yonder-desktop -p yonder-cli`：74 项 Rust 测试通过。
- `cargo test -p yonder-desktop tests::document_execute_uses_dual_grants_and_default_save_as_without_paths -- --exact --nocapture`：macOS TaskHost/Gateway 结构化集成测试通过。
- `python3 scripts/check_architecture.py`、`openspec validate do-s2-agent-gateway --strict`、`git diff --check`：通过。

## 范围限制

本 Goal 不证明 Windows 文件运行时、原生 Windows 选择器、Office/WPS GUI 打开验证、批量或覆盖式 Agent 编辑、XML/ZIP 访问或自动重试。后续 Windows 验证必须另建增量与独立 Goal；Agent 仍须在既有 Gateway 链路显式 Observe、推进与完成任务。
