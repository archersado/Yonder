## ADDED Requirements

### Requirement: 任务分配输入输出契约

系统 SHALL 在新协议任务创建时冻结有界输入摘要和预期输出，并保持幂等一致。

#### Scenario: 创建与重试
- WHEN归属 Agent 创建任务并提供有效摘要
- THEN任务、创建事件、Outbox 与 I/O 契约一起提交
- AND同键不同契约拒绝，旧任务不从 description 回填

### Requirement: 有界交付摘要

系统 SHALL 在目标核验 achieved 后接受归属 Agent 的有界交付摘要。

#### Scenario: 完成任务
- WHEN归属 Agent 使用最新序号、有效核验 ID 和交付摘要完成任务
- THEN终态、事件、Outbox 与结果摘要按统一运行时顺序提交或投影
- AND摘要不替代目标核验、用户确认或产物可用性

### Requirement: 可信能力产物登记

系统 SHALL 只把可信能力成功产生的输出登记为任务产物。

#### Scenario: 文件或文档成功输出
- WHEN File 新建/替换或 Document 另存已成功并 Observe 有效
- THEN Application 生成任务绑定 opaque 引用并发布下一版不可变清单
- AND Agent、UI、Driver不能提交路径或自报产物引用

#### Scenario: 没有文件产物
- WHEN动作失败、unknown、只读、回收或能力本身不产生文件
- THEN不生成假清单，Task Space 通过交付摘要表达任务结果

### Requirement: 最小化详情展示

系统 SHALL 只展示有界 I/O 与交付摘要。

#### Scenario: 查看任务详情
- WHEN用户打开新任务或旧任务详情
- THEN核心详情立即显示摘要或明确缺失状态，不等待产物分页
- AND不显示 description、完整 Agent Payload、截图、输入、路径或完整命令输出
