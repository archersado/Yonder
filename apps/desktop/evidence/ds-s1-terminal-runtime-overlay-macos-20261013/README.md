# DS-S1 终态状态一致性 macOS 证据

2026-10-13 使用 `dev` 正式 debug `Yonda.app`，从 Codex MCP 经 Yonder Local Socket、Agent Gateway、ExecutionRuntime 与 ego-lite BUA 完成连续 `create → finish`。未直连 SQLite、Socket、Driver 或测试宿主。

验收任务 `task_09edc4fd9bd30003ff7880522e51a4d0` 返回 `completed/sequence 12`；Yonda 顶部不再保留“正在”步骤。随后通过应用菜单正常退出并重启，`task.get` 仍返回 `completed/sequence 12`，证明连续尝试边界、异步终态投影和重启恢复均成立。

结构化结果见 `result.json`。Windows 原生验证按用户要求继续暂缓。
