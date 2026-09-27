# 独立 Verification Goal：CM-S1 macOS Agent Command 本机批准

日期：2026-09-27
结论：PASS（macOS Agent Command 本机批准增量；完整 Story 未通过）
Result: PASS

## 验证目标

验证 Agent 只能提议有界结构化命令，本机用户能够在 Task Space 核对完整内容并批准一次或拒绝；归属 Agent 只能用 `command_id` 消费一次批准，执行必须先进入 TM-S7 启动事务，确定结果写入 observed，不确定结果写入 unknown 且不自动重试。响应、事件和日志不得返回完整命令正文。

## 自动化结果

- `cargo test -p yonder-protocol command_execute_contract_only_accepts_the_approved_reference`：通过；执行请求只接受任务、CAS sequence 与 `command_id`，拒绝命令正文和替代确认字段。
- `cargo test -p yonder-adapters approved_command_starts_once_and_records_unknown_without_retry`：通过；覆盖未批准、sequence 冲突、一次消费、重放、unknown 与超时不重试。
- `cargo test -p yonder-desktop --lib`：11/11 通过；其中 `command_gateway_requires_local_approval_and_executes_reference_once` 通过真实 TaskHost、SQLite、Gateway 与 macOS `StructuredCommandAdapter` 执行 `/usr/bin/printf`。
- `cargo test -p yonder-application`：39/39 通过；覆盖 Registry 的 LocalUser 权限、摘要绑定、过期、撤权与有界容量。
- `cargo test -p yonder-cli`：5/5 通过；CLI/MCP 只暴露提议与引用执行参数。
- `cargo run -p yonder-protocol --example generate --locked -- --check`、`python3 scripts/check_architecture.py`、`openspec validate cm-s1-agent-command-approval --strict`：通过。

2026-09-27 TM-S7 交叉审阅发现批准顺序缺口：旧实现虽不会在未批准时调用 Command Adapter，却会先提交 `created→running` 和失败 attempt。现已在 `start_execution` 前增加不返回正文、不消费引用的批准预检；只有已批准且任务、归属 Agent、sequence、摘要和期限仍匹配的引用才能进入启动事务。启动提交后仍原子消费一次，随后才派发。Application、SQLite Adapter 与真实 desktop Gateway 定向回归通过；未批准样本保持 `created` 且没有 attempt。

## macOS 原生界面证据

- 隔离验证器：[`check-command-approval-macos.swift`](../../../apps/desktop/check-command-approval-macos.swift)。
- 待批准截图：[`native-webkit-command-awaiting.png`](../../../apps/desktop/evidence/cm-s1-agent-command-approval-macos-20260927/native-webkit-command-awaiting.png)，完整显示程序、参数、工作目录、环境变量、超时、有效期、“批准执行一次”和“拒绝”。
- 已批准截图：[`native-webkit-command-approved.png`](../../../apps/desktop/evidence/cm-s1-agent-command-approval-macos-20260927/native-webkit-command-approved.png)，显示“已批准，等待归属 Agent 执行”和“撤销批准”。
- 结构化结果：[`result.json`](../../../apps/desktop/evidence/cm-s1-agent-command-approval-macos-20260927/result.json)，批准与撤销各调用一次。

界面证据使用 macOS 原生 `WKWebView` 加载产品 HTML/CSS/JavaScript，并注入隔离 Tauri 调用夹具；它不连接真实任务库，也不执行命令。真实 SQLite/Gateway/命令执行链路由 desktop 集成测试独立覆盖。两类证据组合验证 macOS 增量，但不把夹具描述为正式宿主 E2E。

## 审阅结论与边界

- PASS：协议 1.30 协商、无副作用提议、本机完整预览、批准/拒绝、TM-S7 启动、真实 macOS Adapter、一次消费、撤权、超时和 unknown 不重试。
- 不在范围：Shell、提权、安装、删除、支付、发送、批量命令、云端 WSS、Jev 决策循环。
- Windows Job Object 与 Windows 原生 E2E 按用户决定延期；因此 CM-S1 完整 Story 保持 `verifying`，本 Change 暂不 Archive，也不得将 Story 标记 Done。
