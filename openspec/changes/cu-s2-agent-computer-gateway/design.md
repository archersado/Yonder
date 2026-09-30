# 设计

Rust协议以1.11通用`computer.execute(tool_name, arguments)`信封桥接SDK，不定义CUA动作枚举或参数模型；`task.complete`保持1.10。Gateway递归拒绝受保护目标与执行身份字段，允许透传SDK动作语义`scope`。Worker读取SDK `listToolsJson()`验证工具；非`desktop` scope按SDK Schema注入Yonder解析的窗口与会话，`desktop` scope不注入窗口身份，再调用`callTool`。TaskHost组合现有SQLite、Admission、CU Port与macOS最前方窗口解析；首次动作取得Desktop资源，后续动作复用任务占用。每次SDK调用后强制Observe；用户输入、Worker异常或Observe失败保持保守语义且不重试。完成只接受已停止的观察边界，提交后释放任务资源。

Node Worker与trycua Driver在任务执行期内惰性创建并跨正常动作复用，所有请求仍受唯一Desktop租约串行化。任务完成、用户输入、超时、Worker/SDK异常会终止该会话；后续调用不得把新会话视作原动作延续。原生代码仅提供目标解析和中断信号，已停止后的人工接管定位不进入Agent动作链。

协议1.12新增粗粒度`computer.step`，Application内部顺序复用declare、execute/result与advance；MCP不再默认发现底层`computer.execute`。Worker用trycua后置桌面Observe生成最多4MiB截图，保存到Yonder受控临时目录；Gateway响应仅含紧凑结论、元素数量、MIME和本地路径。任务完成或异常会话终止时清理证据，不写入SQLite、事件、Outbox或日志。

目标解析采用元素优先、视觉兜底：前置Observe默认不请求截图；`type_text`等需要语义目标的动作先解析唯一AX/UIA元素。只有目标缺失或不唯一且尚未执行动作时，Worker才追加一次截图Observe，并把动作结论收敛为已知拒绝而非Worker故障。计划执行响应携带该临时Observation，归属慢脑读取图片后可提交带语义标签的坐标候选。Yonder不自动重试、不持久化截图，也不因引入兜底而在正常元素路径调用视觉能力。

同一任务经`launch_app`取得的可信应用身份适用于后续所有窗口级后台动作及其Observe，而不只适用于`bring_to_front`。Worker每步刷新该应用的PID/窗口；不得退回当前前台窗口或隐式激活应用。`scope=desktop`不注入窗口身份。

窗口级动作的后置Observe固定使用同一可信窗口的`get_window_state`；仅显式`scope=desktop`使用`get_desktop_state`，避免用当前前台桌面截图证明后台动作或驱动错误的视觉重规划。

计划片段正常完成时保留最后一次临时Computer Observation并随1.34响应返回，使坐标动作能够由慢脑核验；它不写入任务事件或持久化，也不要求结构化元素已足够时调用视觉模型。

动作effect为`partial/unverifiable/suspected_noop`时，attempt仍按`unknown/observe-failed`停止且不重试；若后置窗口Observe有效，Application保留其临时Observation并随交回响应返回。Worker停止会话但延迟清理该证据，直至任务终结或宿主回收。
