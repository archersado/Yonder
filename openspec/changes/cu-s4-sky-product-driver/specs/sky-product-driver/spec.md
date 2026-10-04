# Sky Product Driver Delta

## ADDED Requirements

### Requirement: macOS只使用Sky产品Driver

系统 MUST 在macOS产品组合根只构造固定版本Sky Worker；不得携带或回退trycua。

#### Scenario: Sky依赖不可用

- **WHEN** 固定Sky入口或包身份校验失败
- **THEN** CUA capability为unavailable，且不启动其他CUA Driver

### Requirement: 使用官方签名MCP桥接

系统 MUST 通过固定Sky包提供的官方签名`SkyComputerUseClient mcp`访问Computer Use服务；Yonder不得直接连接服务Socket或伪造OpenAI entitlement。

#### Scenario: Yonder执行Sky动作

- **WHEN** 受监管Worker需要列举应用、Observe或执行动作
- **THEN** 同发行物的OpenAI签名Node运行Worker，Worker只通过同一签名Client的MCP stdio工具调用，并在会话结束时关闭该Client

#### Scenario: Client请求应用使用授权

- **WHEN** Client为当前任务已收敛的唯一`launch_app`目标发出固定空Schema elicitation
- **THEN** Worker只接受该应用的会话访问；目标或Schema不匹配时拒绝，且不由此放行发送等副作用

### Requirement: 新鲜应用级AX元素优先

系统 MUST 对同任务绑定的唯一应用在每步前后读取新鲜AX transcript，并优先使用当前`element_index`执行。

#### Scenario: QQ音乐公开唯一搜索框

- **WHEN** 新鲜transcript包含唯一可操作的搜索文本框
- **THEN** Worker使用该次index执行聚焦或赋值，动作后重新Observe且不走坐标兜底

#### Scenario: 元素缺失或多义

- **WHEN** 语义目标无法从新鲜transcript唯一解析
- **THEN** Worker停止动作并返回同应用有界视觉Observation供慢脑重规划，不自动重试

### Requirement: AX transcript不持久化

系统 MUST 仅在Worker内使用AX transcript；不得写入任务状态、事件、Outbox、日志或顶部浮窗。

#### Scenario: 动作完成

- **WHEN** Worker取得动作前后transcript
- **THEN** 对外Observation只包含元素数量、受控截图引用与可见性，不包含transcript正文
