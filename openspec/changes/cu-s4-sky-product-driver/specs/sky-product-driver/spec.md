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

- **WHEN** Client在当前任务已收敛的唯一`launch_app`目标调用范围内发出固定空Schema elicitation
- **THEN** Worker只接受该应用的会话访问；调用范围外或Schema不匹配时拒绝，且不由此放行发送等副作用

### Requirement: 新鲜应用级AX元素优先

系统 MUST 对同任务绑定的唯一应用在每步前后读取新鲜AX transcript，并优先使用当前`element_index`执行。

#### Scenario: 已安装应用尚未进入官方近期目录

- **WHEN** `launch_app`携带合法bundle id且官方`list_apps`零命中
- **THEN** Worker只在固定系统应用根按Info.plist唯一解析完整路径后启动；零命中、多命中、符号链接或路径逃逸均安全交回

#### Scenario: QQ音乐公开唯一搜索框

- **WHEN** 新鲜transcript包含唯一可操作的搜索文本框
- **THEN** Worker使用该次index执行聚焦或赋值，动作后重新Observe且不走坐标兜底

#### Scenario: 元素缺失或多义

- **WHEN** 语义目标无法从新鲜transcript唯一解析
- **THEN** Worker停止动作并返回同应用有界视觉Observation供慢脑重规划，不自动重试

#### Scenario: 通用文本输入没有唯一搜索元素

- **WHEN** 已绑定应用不存在唯一搜索文本框且计划候选为通用`input-text`
- **THEN** Worker可对该应用执行`type_text`，但只在动作后应用状态发生变化时确认成功

#### Scenario: 锁屏或目标窗口不可观察

- **WHEN** 签名Client返回固定`cgWindowNotFound`错误
- **THEN** Worker返回有界`target-window-unavailable`阶段并交回，不记录原始错误且不继续动作

### Requirement: 可信窗口绑定跨步骤外置保持

系统 MUST 由Rust CUA Adapter运行态持有最近一次成功启动产生的同任务规范bundle id；Node Worker与签名MCP Client不得成为跨步骤状态所有者。

#### Scenario: Worker或Client在同一任务中重建

- **WHEN** 成功启动后的Worker进程或签名MCP Client被重建，归属慢脑随后提交同任务后续窗口动作
- **THEN** Adapter仍注入原可信bundle id，Worker按该身份重新解析唯一运行App/窗口，不退回当前前台

#### Scenario: 任务边界清理绑定

- **WHEN** CUA会话结束、切换到其他任务或开始新的`launch_app`
- **THEN** Adapter清除旧绑定，且新任务不能读取或复用旧任务应用身份

#### Scenario: 外部调用尝试覆盖绑定

- **WHEN** Agent或UI提交应用、PID、窗口或内部绑定字段
- **THEN** Gateway继续拒绝受保护身份；绑定不进入外部协议、SQLite、事件或Outbox

#### Scenario: 自绘控件需要双击激活

- **WHEN** 协议1.41计划以通用`activate-control + click(x,y)`提交`click_count=2`
- **THEN** Worker把次数原样映射到固定Sky并在动作后Observe；其他语义、元素点击或大于2的次数在Gateway拒绝

#### Scenario: 自绘候选需要键盘下移

- **WHEN** 协议1.41计划以通用`activate-control + press_key`提交`ARROWDOWN`
- **THEN** Worker只下移一个候选并在动作后Observe；其他导航键及发送语义在Gateway拒绝

### Requirement: 计划片段任务可安全终结

Gateway MUST 按任务实际运行时归属选择终结路径，不得仅因组合根存在内存Runtime就把未登记的计划片段任务误报为不存在。

#### Scenario: 计划片段未登记到内存Runtime

- **WHEN** 任务的最新计划动作已Observed并推进，内存Runtime对该任务返回`NotFound`
- **THEN** Gateway使用计划片段既有桌面租约与attempt门禁完成任务；Runtime的冲突、背压或不可用不得触发该兼容路径

### Requirement: AX transcript不持久化

系统 MUST 仅在Worker内使用AX transcript；不得写入任务状态、事件、Outbox、日志或顶部浮窗。

#### Scenario: 动作完成

- **WHEN** Worker取得动作前后transcript
- **THEN** 对外Observation只包含元素数量、受控截图引用与可见性，不包含transcript正文
