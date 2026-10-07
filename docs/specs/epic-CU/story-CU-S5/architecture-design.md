# CU-S5 架构设计

## 边界与依赖

Desktop 只消费 Application 投影；Adapters 只实现 Store/Driver Port；Application 持有核验规则并依赖 Domain/Protocol；CLI 只依赖 Rust Protocol。目标判断由归属慢脑经 Gateway 输入，Jev、CUA Driver、BUA Bridge、Office 与 Command Adapter 均不得自行完成任务或互相调用。

## 状态与契约

Application 新增 `GoalVerification`：`verification_id`、`task_id`、`observation_sequence`、`outcome`、`verified_sequence`。它只表达终局目标判断，不保存理由正文。活动任务由 ExecutionRuntime 持有最新核验；兼容执行链由 TaskStore 以同一事件、Outbox 和状态批次投影。

任何新步骤、计划、用户控制或执行事件发生后，核验因其 `verified_sequence` 不再等于任务当前序号而自然失效，无需可变覆盖旧记录。

## Gateway 与协议

Rust 协议新增 `task.goal.verify`、`task.goal.verify` capability、`GoalVerifyParams` 与核验响应。`task.complete` 增加 `verification_id`。Gateway 校验握手、归属、deadline、CAS、Observation 序号和核验 ID，不解释模型语义。

## Application 与持久化

Runtime 命令 `VerifyGoal` 追加连续事件并更新快照；`not-achieved` 同时形成 HandBack 状态，`achieved` 保持 Running 并开放终结门禁。SQLite projector 批量投影事件与 Outbox；尚未迁入 Runtime 的兼容链使用 `task_goal_verifications` 不可变记录完成同样 CAS。

完成用例必须检查最新 achieved 核验 ID、核验后无新事件、当前无执行中步骤。失败终结不要求 achieved 核验，以免真实失败无法上报。

## 边界

目标语义判断仍属于归属慢脑，不进入 Jev、Driver、SQLite trigger 或 React。核验不携带模型思维链，不自动重试动作，也不放宽发送确认和 unknown 规则。

## 失败与验证

无核验、not-achieved、错误 ID、陈旧序号和非最新 Observation 均安全拒绝完成；projector 失败只形成运行时积压，不撤回已接受的内存核验事实。测试覆盖协议严格解码、Runtime 失效、SQLite CAS/事件/Outbox、Gateway 能力协商、CLI 工具和顶部投影；macOS 独立 Goal 通过后才进入 verifying，Windows 继续暂缓。
