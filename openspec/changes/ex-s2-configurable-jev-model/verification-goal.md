# Verification Goal：EX-S2 可配置 Jev 模型

## 目标

证明正式 macOS `Yonda.app` 从独立设置面板保存本地 endpoint 与模型标识后，经安装包 MCP、Agent Gateway、Application、Jev Adapter 和本地 `/v1/systemone` 完成一次真实多 AX 候选选择；同时证明已有配置兼容、非法模型拒绝及远端白名单不回退。

## 门槛

- Application、Adapter 与桌面 UI 自动化测试通过。
- `node --check` 与 `openspec validate ex-s2-configurable-jev-model --strict` 通过。
- 正式任务事件证明第一动作 Observe 有效、多个自然 AX 候选进入 Jev、模型选择已派发或明确返回可区分 handback；不得直接调用 Worker、Driver、Socket 或 SQLite 代替产品链路。
- 不记录 API Key、完整模型请求、AX transcript、截图或候选正文。

## 2026-10-09 本地模型首轮结果

正式任务 `task_e8b300f7aa99327809f9bc2508f71c4b` 经新版 `Yonda.app → MCP → Gateway → Application → Jev Adapter` 执行。系统设置启动与动作后 Observe 成功；自然 AX 多候选进入快脑后约 30 秒安全交回，任务事实为“快脑决策不可用”，未派发候选点击，测试随后取消。

同一安装包固定的 TypeSafe SDK 0.6.0 对 `http://127.0.0.1:8002`、模型 `english` 运行等价的十候选有界请求，精确在 30027ms 返回 `APITimeoutError`；同期 `/health` 为 HTTP 200 且 `english` 已加载。两候选最小请求可在 1 秒内返回兼容答案。因此配置模型传递与本地契约已越过原远端门禁，本轮阻塞收敛为十候选推理超过 Adapter 30 秒硬上限；不能记为完整正向 PASS。
