# AD-TM-23 事件驱动执行运行时与事后持久化

状态：Accepted（2026-09-29，用户明确废除“SQLite 是当前状态唯一事实源、每步同步事务后推进”约束）。关联 TM-S9、TM-S2、TM-S3、TM-S7、TM-S8、EX-S2。Architecture Impact：architecture-change（活动状态所有者、持久化时序、Gateway 查询与恢复）。

## 被覆盖的旧约束

自本决定接受起，架构主干及既有 ADR、Story、OpenSpec 中以下要求不再适用：

- SQLite 当前状态表是活动任务的唯一当前事实源；
- 每一步必须等待状态、事件、Outbox 同事务提交后才能推进下一步；
- UI、Gateway 或执行协调必须回读 SQLite 才能判断活动任务当前状态。

历史文档保留追溯，但与本决定冲突时以本决定为准。落盘批次内部的状态、事件、Outbox 原子一致性仍保留，不能据此写出相互矛盾的历史。

## 决定

- Application 内单一有界 `ExecutionRuntime` 事件循环是活动任务当前状态的唯一事实源，持有当前步骤、attempt、计划游标、租约、控制请求、Observe 与严格递增的逻辑序号。
- Gateway 命令、Driver 结果、Observe、用户控制和 UI 投影都通过类型化 command/event 进入同一 reducer；动作完成并 Observe 后即可推进或 HandBack，不等待 SQLite。
- SQLite、events 和 Outbox 是内存事件流的异步事后投影及重启检查点。单一 projector 按连续序号批量写入；确认只推进 checkpoint，不得反写覆盖更晚的内存状态。
- 每任务和全局队列均有界。达到硬限时完成已开始动作及 Observe，然后在下一副作用前暂停并暴露 `persistence-backpressure`；不得丢弃结果、无限占内存或让数据库故障卡住当前 Driver。
- Gateway 对活动任务读取 runtime；历史/非活动任务读取 SQLite。同一任务不得同时存在两个可写状态所有者。
- 崩溃或断连时未确认副作用仍为 unknown 且禁止自动重试。重启只从最后完整 checkpoint 恢复，并把遗留 running 转 interrupted，不猜测未落盘结果。
- 发送、支付、删除、安装和提权仍须副作用前取得本机确认；异步持久化不得把确认改成事后补记。
- 不引入完整 Event Sourcing、第二数据库、本地 HTTP 或通用消息总线。使用进程内有界 channel、单一 runtime owner 和单一 persistence projector。

## 迁移与验证

实现可按能力分阶段提交，但完成口径同时覆盖 CUA、BUA、Document/Office 与 Command；任何一条仍由同步 SQLite 写入驱动时，本 Story 不得标记完成或 Archive。统一验证 SQLite 延迟/失败不阻塞 Driver→Observe→下一步、即时顶部投影、背压暂停、取消/接管、HandBack、崩溃恢复与发送确认。macOS 先行，Windows继续暂缓。
