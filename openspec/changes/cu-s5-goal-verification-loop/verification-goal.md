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

仓库全局 `scripts/check_architecture.py` 仍被既有 `TM-S9` Story README 缺少 `Story:` 字段阻断；CU-S5 自身所需章节、双向 OpenSpec 关联与严格 OpenSpec 校验均已通过。该既有规划缺口未在本 Story 越界修改。
