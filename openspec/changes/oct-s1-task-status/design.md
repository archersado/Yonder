当前归属 Story：TM-S1；规划：`docs/specs/epic-TM/story-TM-S1/README.md`。旧编号保留历史追溯。

状态：Frozen。下文是已实施批次与当时候选设计的历史记录；新协议、持久化、身份、宿主或UI变更必须进入对应模块Story与新OpenSpec，不从本文继续Apply。

# 设计

当前归属增量依 AD-OCT-04：新库 schema_version=2，创建归属取自可信 AuthContext，查询校验绑定身份并按 owner_agent_id 过滤；LocalUser 可作全量总览。v1 拒绝且保留，迁移另建 Change。下文无归属和 schema_version=1 描述保留为先前实现批次记录，现行验证见 verification-task-ownership.md。

task.list 核心增量已按 AD-OCT-02 完成：使用已有 tasks 主键和绑定参数，最多读取 limit+1 行；Application 返回截断后的快照和可选下一游标，无 schema 迁移。当前 Store 尚无任务归属，查询只供可信本机调用方限定存储实例使用，不向外部 Agent 开放。详细验证见 verification-task-list.md。

多任务增量依据 AD-OCT-03（Proposed），Architecture Impact：architecture-change，扩展查询协议和资源准入语义，不改变 SQLCipher 事实源及依赖方向。先补 AD-OCT-02 的 task.list 详细契约，再实现 Rust 类型、生成物、Application/Adapter 列表查询、资源准入与桌面总览。前端展示授权任务快照，不拥有执行队列；按各任务 sequence 接收更新，不将其当作全局游标。桌面实施仍受 AD-E0-01 门禁约束。

CI 增量遵循既有架构围栏，Architecture Impact：conforming，不改变协议、持久化或技术栈。使用 Cargo metadata 检查全部正式 Workspace 成员的普通、开发、构建及条件依赖；Domain 禁止外部依赖，其他成员遵循既有层级。Windows/macOS 执行锁定依赖的 Workspace 测试与协议生成检查。PR 正文明确关联一个 Story、一个 OpenSpec Change 和仓库内验证记录，检查文件存在及双向引用。库层 CI 不代替原生 UI/Driver/权限 E2E；后续原生实现仍须补充证据门禁。

只读协议遵循 AD-OCT-02：Rust 唯一定义 JSON-RPC task.get/task.events 请求、快照、事件和响应；生成 JSON Schema/TypeScript 并提供只读漂移检查。Application 查询分派仍从 TaskStore 读取，不新建缓存。64 KiB 入口限制、严格字段、元数据/截止时间、分页及十进制序号校验在查询前完成。此阶段不开放网络，尚无身份认证和 hello 协商。

Domain 接收数据库读取出的任务状态和当前 sequence，纯计算新的状态与事件，不保存全局状态。Application 后续负责事务边界和 expected_sequence 并发检查；成功提交后才能对外发布事件。

终态 completed/failed/cancelled 禁止迁出；paused/waiting-for-user/interrupted 仅接受显式 Resume 才进入 running。进程恢复只将 running 转为 interrupted；动作 unknown 留待执行器 Story 定义，不推断成功。

此批不改变传输协议或数据库格式，不创建空的 Adapter、UI 或 CLI。

Application 的 TaskStore Port 只接受原子 compare-and-commit：Adapter 必须在一个事务中校验 expected_sequence，更新状态、插入事件和 Outbox。Application 不缓存快照、不直接发布事件，提交失败只返回错误。事件分页上限 100；after_sequence 为排他游标。Port 返回领域数据供后续协议层转换，不作为第二套网络协议类型。

存储实施遵循 AD-OCT-01：新库初始化 schema_version=1；已有库只接受该版本。tasks/events/outbox 同事务，Outbox 引用不可变事件。创建接口只创建 created 任务，初始序号 1；外部 Gateway 幂等键留待协议实现，当前接口不得直接作为外部 API。

启动恢复遵循 AD-OCT-01 启动恢复补充：TaskStore 按 id 稳定排序读取最多 100 个 running 快照；Application 使用现有 Interrupt 迁移和 CAS 逐任务提交。不在 open 自动恢复。宿主获得单实例所有权后、对外服务前重复调用至返回 0；任何失败阻断启动，不自动重试动作。中途失败允许之前任务已恢复，下次显式恢复不重复产生这些任务的事件。当前仅交付用例与数据库验证，宿主接线未完成。

当前资源准入增量遵循 AD-OCT-06：Application 互斥锁内有界原子分配资源，凭证显式停止确认后释放，用户接管阻断新桌面准入。资源占用不是任务事实源；不接数据库或执行器，不新增协议。密钥相关工作按用户要求暂停；环绕菜单仅保留规格。

准入后启动补充：admission.start 复用 try_acquire 与 transition(Start)，先获取资源后提交事务，成功返回 Task/Permit；失败释放未派发占用，不影响其他任务。验证见 verification-admitted-start.md；仍不派发真实动作。

结束用例补充：Permit.finish_after_stop 仅接收确定的 Completed/Failed，由凭证绑定 task_id，先提交终态事务再释放；状态失败返回凭证，释放失败明确返回已提交 Task。详见 AD-OCT-06 和 verification-admitted-finish.md。
