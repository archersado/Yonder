# yonder CLI

Codex本地接入：先启动Yonda桌面应用，再执行：

```bash
codex mcp add yonder -- /path/to/yonder mcp
codex mcp get yonder
```

Codex通过MCP stdio启动CLI；CLI经当前用户私有UDS调用运行中的Yonda，不启动第二个桌宠。写工具仍遵循Codex审批。

## Agent用户输入桥接

需要Codex CLI 0.156.1或更高版本。Codex TUI和Yonder桥接必须连接同一App Server，不能把普通CLI会话的持久队列当作当前turn。以已知会话ID为例：

```bash
codex app-server --listen unix:///tmp/yonder-codex.sock
codex --remote unix:///tmp/yonder-codex.sock resume <thread-id>
YONDER_AGENT_ID=codex YONDER_CODEX_THREAD_ID=<thread-id> YONDER_CODEX_APP_SERVER_SOCKET=/tmp/yonder-codex.sock /path/to/yonder agent-bridge
```

桥接直接使用该监听socket上的WebSocket传输，只在thread已由App Server加载后向Yonder声明`user_input`；活动turn直接调用`turn/steer`，空闲或中断后调用`turn/start`。App Server未确认时不会向Yonder报告成功。
