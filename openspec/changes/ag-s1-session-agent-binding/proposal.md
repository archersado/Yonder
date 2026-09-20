# AG-S1 本地会话 Agent 身份绑定

关联 AG-S1 与更新后的 Accepted AD-AG-05。移除 macOS UDS 对 `codex-cli` 的固定身份：连接的首个 `gateway.hello` 绑定其 `agent_id`，后续帧仅能使用该 ID。Architecture Impact：architecture-change（本地 Gateway 身份绑定）。

不新增认证机制、端口或持久化字段。当前 OS 用户私有 UDS 仍是 MVP 认证边界；`agent_id` 仅决定任务归属隔离。
