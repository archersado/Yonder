# AD-TM-21 执行尝试开始历史投影

状态：Accepted（2026-09-24，TM-S5 执行尝试开始历史子范围）。关联 TM-S2 / TM-S5、CU-S2、DS-S2。Architecture Impact：architecture-change（协议 1.25）；不改变执行状态机、Driver 或持久化。

## 背景

Accepted AD-TM-08 已规定 `prepare_attempt` 把 `created→running`、Start 事件、Outbox 与 Application 生成的完整执行身份原子保存到 `task_attempts.accepted_sequence`。协议 1.5 只投影尝试最终 `attempt_result`，当前时间线在步骤声明与最终结果之间缺少已提交的尝试起点，用户无法按历史确认哪个 step/attempt 被正式准备。

## 决定

- Adapter 只按 `task_attempts.accepted_sequence = events.sequence` 投影 `attempt_started`，携带不可变 `step_id`、`attempt_id`、`worker_instance_id`、`host_session_id`。
- 该字段固定表示 prepared 尝试已原子接受，不读取记录稍后的当前 phase 反推过去；不表示 Driver 已派发、动作成功、Observe 有效或安全停止。
- Rust 协议 1.25 为 `TaskEvent` 增加可选 `attempt_started`。1.24 及以下字段缺省；Gateway 沿用实时授权、连续性和 8 KiB/256 KiB 编码预算。
- Task Space 只显示步骤与尝试标识，不显示 Worker/宿主内部标识，不展示原始参数、输入、截图或完整命令输出。
- schema 7 以前没有尝试记录的旧任务不回填；损坏或部分身份明确失败。不得从 `running` 状态或最终结果补造开始事实。

## 验证

覆盖准备与最终结果保持不同序号、1.24/1.25 隔离、越权拒绝、损坏身份、缺失记录零回填及响应预算。macOS 使用隔离数据库、私有 Unix Socket 和正式 Tauri 宿主验证；Windows 按用户决定暂缓。
