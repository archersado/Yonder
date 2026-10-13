# Design：终态异步投影恢复

`RuntimeProjectionWorker` 按任务消费连续 pending。只有 projector 成功提交状态、事件与 Outbox，并由 Runtime 接受 ACK 后，才能移除 dirty；失败保持 dirty 并按既有间隔重试。诊断只记录错误分类与累计次数，不记录任务正文、事件 Payload 或动作参数。

Worker 停止改为两阶段：拒绝新的后台循环后，在有界期限内继续投影已知 dirty 与 wake 队列，直至 pending 为空或到达上限。`TaskHost` 保持 projector 先于 Runtime 析构，正常关闭完成排空；强制进程终止仍由最后成功 checkpoint 恢复为 `interrupted`，不伪造终态。

根因修复依 Accepted AD-TM-04 的 2026-10-13 修订：schema 25→26 重建 `task_attempts` 与引用它的 `task_controls`，允许 stopped attempt 保留此前 prepared/observed/unknown 三种结果形状。`ControlStopped(cancel)` 仍在同一 projector 事务写 task、event、Outbox、control 和 stop 关联；unknown 字段不被改写为 observed，也不宣称副作用回滚。

验证使用真实控制取消形成终态，先等待 projector checkpoint，再重启正式宿主并经 Gateway 读取同一任务。自动化覆盖投影失败后恢复、终态事件、Drop 排空和原子回滚。
