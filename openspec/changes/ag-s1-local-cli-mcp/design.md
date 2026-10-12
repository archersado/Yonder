# 设计

桌面组合根在TaskHost就绪后启动有界Local Socket服务，每连接固定`codex-cli` AuthContext并持独立GatewaySession。父目录0700、端点0600；Listener停止后清理端点。请求和响应沿用`crates/protocol`，不修改wire或SQLite。

`apps/yonder-cli`提供共享IPC调用与`mcp`子命令。MCP stdio只实现initialize、notifications/initialized、ping、tools/list和tools/call；工具参数转换为现有Rust Request并先hello 1.4。stdout保持纯MCP，错误为结构化tool result，stderr只写固定诊断。

MCP子进程与调用方stdio生命周期一致：stdin保持打开时阻塞等待下一帧，不设置业务空闲超时；stdin EOF、宿主取消或不可恢复的stdio错误时退出。此前“连续10分钟无输入即退出，父进程会按需重新拉起”的假设已被正式Codex长会话证伪，宿主会保留关闭的transport并使后续调用永久失败。Gateway断开只清除当前UDS连接并让本次调用结构化失败，后续工具调用重新hello；不关闭桌面TaskHost、不清任务状态、不重试副作用。

Windows Named Pipe、Agent注册/撤权、自动安装到PATH和云端连接不在本轮；这些缺口保留，不用本轮macOS证据替代。
