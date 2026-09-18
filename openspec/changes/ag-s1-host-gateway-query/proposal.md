# AG-S1 正式宿主Gateway查询

关联docs/specs/epic-AG/story-AG-S1及AD-OCT-05、AD-DS-01、AD-ST-01。问题：已有GatewaySession只在核心测试使用，正式TaskHost只提供本机查询。首批让可信会话读取同一SQLite，保留握手与归属门禁，为实际接入准备；不提供任务创建或公开传输。

Architecture Impact：conforming。desktop→application/adapters不变，Rust协议/SQLite schema不变，GatewaySession与TaskHost所有权不变。实际IPC认证与执行门禁未满足，不顺带开放。Windows继续暂缓。
