# Proposal：Windows ego-lite BUA Bridge

Story：BU-S1。关联 Accepted AD-E0-03。

用户于 2026-10-24 确认 Windows 已安装且可从 PowerShell 启动 ego lite，满足原重启条件。将既有 macOS `BrowserUsePort` 与 ego-lite Worker 以同一契约接入 Windows，提供 create/reuse、observe、handOff、takeOver、finish；不复制 Browser Task Space，不回退 CUA，不改变 Agent 协议或 SQLite。

Architecture Impact：conforming。模块：BU。依赖方向不变；无协议、持久化或迁移变化。
