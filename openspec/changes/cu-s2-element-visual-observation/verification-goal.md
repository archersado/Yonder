# CU-S2 元素优先与窗口视觉证据 Verification Goal

状态：核心与隔离Worker检查PASS；正式macOS Yonder Runtime双样本待完成，Windows暂缓，不Archive。

## 已完成检查

- `cargo test -p yonder-protocol -p yonder-application`：67项PASS。
- `cargo check -p yonder-desktop`：PASS。
- `python3 apps/desktop/check-cua-visual-fallback.py`：证明歧义元素不执行副作用、只采集同一后台可信窗口截图，输出`element_first=true`、`visual_fallback=true`、`side_effect_executed=false`。

## 2026-10-08 Sky 正式 Worker 回归修正

- 修正正式桌面组合根使用 Sky Worker 后的策略偏差：正常 AX/transcript 成功路径不再落盘或通过 Gateway 返回截图；坐标动作、动作结果不可核实，以及副作用派发前的元素缺失/歧义边界才生成临时截图。
- `target-semantic-element` 在动作派发前收敛为 `refused + observe_valid`，并把同一可信窗口的临时 Observation 随计划交回；坐标引用过期继续保持 `unknown/unverifiable`，两者均不重放动作。
- `plan_fragment_continuously_executes_verified_slots_in_one_call` 覆盖 `UnknownObserved → handback` 保留截图引用；Gateway 合约测试覆盖协议版本门禁、响应编码解码及 replan wake 不携带敏感正文。
- `node --check crates/adapters/src/sky_cua_worker.mjs`、Application/Gateway 与 Adapter 定向测试通过。正式 GUI 双样本仍保留在下述门禁中，本次不提前把完整 Goal 标记为 PASS。

## 待完成原生证据

- 正式Yonder GUI与安装包内MCP：唯一元素路径不产生视觉定位截图。
- 同一窗口制造元素歧义：原动作未执行，协议1.34交回临时窗口Observation。
- 任务终结后临时截图清理，SQLite/事件/Outbox不含路径或图像数据。
