# Design：事件循环与异步 projector

Application 新增 `ExecutionRuntime` command/event reducer；每任务逻辑序号由 reducer 分配。Driver 在受监管 Worker 执行，结果作为事件返回，Observe 后立即推进、HandBack 或暂停。Desktop 组合根用有界 channel 接线，并把展示事件直接投给控制条/桌宠。

`PersistenceProjector` 单写者批量消费连续事件，复用 SQLite 事务写快照、events 与 Outbox；ACK 不参与下一步决策。首阶段可从 SQLite hydrate 初始任务/计划，但 Driver 生命周期内不得访问 SQLite。硬背压只在下一副作用前暂停。
