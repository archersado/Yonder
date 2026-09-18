# AD-TM-10 Agent 等待用户状态提交

状态：Accepted（2026-09-18，TM-S1 等待用户首批子范围）。

## 背景

产品简报要求桌宠呈现“等待用户介入”，架构主干要求持久化等待原因。Domain、SQLite 状态和桌宠已经识别 `waiting-for-user`，但正式 Gateway 没有进入该状态的入口，因此运行任务无法用真实 Agent 事实触发对应表现。

## 决定

协议 1.17 增加 Agent 能力 `task.wait-for-user` 与方法 `task.wait_for_user`。请求携带 `task_id`、`expected_sequence` 和 1～512 字节的 `reason`；仅归属 Agent、`running` 状态、最后一次 attempt 已 Observe 并推进到 `stopped` 步骤边界且没有待处理控制请求时接受。

SQLite schema 14 将等待原因写入产生 `running → waiting-for-user` 的同一事件；状态、事件和 Outbox 保持同事务。`task.events` 在 1.17 返回该事件的 `wait_reason`，旧协议投影移除该字段。提交成功后释放任务准入资源，使桌宠由既有事件驱动映射显示等待状态。

## 边界

本决定不提供 Resume、不自动提交用户输入、不新增“等待外部响应”任务状态，也不允许在未确认动作或 unknown 后释放资源。恢复仍受 TM-S4、RC-S1 与新鲜 Observe 门禁约束。
