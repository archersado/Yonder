# Computer Gateway Delta Spec

## ADDED Requirements

### Requirement: Agent通过统一Gateway执行CUA

系统 MUST 由Yonder生成执行身份、解析原生目标并取得唯一桌面租约；Agent MUST NOT 提交PID、窗口号或Driver路径。

#### Scenario: SDK原生工具桥接

- **WHEN** 归属Agent声明步骤并提交SDK公布的tool name与arguments
- **THEN** Yonder拒绝受保护目标字段，注入可信目标，经固定SDK `callTool`执行、强制Observe，并把结果写入同一任务时间线

#### Scenario: SDK动作契约单一来源

- **WHEN** SDK新增、删除或修改动作
- **THEN** Yonder读取`listToolsJson()`并使用SDK校验，不维护第二份动作枚举或参数Schema

#### Scenario: SDK桌面作用域原样桥接

- **WHEN** Agent调用SDK已声明支持的动作并提交`scope: desktop`
- **THEN** Gateway保留该SDK参数且Worker不注入窗口PID或窗口号
- **AND** 动作及参数合法性仍由SDK Schema决定，Yonder不复制作用域动作模型

#### Scenario: 连续动作使用统一执行会话

- **WHEN** 同一宿主在没有unknown或用户接管边界时连续执行多个CUA动作
- **THEN** 所有动作经同一个受监管trycua Driver会话串行执行，不切换原生动作执行器
- **AND** 每个动作仍独立Observe并写入对应attempt结果

#### Scenario: 异常边界终止执行会话

- **WHEN** 用户输入、超时、Worker或SDK异常使动作结果未知
- **THEN** Yonder终止当前trycua会话且不自动重试，后续动作须重新Observe并由Agent决策

### Requirement: 单次往返执行CUA步骤

系统 MUST 将合法步骤声明、动作派发、强制Observe和普通边界推进合并在一次`computer.step`往返内完成。

#### Scenario: 单次声明执行并推进

- **WHEN** Agent提交步骤标识、展示名称、SDK工具名与参数
- **THEN** Yonder在一次`computer.step`调用内完成声明、动作、强制Observe与普通边界推进
- **AND** MCP默认工具列表不要求Agent分别调用declare、execute和advance

### Requirement: Observe证据由Yonder提供

系统 MUST 为已完成的SDK动作提供有界后置观察证据，且不持久化截图或完整Driver Payload。

#### Scenario: 返回有界观察证据

- **WHEN** trycua动作完成且后置桌面Observe有效
- **THEN** 响应返回紧凑动作结论、元素数量及可用截图的MIME与本地只读路径
- **AND** 截图不超过4MiB，不进入日志、SQLite事件或Outbox，并在任务完成或异常会话结束时清理

### Requirement: 用户输入优先

系统 MUST 让真实用户输入优先于Agent CUA动作，并进入保守中断边界。

#### Scenario: 用户输入中断任务

- **WHEN** CUA动作期间出现新的真实用户键鼠输入
- **THEN** 系统终止Worker、记录unknown并中断任务、释放桌面输入租约，且不自动重试或恢复

### Requirement: 显式完成

系统 MUST 只在Agent显式请求完成且最后一个步骤已Observe推进后提交completed。

#### Scenario: 最后步骤显式完成

- **WHEN** Agent已推进最后一个Observed步骤并显式请求完成
- **THEN** 系统原子提交completed后释放任务资源
