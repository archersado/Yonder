# Verification Goal：EX-S2 可配置 Jev 模型

## 目标

证明正式 macOS `Yonda.app` 从独立设置面板保存本地 endpoint 与模型标识后，经安装包 MCP、Agent Gateway、Application、Jev Adapter 和本地 `/v1/systemone` 完成一次真实多 AX 候选选择；同时证明已有配置兼容、非法模型拒绝及远端白名单不回退。

## 门槛

- Application、Adapter 与桌面 UI 自动化测试通过。
- `node --check` 与 `openspec validate ex-s2-configurable-jev-model --strict` 通过。
- 正式任务事件证明第一动作 Observe 有效、多个自然 AX 候选进入 Jev、模型选择已派发或明确返回可区分 handback；不得直接调用 Worker、Driver、Socket 或 SQLite 代替产品链路。
- 不记录 API Key、完整模型请求、AX transcript、截图或候选正文。
