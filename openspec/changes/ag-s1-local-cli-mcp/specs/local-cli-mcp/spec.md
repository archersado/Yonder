# 本地 CLI 与 MCP

## ADDED Requirements

### Requirement: 当前用户私有本地Gateway
macOS桌面进程必须在应用数据目录提供0700父目录、0600端点的UDS，每连接固定可信Agent身份并建立独立GatewaySession；不得开放HTTP/TCP或第二个TaskHost。

### Requirement: Codex MCP桥接
安装包内`yonder mcp`必须以MCP stdio暴露既有任务创建、读取、取消与步骤工具，连接后先完成Gateway 1.4握手；不得启动第二个桌宠或接受调用方自报agent_id。

### Requirement: 有界失败与生命周期
完整帧最多64KiB，坏帧只关闭当前连接；桌面未运行、协议拒绝和业务失败必须结构化反馈且不泄漏请求正文。桌面正常退出清理端点，断连不得自动重试副作用。

### Requirement: 平台完成边界
macOS必须提供真实UDS、无TCP、单桌宠及Codex MCP创建任务证据。Windows使用Named Pipe但当前暂缓，不得标记通过或Archive完整Story。
