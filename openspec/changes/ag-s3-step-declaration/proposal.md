# AG-S3 Agent 步骤声明

关联 Story：`docs/specs/epic-AG/story-AG-S3/`；依据 Accepted AD-AG-04。

为后续执行、Observe 和接管建立可信 `step_id`。协议 1.4 允许归属 Agent 为 `created` 任务登记纯文本步骤并读取最近声明；登记不执行动作、不改变任务状态。SQLite 6 将声明、同状态事件及 Outbox 同事务保存。

Architecture Impact：architecture-change（Gateway 协议、Application Port、SQLite schema）。Rust 类型为协议唯一来源；Windows 暂缓，首批通过库合约与私有 stdio Agent 验证。
