# 本地 CLI 与 MCP

## ADDED Requirements

### Requirement: 当前用户私有本地Gateway

桌面进程 MUST 在 macOS 提供当前用户私有 UDS、在 Windows 提供当前用户私有 Named Pipe，并按连接绑定可信Agent身份。

#### Scenario: 建立私有UDS连接

macOS桌面进程必须在应用数据目录提供0700父目录、0600端点的UDS，每连接固定可信Agent身份并建立独立GatewaySession；不得开放HTTP/TCP或第二个TaskHost。

#### Scenario: 建立 Windows 私有 Named Pipe 连接

Windows 桌面进程必须通过固定 namespaced Local Socket 提供当前用户私有 Named Pipe，每连接固定可信 Agent 身份并建立独立 GatewaySession；不得开放 HTTP/TCP、接受外部覆盖名称或启动第二个 TaskHost。

### Requirement: Codex MCP桥接

安装包内`yonder mcp` MUST 通过MCP stdio桥接既有Gateway工具，并先完成协议握手。

#### Scenario: MCP握手后创建任务

安装包内`yonder mcp`必须以MCP stdio暴露既有任务创建、读取、取消与步骤工具，连接后先完成Gateway 1.4握手；不得启动第二个桌宠或接受调用方自报agent_id。

### Requirement: 有界失败与生命周期

本地CLI/MCP连接 MUST 提供有界帧和结构化失败反馈，且不自动重试副作用。

#### Scenario: 桌面未运行

完整帧最多64KiB，坏帧只关闭当前连接；桌面未运行、协议拒绝和业务失败必须结构化反馈且不泄漏请求正文。桌面正常退出清理端点，断连不得自动重试副作用。

### Requirement: 平台完成边界

Story完成证据 MUST 分别覆盖 macOS 真实 UDS、Windows 真实 Named Pipe、无 TCP、单桌宠和 Codex MCP 创建任务。

#### Scenario: macOS完成证据

macOS必须提供真实UDS、无TCP、单桌宠及Codex MCP创建任务证据。

#### Scenario: Windows 完成证据

Windows 必须提供真实 Named Pipe、无 TCP、单桌宠及 Codex MCP 创建任务证据；不得以隔离管道样本或 macOS 证据替代。
