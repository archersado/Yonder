# 设计

Rust协议以1.11通用`computer.execute(tool_name, arguments)`信封桥接SDK，不定义CUA动作枚举或参数模型；`task.complete`保持1.10。Gateway递归拒绝受保护目标与执行身份字段，允许透传SDK动作语义`scope`。Worker读取SDK `listToolsJson()`验证工具；非`desktop` scope按SDK Schema注入Yonder解析的窗口与会话，`desktop` scope不注入窗口身份，再调用`callTool`。TaskHost组合现有SQLite、Admission、CU Port与macOS最前方窗口解析；首次动作取得Desktop资源，后续动作复用任务占用。每次SDK调用后强制Observe；用户输入、Worker异常或Observe失败保持保守语义且不重试。完成只接受已停止的观察边界，提交后释放任务资源。

Node Worker与trycua Driver在任务执行期内惰性创建并跨正常动作复用，所有请求仍受唯一Desktop租约串行化。任务完成、用户输入、超时、Worker/SDK异常会终止该会话；后续调用不得把新会话视作原动作延续。原生代码仅提供目标解析和中断信号，已停止后的人工接管定位不进入Agent动作链。

协议1.12新增粗粒度`computer.step`，Application内部顺序复用declare、execute/result与advance；MCP不再默认发现底层`computer.execute`。Worker用trycua后置桌面Observe生成最多4MiB截图，保存到Yonder受控临时目录；Gateway响应仅含紧凑结论、元素数量、MIME和本地路径。任务完成或异常会话终止时清理证据，不写入SQLite、事件、Outbox或日志。
