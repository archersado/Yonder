# Windows 补充验证：本地 CLI/MCP 复测

日期：2026-10-24
结论：PASS（Windows Transport 范围）

- Desktop 正式组合根在 Windows 启动当前用户 Named Pipe `com.yonder.desktop.agent`。
- 安装包同源 `yonder.exe mcp` 通过 Named Pipe 完成 Gateway 握手并到达任务用例；未登记 Agent 返回既有“Agent已禁用、撤权或未登记”，证明请求已越过 Transport，而非旧版“无法定位Yonder数据目录”。
- 隔离 Named Pipe 协议样本完成会话绑定和输入确认。
- Desktop 进程无 TCP 监听；没有新增本地 HTTP Server。

本次验证关闭 2026-10-23 的“产品 Windows Transport 缺失”失败项。Agent 登记后的完整任务业务流仍由既有 AG-S1/AG-S2 验证覆盖，不以绕过登记写库冒充通过。
