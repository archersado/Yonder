# Agent 步骤声明增量

## ADDED Requirements

### Requirement: Agent 声明未开始任务步骤

系统 MUST 只允许归属Agent在created任务上以当前序号原子提交不可变步骤声明。

#### Scenario: 首次声明
- **GIVEN** 归属 Agent 已协商协议 1.4，任务为 `created`
- **WHEN** 以当前序号调用 `task.step.declare`
- **THEN** 任务序号、不可变声明、同状态事件与 Outbox 同事务提交，状态仍为 `created`

#### Scenario: 幂等与门禁
- **WHEN** 相同 `task_id + step_id + label` 重试
- **THEN** 返回既有声明和当前快照，不增加记录；不同标签返回 `-32013`
- **WHEN** 未握手、旧版本、身份不符、越权、LocalUser 写入、过期或非法参数
- **THEN** 拒绝且不写库

#### Scenario: 状态与配额
- **WHEN** 首次声明的任务不再是 `created`、序号冲突或声明达到配额
- **THEN** 分别返回停止契约、序号冲突或配额错误，原事实不变

### Requirement: 读取声明与兼容事件

系统 MUST 返回当前声明，并按会话协议版本投影兼容事件。

#### Scenario: 当前声明
- **WHEN** 1.4 Agent 或可信本机调用 `task.step.get`
- **THEN** 从同一读取事务返回任务快照及最近声明；无声明为 `null`

#### Scenario: 新旧事件投影
- **WHEN** 1.4 会话读取事件
- **THEN** 声明事件包含 `step_declaration`
- **WHEN** 1.0～1.3 会话读取相同事件
- **THEN** 保留连续事件序号并移除声明字段

### Requirement: 安全迁移

系统 MUST 只对可安全判定格式的明文库执行步骤能力迁移，并保留全部历史事实。

#### Scenario: 明文升级
- **WHEN** schema 5 或可安全升级的旧明文库启用步骤能力
- **THEN** 先备份再事务迁移到 schema 6，旧任务和历史原样保留
- **WHEN** 库为已有加密 schema 5、危险或未知格式
- **THEN** 拒绝迁移且不修改原库
