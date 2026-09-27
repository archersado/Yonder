## ADDED Requirements

### Requirement: Agent 命令提议必须等待本机批准

归属且已登记的 Agent 可以为其 `created` 任务提交有界结构化命令提议。系统 MUST 将完整提议仅保存于有界内存，返回不透明 `command_id` 与等待状态；在本机用户批准前不得启动任务、创建 attempt、调用 Command Runtime，或写入 SQLite、事件、Outbox、日志。

#### Scenario: 有效提议不会产生副作用

- **WHEN** 归属 Agent 为 `created` 任务提交合法 program、args、cwd、env 和 timeout
- **THEN** 系统返回唯一 `command_id` 与 `awaiting_user`
- **AND** 任务保持 `created`，不存在新的 attempt、事件或 Outbox
- **AND** Command Runtime 未被调用

#### Scenario: 非归属 Agent 不能创建或读取提议

- **WHEN** 非归属 Agent、已撤权 Agent 或未登记会话提交或查询该任务的命令提议
- **THEN** 系统拒绝请求
- **AND** 不返回命令内容、批准状态或可消费引用

### Requirement: 本机批准精确绑定命令和任务

只有可信本机用户可以在 Task Space 预览并批准或拒绝提议。系统 MUST 将批准绑定 task、owner Agent、完整命令 SHA-256 摘要、当前 sequence 和一次性 `command_id`，最长有效五分钟；完整命令不得经 Gateway、事件、Outbox、日志或桌宠暴露。

#### Scenario: 参数替换要求重新批准

- **WHEN** Agent 以不同的 program、args、cwd、env 或 timeout 请求执行
- **THEN** 原 `command_id` 不可用于该请求
- **AND** 系统要求创建新的本机批准提议

#### Scenario: 拒绝或过期后不能执行

- **WHEN** 用户拒绝、批准过期、任务终态、Agent 撤权/断连或 TaskHost 重启
- **THEN** 对应 `command_id` 失效
- **AND** 后续执行请求失败关闭且 Command Runtime 未被调用

### Requirement: 执行必须先进入统一任务启动事务

Agent 执行请求只可携带 `task_id`、`expected_sequence` 和已批准的 `command_id`。系统 MUST 先按 TM-S7 提交统一启动事务，再原子消费批准并调用 AD-CM-01 Command Runtime；执行结果必须按既有 attempt observed/unknown 语义记录，unknown 不自动重试。

#### Scenario: 已批准命令只执行一次

- **WHEN** 归属 Agent 以匹配的 sequence 执行已批准 `command_id`
- **THEN** 系统先提交 `created→running`、步骤、attempt、事件和 Outbox
- **AND** 原子消费批准后调用一次 Runtime
- **AND** 相同 `command_id` 的重放被拒绝

#### Scenario: 启动事务失败不消费批准或派发

- **WHEN** TM-S7 启动事务因 CAS、准入或 Outbox 失败而不能提交
- **THEN** 系统不调用 Runtime
- **AND** 批准不被消费
- **AND** 不产生半提交任务事实
