# AD-CU-05 Agent Computer Gateway

状态：Accepted；日期：2026-09-16。关联 CU-S2、AG-S1、TM-S2/TM-S3、AD-CU-02/04、AD-BU-02。

本地 Agent 通过统一 Gateway 请求 `computer.execute` 和 `task.complete`。2026-09-16用户修订：Yonder不得复制或维护CUA动作枚举与参数模型；协议1.11把原1.9的`type-text`专用参数替换为通用`tool_name + arguments`信封，SDK `listToolsJson()`是动作名称与参数Schema来源。Yonder生成attempt、Worker与host身份，取得唯一`Resource::Desktop`，每次调用后强制Observe并写入任务事件。

Agent不得提交PID、窗口号、Session、target、snapshot、元素token/index、Worker或host身份及SDK路径。Gateway递归拒绝这些受保护字段；SDK定义的`scope`是动作路由语义，不是原生身份，允许透传并由SDK Schema校验。Worker只在非`desktop` scope且SDK Schema包含相应字段时注入Yonder解析的窗口目标；`desktop` scope不注入PID或窗口号，再原样调用`callTool`。SDK拒绝未知动作、非法参数与授权不足；Yonder不解释动作语义、不做规划、不维护第二份动作白名单。协议的Rust类型仍是Gateway信封唯一来源，SDK动作Schema不复制到Rust协议。

2026-09-16实现证据：macOS目标解析先读取AX焦点应用/窗口并映射WindowServer ID，避免跨Space全局顺序误选；只读CGEventTap仅把来源PID为0的真实硬件键鼠计为用户接管，SDK合成事件不触发误停。协议1.11通用桥接的`type_text`与`press_key`证据分别位于`apps/desktop/evidence/computer-sdk-bridge-20260916`和`apps/desktop/evidence/computer-sdk-bridge-press-key-20260916`；真实鼠标中断证据位于`apps/desktop/evidence/computer-gateway-user-input-20260916`。

macOS目标固定为请求到达时最前方、可见、非Yonder的layer-0窗口，由宿主原生Adapter解析为内部`WorkTarget`；目标缺失或不唯一时不准备attempt。该选择规则不做语义规划、不按标题猜测、不移动窗口。单次SDK参数JSON不得超过16KiB，正文不得进入日志、事件或验证证据。

Driver 使用固定 trycua SDK 0.25.0 和受监管按需 Worker。Capability 只有在 SDK、Node、辅助功能权限、目标解析与用户输入监测均可用时才为 available。动作期间检测到新的真实 HID 键鼠输入时终止 Worker，将 attempt 记为 `unknown/user-input`，任务转为 `interrupted`并释放桌面输入租约；不得重试或自动恢复，也不得开启 Recording。

2026-09-16连续动作修订：同一任务执行期内的CUA动作复用一个受监管Node Worker和同一个trycua Driver会话，按唯一Desktop租约串行执行；不得在连续动作中插入另一套原生动作执行器。每个动作仍独立生成attempt并强制Observe。任务完成、用户输入、超时、Worker/SDK崩溃或宿主退出时销毁该会话，后续动作只能由Agent在重新Observe并作出决策后进入新会话。macOS原生代码仅提供可信目标解析、HID中断信号，以及已停止后人工接管的WorkRef定位；WorkRef定位不得发生在Agent CUA动作序列中。

已Observe的普通步骤仍使用1.7 `task.step.advance`。`task.complete`只允许归属Agent在running、最近attempt已stopped、无pending control且持有任务资源时提交；完成事务成功后才释放资源。Windows按用户决定暂缓，声明dependency missing；本决定不增加语义规划、自动安装或绕过SDK授权。

2026-09-16粗粒度接口修订：协议1.12新增`computer.step`，把步骤声明、attempt准备、SDK动作、强制Observe和普通边界推进收敛为一次Agent往返；SQLite仍按原事务边界逐阶段提交，崩溃恢复语义不变。1.11底层`computer.execute`保留兼容但不再向MCP默认发现，避免每个动作由Agent重复调用declare/execute/advance。响应只返回紧凑任务状态、动作结论和有界Observe证据，不回传完整SDK Payload。

Node Worker继续是Yonder应用包内的固定trycua Adapter：其进程隔离承担SDK崩溃、超时和用户输入中断，不属于Agent协议层，也不消耗模型token。除非trycua提供稳定受支持的Rust进程内SDK，否则不复制其生成FFI或把Worker代码移入Skill。后置Observe由trycua生成桌面状态；截图最多4MiB，写入Yonder受控临时目录并以本地只读路径返回，任务完成或会话终止时清理。日志、事件和Outbox不得写入截图、正文或完整结构化Payload。

2026-09-17跨Space应用前置修订：`launch_app`保持SDK定义的后台启动语义，Yonder不得把它隐式改写为`bring_to_front`。Worker只在同一任务、同一受监管SDK会话内暂存最近一次成功`launch_app`返回的bundle id、PID与普通窗口号；每次Observe先按SDK bundle id解析当前主进程，处理启动器向主进程的PID交接。Agent下一步显式调用`bring_to_front`且不提交受保护身份时，Worker注入该可信返回值。缓存不持久化、不跨任务，并随Worker会话终止清除。SDK若不能把窗口从其他Space前置，结果保持失败；Yonder不得硬编码Dock坐标、调用系统脚本或伪报已前置。屏幕截图权限不可用时，`target_visible`仅用于启动和显式前置：后台启动按SDK契约为false，前置要求SDK已确认动作成功且后置窗口可见；其他动作返回null，不依赖瞬时active或单独on-screen状态猜测前台，也不解释界面内容。
