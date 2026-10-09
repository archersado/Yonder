# AG-S5 连接状态 macOS Verification Goal

日期：2026-10-09

## 目标

证明桌宠不会再把可用的MCP任务Gateway误报为“Agent未连接”，同时不把单向MCP连接冒充为可接收反向输入的双向会话。

## 证据

- `cargo test -p yonder-desktop agent_input --lib`：5/5通过；新增用例覆盖完全断开、仅Gateway、Gateway与输入会话同时连接、仅输入会话移除及最终断开。
- `cargo check -p yonder-desktop --bin yonder-desktop`与`cargo test -p yonder-cli`：通过，CLI 5/5。
- `openspec validate ag-s5-agent-input-channel --strict`与`python3 scripts/check_architecture.py`：通过。
- 正式debug `.app`由`package-macos-preview.py`生成并经LaunchServices启动；包内`yonder mcp`执行真实`task.get`后，CUA可访问树显示“任务通道已连接 · 输入通道未连接”。
- 关闭同一MCP stdio进程后，可访问树立即变为“任务通道未连接 · 输入通道未连接”；重新启动并调用Gateway后恢复为前一状态，无SQLite轮询。

## 结论

macOS独立Verification Goal通过。任务通道与输入通道已由各自真实连接生命周期驱动，连接徽标及可访问文本不会再混淆两种能力。Windows验证仍按主人既有决定暂缓，AG-S5不据此Archive。
