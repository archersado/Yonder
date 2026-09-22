# Agent Input Delta

## ADDED Requirements

### Requirement: 定向自动投递

Yonder MUST 把显式用户入口输入定向投递给交互开始时绑定的可接收Agent会话。

#### Scenario: 投递到绑定会话

- **WHEN** 显式用户入口产生最终非空输入
- **THEN** Yonder只向交互开始时绑定且声明`user_input`能力的Agent会话投递
- **AND** 不显示语音发送确认页

### Requirement: 本地与云端统一会话

Gateway MUST 对本地和云端Agent Runtime使用同一会话语义与`agent.input`消息。

#### Scenario: 本地或云端投递

- **WHEN** Agent Runtime通过本地UDS/Named Pipe或云端主动WSS连接Yonder
- **THEN** Gateway使用同一`AgentSession`、`agent.input`消息和accepted语义
- **AND** 不按传输类型、Agent厂商或运行位置分叉Application用例
- **AND** 云端Agent Runtime直接通过现有WSS双工信道接收和确认输入，不经过额外连接器

### Requirement: 多Agent确定性路由

Yonder MUST 以显式任务归属或用户激活确定唯一投递目标，不在交互中途切换。

#### Scenario: 多候选要求选择

- **WHEN** 输入来自某个任务上下文
- **THEN** Yonder绑定任务归属Agent的当前会话
- **WHEN** 桌宠独立入口只有一个可接收输入的会话
- **THEN** Yonder自动激活并绑定该会话
- **WHEN** 存在多个候选且没有显式激活会话
- **THEN** Yonder返回`target_required`并在收音前要求选择
- **AND** 交互开始后不得因连接顺序或活跃变化切换目标

### Requirement: 接收确认

调用方 MUST 只在Agent明确accepted后显示投递成功。

#### Scenario: Agent确认接收

- **WHEN** Agent明确返回accepted
- **THEN** 调用方才显示投递成功
- **WHEN** 断连、拒绝、超时或结果未知
- **THEN** 显示失败且不自动重试或切换Agent

### Requirement: 输入边界与去重

Yonder MUST 限制输入大小和确认期限，并按`input_id`去重。

#### Scenario: 重复input_id去重

- **WHEN** Yonder投递`agent.input`
- **THEN** 正文必须为非空UTF-8且不超过16 KiB，确认期限默认10秒且最大60秒
- **AND** Agent连接器按`input_id`去重，重复请求返回原结果且不再次写入Agent会话

### Requirement: 任务与隐私边界

输入投递 MUST 保持纯用户输入传递，不创建任务或记录正文。

#### Scenario: 正文不落库

- **WHEN** 用户输入被投递
- **THEN** Yonder不调用`task.create`、不进行语义规划
- **AND** 日志、任务SQLite、任务事件和Outbox不记录正文

### Requirement: 桌宠连接状态

桌宠 MUST 实时反映可接收输入的Agent连接状态，并以可访问文本呈现。

#### Scenario: 连接建立更新徽标

- **WHEN** 首个可接收输入的Agent会话建立或最后一个会话断开
- **THEN** 桌宠立即更新Agent连接徽标并唤醒隐藏形态
- **AND** 连接态不得覆盖任务生命周期动画，也不得通过轮询任务数据库获得
- **AND** 已连接与未连接状态必须具有可访问文本
