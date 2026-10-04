# Design

`CuaWorker`仅接受`@oai/sky@0.7.1`入口，并从该包相对定位官方签名`SkyComputerUseClient.app/.../SkyComputerUseClient`。组合根从已安装ChatGPT/Codex资源定位SDK；缺失、桥接身份不匹配或MCP握手失败时CUA capability unavailable，不能回退trycua。受监管Node Worker按任务会话复用，内部只监管一个`SkyComputerUseClient mcp` stdio子进程；任务结束、unknown、超时或崩溃时两者一并销毁。

macOS `launch_app`通过桥接MCP `list_apps`把显示名或bundle id收敛为唯一bundle id，并由`get_app_state`后台启动/观察。后续步骤复用同任务缓存目标；没有缓存时才由可信PID解析唯一App bundle路径。每次动作前后读取完整应用状态，解析transcript行首index，不复用跨步index。Yonder不得直接连接`computeruse.sock`，也不得调用Node SDK的direct native-pipe client。

Worker消费Application注入的`_yonder_action_kind`与受保护文本：聚焦搜索优先点击唯一搜索文本框，输入优先对该文本框`set_value`并从后置transcript验证目标值，激活使用受限按键或唯一元素点击。不能唯一定位或验证时返回`UnknownObserved`及同应用截图；语义字段与正文不进入日志。

外部协议和SQLite不新增transcript字段。`ComputerObservation`继续只暴露有界元素数量、临时截图与可见性；因此协议保持1.40。包构建删除trycua模块与旧Worker，只复制Node、Sky Worker和Jev依赖；签名MCP Client继续来自外部固定Sky安装，不复制进Yonder包。
