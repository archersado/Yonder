# yonder CLI

Codex本地接入：先启动Yonda桌面应用，再执行：

```bash
codex mcp add yonder -- /path/to/yonder mcp
codex mcp get yonder
```

Codex通过MCP stdio启动CLI；CLI经当前用户私有UDS调用运行中的Yonda，不启动第二个桌宠。写工具仍遵循Codex审批。
