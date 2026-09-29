# TM-S9 架构设计

活动主链为 `Gateway command → ExecutionRuntime mailbox → reducer → Driver → result/Observe event → reducer → UI/Agent projection`。`ExecutionRuntime` 位于 Application，持有归属 Agent、状态、逻辑 sequence、step/attempt、计划游标、租约、pending control、最新 Observe 和持久化 checkpoint；Desktop、React、Adapter、SQLite 均不是活动状态所有者。

`PersistenceProjector` 单写者订阅 reducer 事件，按任务连续批量写 SQLite 当前快照、events 与 Outbox，ACK 只推进内存 checkpoint。软限合并只读展示刷新；硬限完成当前动作与 Observe 后禁止下一副作用并 HandBack。投影失败可重试相同事件批次，但不能重试 Driver。

schema 22 增加 `task_runtime_projections` 幂等指纹表。一个批次对 `tasks` 快照、`events`、`outbox` 和运行时指纹的写入必须同事务；重复投影只接受逐事件 payload 完全一致的连续前缀。投影通知为有界且可合并，队列已满时 reducer 不等待 SQLite，projector 依靠 dirty task 重试并在成功后 ACK。

CUA `computer.step`、EX-S2 `task.plan.execute`、BUA、Document/Office、Command 和控制链路都使用同一 Runtime Port：Driver 生命周期不持有 TaskHost/SQLite 锁，阶段错误作为类型化事件即时投影。活动 `task.get/events` 读取 runtime 并拼接尚未 checkpoint 的连续尾部；历史查询继续读取 SQLite。发送槽位仍在确认前 HandBack。迁移可分提交，但不保留长期 legacy 同步执行路径。
