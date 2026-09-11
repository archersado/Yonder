# E0-S1 桌面基础栈验证

## Story

作为 Yonder 开发团队，我们需要验证 Tauri 2、常驻透明窗口、托盘和跨平台 Local Socket 的真实行为，以决定桌面基础技术方案是否可进入产品研发。

## 验收条件

1. 最小应用可构建并显示透明、无边框、置顶窗口与托盘入口。
2. Rust Core 可启动 Local Socket echo 服务；同用户 CLI 可连接并完成版本化 JSON 消息往返。
3. Windows 使用 Named Pipe，macOS 使用 Unix Domain Socket，不开放 TCP 端口。
4. 分别记录 Windows/macOS 的启动时间、空闲 CPU、空闲内存、窗口事件延迟及权限行为。
5. 形成 AD-E0-01，明确通过、受限通过或淘汰；不得用 Linux 结果替代目标平台证据。

OpenSpec：`openspec/changes/e0-validate-desktop-foundation/`
