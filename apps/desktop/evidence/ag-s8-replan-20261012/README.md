# AG-S8 replan 终局样本（2026-10-12）

- 正式 Yonder MCP 成功读取任务，新建零用户数据副作用样本并接受计划片段。
- Driver 在不存在的隔离应用上返回 `unknown/worker-failed`，Yonder 在序号 6 正式交回；未重试该副作用。
- 当前 Codex CLI 实际版本为 0.156.1，SessionStart 未拉起 Connector。手动经已安装插件的正式 `connect.sh → yonder codex-hook → app-server proxy` 路径补发后，当前 thread 未出现在共享 daemon 的 `thread/loaded/list`，`thread/resume` 被拒绝。
- 临时隔离安装 Codex CLI 0.162.1 后重试同一活动 thread，仍在 `thread/resume` 阶段被拒绝；说明必须由 0.162.1 从会话启动时拥有该 thread，不能在当前 turn 中换宿主伪造通过。
- 兼容修复已覆盖“共享 daemon 已加载同一 thread 时直接附着，不重复 resume”，20 项 `yonder-cli` 合约测试通过。

结论：样本保持 `blocked`，不将 AG-S8 Verification Goal 标记为 PASS。下次必须用 Codex CLI 0.162.1 新建会话，由受信 SessionStart Hook 自动建立 Connector 后重放同类零副作用 handback。
