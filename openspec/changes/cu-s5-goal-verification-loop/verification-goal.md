# 独立 Verification Goal：CU-S5 目标状态验证闭环

Story: CU-S5
Result: PASS
平台：macOS 26.5.1；Windows 暂缓

## 通过条件

- 无核验、not-achieved、陈旧核验和错误 ID 均不能调用 `task.complete`。
- 最新 achieved 核验可由归属 Agent 完成任务。
- not-achieved 后可提交新计划，unknown 副作用没有自动重试。
- 活动 Runtime 在 SQLite projector 延迟时仍即时返回核验状态。
- 顶部浮窗正确展示等待核验、目标已核验和交回慢脑。
- macOS 正式 Gateway CUA 样本不会把中间面板误判为目标完成。

Windows 对等验证按用户决定暂缓，macOS 证据不得外推。

## 2026-10-07 证据

- 协议测试证明 `task.goal.verify` 严格校验 capability、CAS、Observation 序号与核验 ID；缺少核验 ID 的 `task.complete` 被拒绝，Schema/TypeScript 生成物无漂移。
- Runtime 测试证明核验只能绑定最新成功 Observation，且后续新步骤立即清除旧核验；projector 延迟测试继续证明执行状态不等待 SQLite ACK。
- SQLite/Application 集成样本证明片段结束返回 `awaiting-goal-verification`，无核验不能完成；achieved 核验与事件、Outbox 同序号写入后才能完成，`task.events` 可读取核验事实。
- Desktop 测试证明顶部浮窗在全部步骤成功后保留并显示“等待慢脑核验最终结果”，任务详情显示目标核验状态。
- `cargo test --workspace --locked -- --test-threads=1`：196 个单元/集成测试全部通过，Doc tests 全部通过。
- `openspec validate cu-s5-goal-verification-loop --strict`：PASS。

## 2026-10-09 正式链路回归修正

- 正式Yonder任务`task_62569dbd9e848494033501c858d3a348`在新鲜Observe后提交后续计划时复现：Store接受新版本并返回序号20，但活动Runtime仍以序号19绑定旧片段，导致`task.get`倒退且新片段永久CAS冲突。
- Runtime的`not-achieved`现显式进入`HandBack`；Gateway在写入不可变片段前拒绝非交回边界的replan，避免Store与Runtime分裂；`task.get`不再以较旧Runtime快照覆盖已接受的较新Store序号。
- Application 74项测试通过；SQLite/Runtime集成测试新增“片段耗尽→not-achieved→异步投影→同任务新版本片段连续执行”，验证新片段可重新绑定且没有重放旧动作。

仓库全局 `scripts/check_architecture.py` 仍被既有 `TM-S9` Story README 缺少 `Story:` 字段阻断；CU-S5 自身所需章节、双向 OpenSpec 关联与严格 OpenSpec 校验均已通过。该既有规划缺口未在本 Story 越界修改。

## 2026-10-12 Jev HandBack 后终结回归

- 正式任务 `task_0936219ea4be11d4aad8b4a94d6527c1` 只经打包后的 `Yonda.app → MCP stdio → Agent Gateway → Application Runtime → Sky Worker` 执行，没有直连 Socket、Driver 或 SQLite。
- 启动步骤在 Observation 序号 5 返回 QQ 音乐的“播放”按钮；归属慢脑提交 `not-achieved@8` 后，以新计划版本执行 `SPACE`。动作只派发一次，Observation 序号 11 返回“暂停播放”，同时记录 `action_succeeded=true`、`observe_valid=true`。
- Jev 因不解释开放业务语义在序号 13 交回；归属慢脑读取同一新鲜 Observation 后提交 `achieved@14`。Runtime 只恢复终结门禁，不重启 Driver、不重放动作；随后 `task.complete` 成功，任务在序号 15 为 `completed`。
- `cargo test -p yonder-application`：83/83 通过；暂停态仍拒绝目标核验，unknown、陈旧 Observation 与结果待核实副作用仍不能绕过门禁。`openspec validate cu-s5-goal-verification-loop --strict`、架构检查与 `git diff --check` 均通过。
- 有界结构化证据见 `apps/desktop/evidence/cu-s5-handback-achieved-20261012/`；不保存 transcript、截图、完整 Payload 或用户数据。
