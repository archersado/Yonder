# CUA Target Intent Delta Specification

## ADDED Requirements

### Requirement: 敏感消息意图必须使用有界内存引用

系统 MUST 允许归属Agent经Gateway提出目标与正文，并只返回不透明意图/确认引用。目标与正文 MUST NOT 进入计划、SQLite、事件、Outbox、日志或Jev请求。

#### Scenario: 提出消息意图

- **WHEN** 归属Agent为活动任务提交合法目标与正文
- **THEN** Application在有界内存中保存并返回两个任务绑定引用，Gateway响应不回显敏感值

### Requirement: 计划必须用封闭语义逐步解析目标

计划 MUST 分别表达聚焦目标入口、输入目标查询、激活目标、聚焦消息输入框、填写草稿和发送。Application MUST 在派发前解析引用，Worker MUST 元素优先。元素不可用时，点击语义可接受慢脑依据1.34视觉证据提交的坐标候选；文本语义 MUST NOT 携带坐标，只能使用唯一元素或消费前一片段在同任务同窗口建立的一次性视觉焦点凭据。

#### Scenario: 元素路径可用

- **WHEN** 同一可信窗口存在唯一匹配元素
- **THEN** Worker使用元素完成当前语义动作且不请求视觉定位

#### Scenario: 使用视觉坐标候选

- **WHEN** 前一版本因元素缺失或歧义交回临时窗口截图，慢脑提交同语义坐标候选
- **THEN** Application仍校验动作语义与窗口绑定，Driver只执行该声明坐标并在动作后Observe

#### Scenario: 不可核实视觉聚焦后输入引用文本

- **WHEN** 坐标聚焦被Driver接受且同次窗口Observe有效，但效果为`unverifiable`
- **THEN** Worker保存一次性焦点凭据并交回截图；慢脑核验后，下一片段的同任务同窗口对应文本语义在派发前消费凭据并展开引用文本，不提升前一步成功或自动重试

### Requirement: 失败Desktop实例不得抢占Gateway

Desktop MUST 在TaskHost成功初始化后才创建本地Gateway Socket。持有`host.lock`的正式实例存在时，新实例 MUST 失败退出且 MUST NOT 覆盖、删除或接管现有Socket。

#### Scenario: 第二实例启动

- **WHEN** TaskHost因宿主锁不可用而初始化失败
- **THEN** Desktop在Socket创建前停止启动，既有Gateway保持可达

### Requirement: 发送必须由顶部本机确认一次授权

系统 MUST 仅在执行到发送槽位时允许顶部执行浮窗展示发送预览。只有本机用户可批准或拒绝；批准引用在副作用派发前消费一次，任何结果都不得恢复或自动重试。

#### Scenario: 尚未批准

- **WHEN** 计划到达发送槽位且确认尚未批准
- **THEN** 原动作不执行，计划返回`awaiting-confirmation`并保持槽位，顶部浮窗显示预览与操作按钮

#### Scenario: 批准后结果未知

- **WHEN** 用户批准后发送动作返回超时、断连或`unknown`
- **THEN** 确认引用保持已消费，计划交回慢脑核实，系统不自动重发
