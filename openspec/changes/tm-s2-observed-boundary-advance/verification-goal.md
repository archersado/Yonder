# Verification Goal：TM-S2 连续步骤边界

状态：通过（macOS，2026-09-16）。

在真实SQLite上验证两次attempt连续准备与Observe、普通边界不释放Permit、pending控制/unknown/旧身份拒绝、事件和Outbox严格递增，以及故障时全事务回滚。Windows按用户要求暂缓。

## 结果

- Workspace 40项测试通过；新增真实SQLite样本连续完成两次prepare → result → advance，期间Browser Permit保持占用，结束任务后才释放。
- Outbox触发器故障使普通边界全事务回滚；移除故障后重试只产生一个边界事件。pending控制与unknown均拒绝推进。
- 正式任务库由schema10迁移到11，迁移前后tasks/events/outbox/attempts/controls计数均为15/31/31/0/0，`foreign_key_check`为空。
- 正式Yonda以单进程PID 17862运行；迁移备份已生成，结构化证据见`apps/desktop/evidence/observed-boundary-20260916/result.json`。

本Goal只完成连续步骤共同前置；Gateway动作、Supervisor持有Permit和BUA外部引用持久化仍属后续Change。Windows暂缓。
