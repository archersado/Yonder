# 设计

桌面组合根在TaskHost就绪后启动有界Local Socket服务，每连接固定`codex-cli` AuthContext并持独立GatewaySession。父目录0700、端点0600；Listener停止后清理端点。请求和响应沿用`crates/protocol`，不修改wire或SQLite。

`apps/yonder-cli`提供共享IPC调用与`mcp`子命令。MCP stdio只实现initialize、notifications/initialized、ping、tools/list和tools/call；工具参数转换为现有Rust Request并先hello 1.4。stdout保持纯MCP，错误为结构化tool result，stderr只写固定诊断。

MCP子进程在连续10分钟无stdin输入后退出；父进程可在下一次调用时重新拉起。该规则只清理空闲宿主进程，不关闭桌面TaskHost、不清任务状态、不重试副作用。

Windows 使用 `interprocess` namespaced Local Socket 映射 Named Pipe；desktop 与 CLI 共用固定安装内名称、相同 hello/身份/帧协议和唯一 TaskHost。Named Pipe 采用当前用户进程默认私有 DACL，不接受 UI、Agent 或环境变量覆盖名称。Agent注册/撤权、自动安装到PATH和云端连接仍不在本轮；Windows 与 macOS 分别提供原生证据。
