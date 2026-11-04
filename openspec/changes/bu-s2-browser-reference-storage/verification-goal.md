# Verification Goal：BU-S2 Browser引用事实

状态：PASS（macOS 2026-09-16；Windows补证 2026-10-25）。

在真实SQLite与macOS ego-lite Bridge结果上验证原子写入、finish、unknown不覆盖、Outbox故障回滚、schema11迁移和重启读取。Windows按用户要求暂缓。

## 结果

- 真实ego-lite空间`ego:41`创建后与observed attempt写入SQLite，关闭并重开数据库仍读取相同引用；finish写入`finished=true`并清理空间。
- Outbox故障时attempt、事件和引用全部回滚；后续unknown只更新attempt，不覆盖最后成功引用。
- Workspace 41项测试通过，架构与关联检查通过。
- 正式任务库schema11→12后tasks/events/outbox保持15/31/31，`task_browser_refs=0`符合尚无正式BUA任务，外键检查为空；Yonda单实例PID 20795。
- 证据：`apps/desktop/evidence/browser-reference-20260916/result.json`。

本Goal完成持久化门禁；Agent Gateway Browser动作与Supervisor资源持有由后续Change接入。

2026-10-25 Windows补证：正式桌面进程完成真实ego-lite生命周期后退出并重启；同一`task_id`的finished引用与`ego:8`外部引用可从SQLite恢复，未自动重开或接管空间。由正式Named Pipe查询读取，不以Adapter单测替代。证据见`apps/desktop/evidence/bu-s2-windows-20261025/result.json`。
