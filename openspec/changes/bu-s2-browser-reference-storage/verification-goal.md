# Verification Goal：BU-S2 Browser引用事实

状态：通过（macOS，2026-09-16）。

在真实SQLite与macOS ego-lite Bridge结果上验证原子写入、finish、unknown不覆盖、Outbox故障回滚、schema11迁移和重启读取。Windows按用户要求暂缓。

## 结果

- 真实ego-lite空间`ego:41`创建后与observed attempt写入SQLite，关闭并重开数据库仍读取相同引用；finish写入`finished=true`并清理空间。
- Outbox故障时attempt、事件和引用全部回滚；后续unknown只更新attempt，不覆盖最后成功引用。
- Workspace 41项测试通过，架构与关联检查通过。
- 正式任务库schema11→12后tasks/events/outbox保持15/31/31，`task_browser_refs=0`符合尚无正式BUA任务，外键检查为空；Yonda单实例PID 20795。
- 证据：`apps/desktop/evidence/browser-reference-20260916/result.json`。

本Goal完成持久化门禁；Agent Gateway Browser动作与Supervisor资源持有仍是下一Change。Windows暂缓。
