# Verification Goal：CU-S5 Driver Observation 慢脑判定

## 目标

证明正式计划链不再通过动作类型合成业务事实，归属慢脑能够读取 Driver 返回的最新 Observation，并独立提交目标核验。

## 通过条件

- 协议 1.47、Schema 与 TypeScript 只由 Rust 生成且无漂移。
- Application 测试证明不同 `CuaActionKind` 不改变结构事实；结构事实严格来自 DispatchOutcome/ComputerObservation。
- 旧业务条件缺少 Driver 证明时交回，不误报步骤或任务成功。
- macOS 正式 Gateway 播放样本返回最新 Observation；无慢脑 achieved 核验不能完成，慢脑 not-achieved 可继续新计划，最终 achieved 才完成。
- Windows 原生证据按用户决定暂缓。

## 结果

2026-10-08 自动化基线通过：

- `cargo test --offline --locked -p yonder-protocol -p yonder-application`：协议 18 项、Application 67 项全部通过。
- `cargo test --offline --locked -p yonder-adapters task_store::tests::plan_fragment_continuously_executes_verified_slots_in_one_call`：结构条件连续片段通过。
- `cargo test --offline --locked -p yonder-desktop --lib`：25 项全部通过，发布契约已更新至协议 1.47。
- `check_architecture.py` 被既有、非本变更的 `CU-S6/architecture-design.md` 缺少“状态与契约”章节阻断；本增量没有修改 CU-S6。

正式 macOS Gateway 播放样本尚待重建产品后执行，因此本 Goal 仍为进行中，不能 Archive。
