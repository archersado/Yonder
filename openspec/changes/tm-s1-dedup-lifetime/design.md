# 设计

`SqliteTaskStore::register` 在同一个 `IMMEDIATE` 事务内读取 `owner_agent_id + idempotency_key`。存在且内容一致时返回原任务和 `created=false`；不存在时完成任务、事件、Outbox、来源事实及创建映射后返回 `created=true`。内容不同仍返回幂等冲突。

Application 新增不含技术依赖的 `RegistrationOutcome { task, created }`，`TaskStore` 提供兼容默认方法，避免 UI 或组合根访问 SQLite。现有只需要任务快照的调用继续使用 `register`；正式 Gateway 创建路径使用带结果的方法，编码响应保持协议 1.26 不变。桌面创建信号只读取 `created`，不自行查询或比较任务列表。

事件预算继续由 Rust 协议常量和 `bounded_events_result` 实施。测试覆盖 JSON 转义后的实际字节、连续前缀和首项超限，不复制第二套估算逻辑。

失败矩阵：事务失败不产生 true；同键不同内容冲突且不提示；容量不足只阻止首次创建；命中既有记录不执行容量门禁；进程重启后结果一致。
