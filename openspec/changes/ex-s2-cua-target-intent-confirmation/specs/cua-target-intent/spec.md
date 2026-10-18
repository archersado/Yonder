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

#### Scenario: 不可核实视觉聚焦禁止输入引用文本

- **WHEN** 坐标聚焦被Driver接受且同次窗口Observe有效，但效果为`unverifiable`
- **THEN** Worker交回截图但不保存焦点凭据；下一片段的引用文本语义必须被拒绝，不提升前一步成功或自动重试

#### Scenario: 封闭搜索快捷键

- **WHEN** 企业微信计划为聚焦会话搜索提交`hotkey`候选
- **THEN** 协议只接受`keys=["cmd","f"]`，Worker注入精确窗口target并Observe；其他组合键或附加参数在派发前拒绝

### Requirement: 失败Desktop实例不得抢占Gateway

Desktop MUST 在TaskHost成功初始化后才创建本地Gateway Socket。持有`host.lock`的正式实例存在时，新实例 MUST 失败退出且 MUST NOT 覆盖、删除或接管现有Socket。

#### Scenario: 第二实例启动

- **WHEN** TaskHost因宿主锁不可用而初始化失败
- **THEN** Desktop在Socket创建前停止启动，既有Gateway保持可达

### Requirement: 发送必须由顶部本机确认一次授权

系统 MUST 仅在执行到发送槽位时允许顶部执行浮窗展示发送预览。只有本机用户可批准或拒绝；批准事件 MUST 自动唤醒同一活动计划片段，批准引用在副作用派发前消费一次，任何结果都不得恢复或自动重试。

#### Scenario: 尚未批准

- **WHEN** 计划到达发送槽位且确认尚未批准
- **THEN** 原动作不执行，计划返回`awaiting-confirmation`并保持槽位，顶部浮窗显示预览与操作按钮

#### Scenario: 批准后结果未知

- **WHEN** 用户批准后发送动作返回超时、断连或`unknown`
- **THEN** 确认引用保持已消费，计划交回慢脑核实，系统不自动重发

#### Scenario: 本机批准自动续跑

- **WHEN** 活动片段在发送槽位等待确认且用户在顶部浮窗批准
- **THEN** Desktop以绑定原任务归属、片段版本、Runtime游标和序号的可信内部来源自动继续同一片段
- **AND** 不要求慢脑再次调用执行，不创建Agent会话、不改变候选、不提交新计划或任务终态
- **AND** 人工等待跨越原Gateway deadline时仍可续跑，但确认自身期限、控制状态、活动游标与引用绑定任一失效都必须在副作用前交回

#### Scenario: 自动续跑完成后唤醒终局核验

- **WHEN** 本机批准触发的可信续跑完成最后一个槽位且最新Observe有效
- **THEN** Desktop通过当前唯一归属Agent会话投递有界`goal_verification`输入，要求慢脑读取Gateway新鲜事实后提交目标核验与终态
- **AND** Driver、Desktop与Jev都不得直接把动作成功写成任务完成；浮窗保持到Gateway终态到达后才关闭
- **AND** Agent未连接或投递失败时任务保持运行中并展示等待状态，不丢失任务、不自动重发副作用

#### Scenario: Connector重连恢复待核验任务

- **GIVEN** 计划片段已完成且任务仍在等待终局核验
- **AND** 首次核验唤醒因Connector不兼容或断开而未送达
- **WHEN** 支持协议1.53的同一归属Agent Connector注册成功
- **THEN** Yonder依据Store与Execution Runtime的新鲜事实重投一次有界核验输入
- **AND** 不自动完成任务、不重放发送动作，也不以UI投影作为判断依据

#### Scenario: 批准反馈不被轮询旧投影覆盖

- **WHEN** 用户已批准或拒绝发送，但服务端执行投影仍暂时返回“等待用户确认”
- **THEN** 顶部浮窗在每轮渲染中保持当前任务的本地确认结果，直到新的步骤、交回或终态事实到达
- **AND** “快脑”与“正在执行”文案不得在确认前后状态之间闪烁，旧异步请求不得覆盖新任务或新结果

#### Scenario: 等待确认期间计划片段过期

- **WHEN** 当前活动片段停在发送槽位等待本机确认，归属Agent再次执行时该片段已经过期
- **THEN** Application在消费确认和派发Driver前形成有界handback并允许慢脑提交新片段
- **AND** 系统不得只返回CAS冲突而把过期片段永久保留为运行中，也不得持久化确认状态或敏感正文
