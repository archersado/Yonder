# 任务状态

## Requirement：只读查询协议

系统 SHALL 使用 AD-OCT-02 的 JSON-RPC 2.0 只读契约；Rust 类型是 Schema 和 TypeScript 唯一来源。

### Scenario：请求校验

- **WHEN** task.get/task.events 携带有效 id、agent_id、task.read、未过期 deadline 和 task_id
- **THEN** 从 TaskStore 查询并保留响应 id，events 采用排他序号游标和 1..100 条上限
- **AND** 拒绝超长请求、未知字段、缺失元数据、非法 ID、非规范序号和过期请求

### Scenario：精度和生成一致性

- **WHEN** 序号大于 JavaScript 安全整数上限
- **THEN** JSON 使用规范十进制字符串，不损失精度
- **AND** Rust 类型变更而生成产物未更新时，生成检查失败

## Requirement：状态与事件

系统 SHALL 只接受合法状态迁移，每个实际迁移的 sequence 增加一；拒绝操作不得产生状态或事件。

### Scenario：终态保护

- **WHEN** 已完成、失败或取消的任务收到启动或恢复请求
- **THEN** 拒绝请求，原状态与序号不变

### Scenario：重启恢复

- **WHEN** 重启恢复读取到 running 任务
- **THEN** 生成 interrupted 迁移，不重新执行动作
- **AND** 每批最多处理 100 个任务，每任务状态、事件和 Outbox 同事务，序号只增加一
- **AND** 其他状态不变；重复恢复不追加重复事件

### Scenario：恢复中途失败

- **WHEN** 恢复一批任务时某任务的 Outbox 插入失败
- **THEN** 该任务三表更新回滚并返回错误，之前已恢复的任务保持 interrupted
- **AND** 再次显式恢复只处理剩余 running 任务，不重放动作
- **AND** 普通数据库 open 不触发恢复；生产宿主必须在单实例启动阶段完成恢复后才开放执行（宿主接线待实现）

### Scenario：显式继续

- **WHEN** 外部 Agent 在重新观察后显式恢复 interrupted 任务
- **THEN** 生成 running 迁移和下一事件序号

## Requirement：持久化事务

系统 SHALL 在同一 SQLCipher 事务内创建或更新任务、追加事件并写入 Outbox。失败不得返回成功快照。

### Scenario：读取后发生竞争更新

- **WHEN** 两个连接读取同一任务序号，首个连接提交成功
- **THEN** 第二个连接携带旧序号提交时返回 Conflict
- **AND** 不覆盖首个连接写入的状态，不产生重复事件

### Scenario：Outbox 写入故障

- **WHEN** 状态更新和事件插入后 Outbox 写入失败
- **THEN** 三张表全部回滚，其他连接仍读取之前已提交状态

### Scenario：错误密钥与未知版本

- **WHEN** 使用错误数据库子密钥或读取未知 schema_version
- **THEN** 打开失败，不重建、不清空、不升级原文件
