# 设计

Gateway 1.4 增加 `task.step.declare` 与 `task.step.get`。会话必须完成 1.4 握手且 Adapter 声明支持；身份由组合根绑定。Application 校验 Agent、任务归属和输入，SQLite `IMMEDIATE` 事务负责幂等、CAS、配额以及任务序号/声明/事件/Outbox 原子提交。

`task_id + step_id` 同标签重试返回既有声明与当前快照；不同标签返回冲突。首次声明只允许 `created` 且序号匹配。事件序号连续、状态保持 `created→created`；1.4 事件带可选声明，旧会话保留事件但移除该字段。读取任务与最近声明使用同一数据库事务。

schema 5 及可安全升级的旧明文库先生成 SQLite 一致备份再迁移到 6；已有加密 schema 5、危险或未知格式拒绝且不覆盖。每任务 1024、全库 10000 条上限，不自动清理。
