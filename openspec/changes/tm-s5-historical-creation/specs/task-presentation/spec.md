# task-presentation Delta

## ADDED Requirements

### Requirement: 创建来源历史只来自已提交事实

Yonder MUST 只把与创建事件同序号提交的 source payload 投影为创建来源历史，并保留可信组合根绑定的归属 Agent；MUST NOT 从当前快照为缺失历史的旧任务补造事实。

#### Scenario: 新任务创建来源可追溯

- **GIVEN** 可信组合根创建本地或云端来源任务
- **WHEN** 创建事务提交事件、来源 payload 和 Outbox
- **THEN** 同一创建序号包含相应来源与归属 Agent

#### Scenario: 旧任务缺少 source 历史

- **GIVEN** 迁移任务当前来源为 legacy 但没有 source payload
- **WHEN** 查询历史
- **THEN** 创建事件没有 `creation_event`
- **AND** 系统不从当前快照回填

### Requirement: 创建来源历史按协议和授权隔离

Yonder MUST 只对协议 1.24 的已授权读取输出 `creation_event`，并保持既有连续性、分页和编码预算；1.23 及以下 MUST 缺省该字段。

#### Scenario: 旧协议读取创建事件

- **GIVEN** 同一任务存在已提交来源历史
- **WHEN** 协商协议 1.23
- **THEN** 事件仍返回但不包含 `creation_event`

#### Scenario: 非归属 Agent 查询

- **GIVEN** 调用 Agent 不拥有该任务
- **WHEN** 查询事件
- **THEN** 返回不可见错误
- **AND** 不读取或泄露来源历史

### Requirement: Task Space 明确显示创建来源

Task Space MUST 将创建来源显示为来源类别和 Agent 标识，MUST NOT 显示描述、幂等键、凭据或完整 Agent Payload，也 MUST NOT 把创建归属表示为当前仍有权限。

#### Scenario: 查看创建事件

- **GIVEN** `creation_event` 来源为 local-agent
- **WHEN** 用户查看时间线
- **THEN** 显示“任务创建：本地 Agent”与 Agent 标识
- **AND** 不提供冒充或 Payload 入口
