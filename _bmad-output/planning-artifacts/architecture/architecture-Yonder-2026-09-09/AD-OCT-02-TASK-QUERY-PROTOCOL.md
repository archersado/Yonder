# AD-OCT-02 任务只读协议

未定案建议见 AD-TM-01-TASK-METADATA.md（Proposed）：列表/详情、元数据事件和版本策略尚待需求对齐，不是当前决定。现有 1.0 与派生协议保持原样，不作为已发布兼容保证。

状态：Accepted（OCT-S1 只读契约范围，非 Gateway/IPC 技术门禁通过）。

2026-09-11 归属增量依 AD-OCT-04：快照增加 owner_agent_id；进程内分派必须接收可信 AuthContext，并校验请求 agent_id 与其匹配。get/events 统一拒绝他人任务，list 在分页前过滤归属；下述“尚无归属/认证约束”对应历史批次，真实外部认证与传输仍未接线。

采用 JSON-RPC 2.0，首批仅 `task.get`、`task.events`。Rust 字段 request_id 在 wire 上映射为标准 `id`，不再另设重复字段。只接受字符串请求 ID，不接受通知或批量请求。params 必须携带 agent_id、capability=`task.read`、deadline（Unix 毫秒整数）以及 task_id；events 另带 after_sequence 与 limit（1..100）。截止时间必须晚于校验时钟，不超过 JavaScript 安全整数上限。请求最多 64 KiB；未知字段、缺失元数据、非规范 ID 拒绝。ID 沿用 ASCII 字母/数字/下划线/短横线、1..128 字节。

任务序号在 JSON 中使用十进制字符串，避免 TypeScript number 精度损失；无符号、无前导零（0 除外），有效范围 0..i64::MAX，持久化快照仍从 1 开始。Rust 内部序号继续使用 u64。当前返回状态/序号及状态迁移事件，步骤、观察和意图尚待后续增量，不宣称满足 OCT-S1 全部 AC。

协议 crate 不依赖 Domain/Application/Adapter。Application 显式映射领域状态为协议状态；这不是第二套传输模型。Rust 类型使用 serde 序列化，schemars 生成 JSON Schema，ts-rs 生成 TypeScript；锁定版本，生成物只可通过生成程序更新。Schema 表达静态结构，截止时间及序号上界等语义由 Rust 入口校验。选择生成库而非维护自制转换器：[Schemars](https://docs.rs/schemars/1.2.2/schemars/)、[ts-rs](https://docs.rs/ts-rs/12.0.1/ts_rs/)。

本次只交付进程内查询分派，不创建 Socket、HTTP/TCP 或云端服务。agent_id 是调用声明，不是身份凭据；未来 Gateway 必须将 OS/云端认证结果绑定 AuthContext 后才调用用例。hello 版本/capability 协商、取消、创建幂等键、传输帧和权限校验仍待实现，不可将当前分派直接暴露给外部。

## 2026-09-11 task.list 进程内增量

决定：Accepted（仅可信本机调用方提供的存储实例内查询；不代表多 Agent 授权或并行执行已交付）。关联 AD-OCT-03、OCT-S1。

增加 task.list，沿用 id、agent_id、task.read、deadline。params 增加 limit（必需，1..100）、after_task_id（可省略/null；非空时遵循已有 ID 格式）、include_finished（可省略，默认 false；只接受布尔值）。默认排除 completed/failed/cancelled，保留所有其他状态；true 返回全部状态。

按不可变 task_id 的 SQLite BINARY 升序采用排他游标分页，不用 OFFSET。每次有界读取最多 limit+1 行判断是否有下一页；响应 kind=tasks、tasks 快照数组、next_after_task_id（最后返回项 ID，有后续项时才非 null）。游标不要求对应任务仍存在。不返回未经计算的总数或全局 sequence。

每次查询读取当时已提交的状态，不承诺跨页快照一致性；游标前新建任务或状态筛选变化需从首页刷新。任务更新仍按各自 sequence 判断，不跨任务比较。SQL 参数绑定，不拼接用户提供的游标。

当前 schema 没有任务所有者，故本增量不做虚假的 agent_id 过滤，也不开放 Agent 传输入口。可信调用方必须控制存储实例的授权范围；未来 Gateway 接线前，须先定义并实现 AuthContext/任务归属约束。存储 schema_version 仍为 1，无迁移、新索引或状态缓存。
