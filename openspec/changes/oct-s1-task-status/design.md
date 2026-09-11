# 设计

只读协议遵循 AD-OCT-02：Rust 唯一定义 JSON-RPC task.get/task.events 请求、快照、事件和响应；生成 JSON Schema/TypeScript 并提供只读漂移检查。Application 查询分派仍从 TaskStore 读取，不新建缓存。64 KiB 入口限制、严格字段、元数据/截止时间、分页及十进制序号校验在查询前完成。此阶段不开放网络，尚无身份认证和 hello 协商。

Domain 接收数据库读取出的任务状态和当前 sequence，纯计算新的状态与事件，不保存全局状态。Application 后续负责事务边界和 expected_sequence 并发检查；成功提交后才能对外发布事件。

终态 completed/failed/cancelled 禁止迁出；paused/waiting-for-user/interrupted 仅接受显式 Resume 才进入 running。进程恢复只将 running 转为 interrupted；动作 unknown 留待执行器 Story 定义，不推断成功。

此批不改变传输协议或数据库格式，不创建空的 Adapter、UI 或 CLI。

Application 的 TaskStore Port 只接受原子 compare-and-commit：Adapter 必须在一个事务中校验 expected_sequence，更新状态、插入事件和 Outbox。Application 不缓存快照、不直接发布事件，提交失败只返回错误。事件分页上限 100；after_sequence 为排他游标。Port 返回领域数据供后续协议层转换，不作为第二套网络协议类型。

存储实施遵循 AD-OCT-01：新库初始化 schema_version=1；已有库只接受该版本。tasks/events/outbox 同事务，Outbox 引用不可变事件。创建接口只创建 created 任务，初始序号 1；外部 Gateway 幂等键留待协议实现，当前接口不得直接作为外部 API。

启动恢复遵循 AD-OCT-01 启动恢复补充：TaskStore 按 id 稳定排序读取最多 100 个 running 快照；Application 使用现有 Interrupt 迁移和 CAS 逐任务提交。不在 open 自动恢复。宿主获得单实例所有权后、对外服务前重复调用至返回 0；任何失败阻断启动，不自动重试动作。中途失败允许之前任务已恢复，下次显式恢复不重复产生这些任务的事件。当前仅交付用例与数据库验证，宿主接线未完成。
