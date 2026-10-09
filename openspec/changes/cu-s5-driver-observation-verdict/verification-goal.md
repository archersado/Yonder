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

2026-10-09 使用重建后的正式 GUI/Gateway 执行了播放样本
`task_d13dcc0d0712d4589fba48cdf1edca64`，结果为 **BLOCKED**：

- `/Users/archersado/.cargo/bin/cargo test --offline --locked -p yonder-protocol -p yonder-application`：
  协议 18 项、Application 79 项全部通过；`openspec validate
  cu-s5-driver-observation-verdict --strict` 与 `python3 scripts/check_architecture.py`
  同时通过。

- 首轮打开 QQ 音乐后，Driver 在序号 5 返回有效 Observation；未提交慢脑
  `achieved` 核验时，序号 7 的 `task.complete` 被安全拒绝。
- 慢脑分别在序号 8、18、28、34 提交 `not-achieved`；每次任务均保持
  `running`，且允许归属慢脑提交新的受限计划片段，证明未达成结论不会被误当成完成。
- 序号 31 的恢复 Observation 证明窗口已经可见，但播放控件仍为“播放”；
  `bring-to-front` 的结构成功没有被提升为播放目标达成。
- 基于恢复后新鲜元素引用执行播放控件时，序号 37 返回
  `driver-action-unconfirmed`；基于同一正式 Observation 截图改选另一播放控件时，
  序号 41 返回 `driver-action-rejected`。两项副作用均未自动重试，任务在序号 43
  安全取消。

本次实机证据通过了“无 achieved 不得完成”“not-achieved 后可继续规划”和
“结构动作事实不得合成业务成功”三项边界；但当前 Sky 产品 Driver 尚不能对该样本产出可核验的
播放动作，因此缺少最终 `achieved → task.complete` 正向证据。Verification Goal
保持进行中，任务清单不勾选，Change 不得 Archive。后续应先在对应 Driver Story
修复受 Observation 约束的控件激活，再新建独立任务重跑正向样本。
