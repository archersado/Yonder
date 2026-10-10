# yonder CLI

Codex本地接入：先启动Yonda桌面应用，再执行：

```bash
codex mcp add yonder -- /path/to/yonder mcp
codex mcp get yonder
```

Codex通过MCP stdio启动CLI；CLI经当前用户私有UDS调用运行中的Yonda，不启动第二个桌宠。写工具仍遵循Codex审批。

## Agent用户输入桥接（macOS正式接线）

需要 Codex CLI 0.162.1 或更高版本。首次安装并信任本仓库的 `yonder-codex-connector` 插件后，正常启动 Yonder 和 Codex CLI 即可，不再复制 thread ID 或维护 Socket：

```bash
codex plugin marketplace add /absolute/path/to/Yonder
codex plugin add yonder-codex-connector@yonder-local
```

安装后在 Codex `/hooks` 中审阅并信任该 Hook。`yonder` 必须是当前发布包安装到 PATH 的 CLI。Hook 只在 `startup|resume` 运行，读取 Codex 明确提供的当前 session ID；它不读取 transcript、内部数据库或最近会话。

Yonder 重启时后台连接器会在五分钟有界窗口内恢复同一会话。恢复只重新注册 Gateway，不重放未确认输入。ChatGPT Desktop 当前不支持，Windows 暂缓。

## 旧版显式桥（仅兼容与诊断）

旧版桥要求 Codex TUI 和 Yonder 桥接连接同一 App Server，不能把普通 CLI 会话的持久队列当作当前 turn。以已知会话 ID 为例：

```bash
codex app-server --listen unix:///tmp/yonder-codex.sock
codex --remote unix:///tmp/yonder-codex.sock resume <thread-id>
YONDER_AGENT_ID=codex YONDER_CODEX_THREAD_ID=<thread-id> YONDER_CODEX_APP_SERVER_SOCKET=/tmp/yonder-codex.sock /path/to/yonder agent-bridge
```

桥接直接使用该监听socket上的WebSocket传输，只在thread已由App Server加载后向Yonder声明`user_input`；活动turn直接调用`turn/steer`，空闲或中断后调用`turn/start`。App Server未确认时不会向Yonder报告成功。
