# TM-S9 架构设计

## 边界与依赖

活动主链为 `Gateway command → ExecutionRuntime mailbox → reducer → Driver → result/Observe event → reducer → UI/Agent projection`。`ExecutionRuntime` 位于 Application，持有归属 Agent、状态、逻辑 sequence、step/attempt、计划游标、租约、pending control、最新 Observe 和持久化 checkpoint；Desktop、React、Adapter、SQLite 均不是活动状态所有者。

## 状态与契约

`PersistenceProjector` 单写者订阅 reducer 事件，按任务连续批量写 SQLite 当前快照、events 与 Outbox，ACK 只推进内存 checkpoint。软限合并只读展示刷新；硬限完成当前动作与 Observe 后禁止下一副作用并 HandBack。投影失败可重试相同事件批次，但不能重试 Driver。

schema 22 增加 `task_runtime_projections` 幂等指纹表。一个批次对 `tasks` 快照、`events`、`outbox` 和运行时指纹的写入必须同事务；重复投影只接受逐事件 payload 完全一致的连续前缀。投影通知为有界且可合并，队列已满时 reducer 不等待 SQLite，projector 依靠 dirty task 重试并在成功后 ACK。

## 失败与验证

CUA `computer.step`、EX-S2 `task.plan.execute`、BUA、Document/Office、Command 和控制链路都使用同一 Runtime Port：Driver 生命周期不持有 TaskHost/SQLite 锁，阶段错误作为类型化事件即时投影。活动 `task.get/events/list` 读取 runtime 并拼接或覆盖尚未 checkpoint 的连续状态；历史查询继续读取 SQLite。`task.list` 先取得有界 SQLite 页，再只对页内同任务应用更高或相同序号的 Runtime 快照，随后按请求的 `include_finished/running_only` 过滤，避免旧 running 泄漏；不跨页注入未在持久页中的新任务，以保持游标稳定。发送槽位仍在确认前 HandBack。迁移可分提交，但不保留长期 legacy 同步执行路径。

Projector 对每个脏任务保留待处理标记，只有连续事件事务提交并 ACK 后才移除；失败必须按有界间隔重试并输出不含任务正文的分类诊断。正常关闭先停止接收新投影，再在有界期限内排空现有 pending；超时保留明确诊断，不允许把 SQLite 写重新放回 Driver/Observe 同步返回路径。重启验证必须同时检查终态、事件连续性和 Runtime pending 已归零。
