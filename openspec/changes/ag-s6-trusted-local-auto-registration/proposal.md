# AG-S6 可信本地 Agent 自动登记

关联 AG-S6、AD-AG-08、EX-S2 原生验证。Architecture Impact：architecture-change（本地认证完成后的 Agent 生命周期）。本 Change 仅让已由 Desktop 私有开发 stdio 组合根绑定的固定身份，在首次 Gateway 会话前登记为 enabled；不信任请求 JSON 的 agent_id，不自动登记 UDS、WSS 或未知传输。用户禁用、撤权及会话断开语义保持优先。
