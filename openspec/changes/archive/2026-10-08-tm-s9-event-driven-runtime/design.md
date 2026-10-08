# Design：事件循环与异步 projector

Application 新增 `ExecutionRuntime` command/event reducer；每任务逻辑序号由 reducer 分配。Driver 在受监管 Worker 执行，结果作为事件返回，Observe 后立即推进、HandBack 或暂停。Desktop 组合根用有界 channel 接线，并把展示事件直接投给控制条/桌宠。

`PersistenceProjector` 单写者批量消费连续事件，复用 SQLite 事务写快照、events 与 Outbox；ACK 不参与下一步决策。首阶段可从 SQLite hydrate 初始任务/计划，但 Driver 生命周期内不得访问 SQLite。硬背压只在下一副作用前暂停。

SQLite schema 22 以 `task_runtime_projections(task_id, sequence, payload)` 保存运行时事件的幂等指纹，并与既有 `events`、`outbox` 共用事务。projector 唤醒采用有界、可合并通知；通知满不会阻塞 reducer。写入失败保留 dirty task 定时重试；只有完整连续批次提交成功才回送 ACK，重复批次按 payload 一致性判定幂等，冲突关闭。
