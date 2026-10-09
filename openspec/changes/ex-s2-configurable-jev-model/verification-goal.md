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

用户随后澄清面板实际保存的是 `typed-decisions`。服务首次收到该模型时执行按需 checkpoint 加载，撞上 30 秒 Adapter 上限；加载完成后 `/health` 明确投影 `loaded=["typed-decisions"]`。新的正式任务 `task_107841c7bdc8978f1b8a42fab0020de9` 在暖模型下 4.6 秒完成 Jev 调用，不再返回依赖不可用，而是按既有门禁以“快脑置信度不足”交回，未派发候选点击并已取消清理。这证明本地 endpoint、配置模型名与 Adapter 调用链 PASS；完整动作正向结果仍受本地模型置信度语义与产品固定 0.75 门槛不匹配阻断。

## 2026-10-09 置信策略配置增量

按 AD-EX-03 与 EX2-24，`JevConfig` 保存可填写的 `confidence_field` 和
`confidence_threshold`。字段名为空、历史 JSON 缺字段或使用默认值时均读取
`confidence + 0.75`；旧 `confidence_metric` 兼容迁移。候选选择与步骤核验共用同一配置
快照，Worker 严格读取选定字段，字段缺失、非有限或越界时失败并交回，不探测其他属性。

结构化验证结果：

- `cargo test --offline --locked -p yonder-application --lib`：79 项通过；覆盖可填写/空字段、默认 `confidence`、旧字段迁移，以及配置阈值同时作用于候选选择与步骤核验。
- `cargo test --offline --locked -p yonder-adapters --lib`：85 项通过；其中 Jev 模块 7 项通过，证明 `answer_confidence` 缺失时不会读取仍存在的 `confidence`。
- `cargo test --offline --locked -p yonder-desktop --lib`：29 项通过。
- `node --check crates/adapters/src/jev_worker.mjs`、`node --check apps/desktop/ui/jev-settings.js` 与 `openspec validate ex-s2-configurable-jev-model --strict` 通过。
- `cargo fmt --all -- --check` 仍被仓库既有大量未格式化 Rust 文件阻断；本 Change 的 `git diff --check` 通过，未对无关文件执行全仓机械重排。

正式 macOS GUI 面板保存与本地 `multilingual` Gateway 样本仍须在合并、重建并重启
`Yonda.app` 后执行；本节不提前宣称原生 Goal PASS。
