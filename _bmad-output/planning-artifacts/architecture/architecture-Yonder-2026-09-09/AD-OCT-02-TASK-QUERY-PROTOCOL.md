# AD-OCT-02 任务只读协议

状态：Accepted（OCT-S1 只读契约范围，非 Gateway/IPC 技术门禁通过）。

采用 JSON-RPC 2.0，首批仅 `task.get`、`task.events`。Rust 字段 request_id 在 wire 上映射为标准 `id`，不再另设重复字段。只接受字符串请求 ID，不接受通知或批量请求。params 必须携带 agent_id、capability=`task.read`、deadline（Unix 毫秒整数）以及 task_id；events 另带 after_sequence 与 limit（1..100）。截止时间必须晚于校验时钟，不超过 JavaScript 安全整数上限。请求最多 64 KiB；未知字段、缺失元数据、非规范 ID 拒绝。ID 沿用 ASCII 字母/数字/下划线/短横线、1..128 字节。

任务序号在 JSON 中使用十进制字符串，避免 TypeScript number 精度损失；无符号、无前导零（0 除外），有效范围 0..i64::MAX，持久化快照仍从 1 开始。Rust 内部序号继续使用 u64。当前返回状态/序号及状态迁移事件，步骤、观察和意图尚待后续增量，不宣称满足 OCT-S1 全部 AC。

协议 crate 不依赖 Domain/Application/Adapter。Application 显式映射领域状态为协议状态；这不是第二套传输模型。Rust 类型使用 serde 序列化，schemars 生成 JSON Schema，ts-rs 生成 TypeScript；锁定版本，生成物只可通过生成程序更新。Schema 表达静态结构，截止时间及序号上界等语义由 Rust 入口校验。选择生成库而非维护自制转换器：[Schemars](https://docs.rs/schemars/1.2.2/schemars/)、[ts-rs](https://docs.rs/ts-rs/12.0.1/ts_rs/)。

本次只交付进程内查询分派，不创建 Socket、HTTP/TCP 或云端服务。agent_id 是调用声明，不是身份凭据；未来 Gateway 必须将 OS/云端认证结果绑定 AuthContext 后才调用用例。hello 版本/capability 协商、取消、创建幂等键、传输帧和权限校验仍待实现，不可将当前分派直接暴露给外部。
