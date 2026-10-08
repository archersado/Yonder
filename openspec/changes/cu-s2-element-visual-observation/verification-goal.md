# CU-S2 元素优先与窗口视觉证据 Verification Goal

状态：PASS（核心、隔离Worker与正式macOS Yonder Runtime双样本）；Windows暂缓，完整Story不Archive。

## 已完成检查

- `cargo test -p yonder-protocol -p yonder-application`：67项PASS。
- `cargo check -p yonder-desktop`：PASS。
- `python3 apps/desktop/check-cua-visual-fallback.py`：证明歧义元素不执行副作用、只采集同一后台可信窗口截图，输出`element_first=true`、`visual_fallback=true`、`side_effect_executed=false`。

## 2026-10-08 Sky 正式 Worker 回归修正

- 修正正式桌面组合根使用 Sky Worker 后的策略偏差：正常 AX/transcript 成功路径不再落盘或通过 Gateway 返回截图；坐标动作、动作结果不可核实，以及副作用派发前的元素缺失/歧义边界才生成临时截图。
- `target-semantic-element` 在动作派发前收敛为 `refused + observe_valid`，并把同一可信窗口的临时 Observation 随计划交回；坐标引用过期继续保持 `unknown/unverifiable`，两者均不重放动作。
- `plan_fragment_continuously_executes_verified_slots_in_one_call` 覆盖 `UnknownObserved → handback` 保留截图引用；Gateway 合约测试覆盖协议版本门禁、响应编码解码及 replan wake 不携带敏感正文。
- `node --check crates/adapters/src/sky_cua_worker.mjs`、Application/Gateway 与 Adapter 定向测试通过；正式GUI双样本已在下节补齐，因此macOS范围标记为PASS。

## 2026-10-08 正式 macOS Yonder Runtime 双样本

- 产品路径统一为 `Yonda.app → MCP stdio → Agent Gateway → 不可变计划片段 → Sky CUA Worker → Observe`，未直连 Socket、Driver、SQLite，也未用验证脚本代替正式链路。
- 元素优先样本 `task_15bdcb973c7dd129d751b20539f5d451`：QQ音乐唯一搜索文本框由AX/Transcript命中，两个槽位均 `action_succeeded=true`、`observe_valid=true`；最后一次Observation含250个元素且不含截图。目标核验为`achieved`，任务完成于序号13。
- 视觉回退样本 `task_00a26226111bc47c9a6ce2c070267b99`：TextEdit不存在目标搜索控件，第二槽位记录 `action_succeeded=false`、`observe_valid=true`，并交回同一窗口1172×976 JPEG临时Observation。原点击在副作用前明确拒绝，不是`unknown`且未自动重试；读取证据后任务在安全边界取消于序号12。
- 有界证据见 `apps/desktop/evidence/cu-s2-element-visual-formal-macos-20261008/result.json`。仓库不保存截图、Transcript、输入正文、完整Agent Payload或本机截图路径；任务事件只记录结构化attempt结果，不含图像数据。

## 仍暂缓

- Windows原生对照证据按主人决定暂缓。
- 完整CU-S2保持`verifying`，本Change不Archive。
