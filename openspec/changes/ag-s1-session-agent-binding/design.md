# 设计

Socket 连接在读取第一帧前不创建 GatewaySession。第一帧必须是可解码的 `gateway.hello`；取其 `agent_id` 建立该连接专属 Session，随后复用既有协议校验拒绝身份不匹配、未握手与非法 ID。

`yonder mcp` 不再内置 Agent ID，启动时要求 `YONDER_AGENT_ID`，并在同一连接的 hello 和所有工具请求中使用它。测试覆盖两个不同 ID 分别创建任务，以及同一连接切换 ID 被拒绝。
