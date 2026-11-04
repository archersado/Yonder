# Windows 补充验证：本地 CLI/MCP

日期：2026-10-23
结论：FAIL（产品 Windows Transport 缺失）

- `yonder.exe mcp` 能启动，`tools/list` 返回 14 个工具。
- 调用 `task_list` 返回 `无法定位Yonder数据目录`。
- CLI 当前依赖 `HOME` 下的 macOS `Library/Application Support/Yonder/agent.sock`；desktop 本地 Socket 启动也仅在 macOS 条件编译范围内。
- 隔离 Named Pipe 协议样本通过，但产品 desktop 与 CLI 没有接入该实现，不能作为 AG-S1 产品 E2E 通过证据。

Windows Named Pipe 与原生验证任务保持未完成；依赖本地 Gateway 的 DS-S2 和 TM 产品 E2E 也继续受阻。
