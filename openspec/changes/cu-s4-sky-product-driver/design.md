# Design

`CuaWorker`仅接受`@oai/sky@0.7.1`入口，并从该包相对定位同发行物的OpenAI签名Node与官方签名`SkyComputerUseClient.app/.../SkyComputerUseClient`。组合根校验Team ID、Client identifier与App Group；缺失、桥接身份不匹配或MCP握手失败时CUA capability unavailable，不能回退trycua。签名Node运行受监管Worker，Worker按任务会话复用并只监管一个`SkyComputerUseClient mcp` stdio子进程；任务结束、unknown、超时或崩溃时两者一并销毁。

macOS `launch_app`通过桥接MCP `list_apps`把显示名或bundle id收敛为唯一bundle id，并由`get_app_state`后台启动/观察。成功后的`task_id + bundle_id`由Rust Adapter运行态外置持有，Node Worker不保存模块级目标；后续步骤由Adapter只读注入同任务绑定，Worker每步重新解析唯一运行App/窗口。Worker或MCP Client重建只重建执行资源，不清除绑定；任务会话结束、任务切换或新启动尝试才清除。没有匹配绑定时才由可信PID解析唯一App bundle路径。每次动作前后读取完整应用状态，解析transcript行首index，不复用跨步index。Yonder不得直接连接`computeruse.sock`，也不得调用Node SDK的direct native-pipe client。

握手后读取固定工具Schema；仅当Schema公开`disableDiff`或`disable_diff`时才请求关闭差异状态，正式Sky 0.7.1未公开时不得发送额外字段。transcript只从结构结果、带说明前缀的包装JSON、普通文本块或标准嵌入文本资源解包，并要求解析出带索引的AX元素；状态摘要或空增量不能被解释为“应用没有元素”。

`list_apps`未列出合法bundle id时，只在固定的`/Applications`、`/System/Applications`与`/System/Applications/Utilities`一级目录读取应用`Info.plist`，唯一命中后才把规范完整路径交给签名Client；禁止环境覆盖、全盘递归、显示名猜测、符号链接与多命中选择。

Worker消费Application注入的`_yonder_action_kind`与受保护文本：聚焦搜索优先点击唯一搜索文本框，输入优先对该文本框`set_value`并从后置transcript验证目标值，激活使用受限按键或唯一元素点击。不能唯一定位或验证时返回`UnknownObserved`及同应用截图；语义字段与正文不进入日志。

MCP应用使用elicitation只在当前任务唯一`launch_app`目标的串行MCP调用尚未返回时接受固定空对象Schema；其他server request拒绝。目标身份来自已校验bundle id与路径，不信任本地化提示文案。接受只覆盖应用会话访问，不覆盖Yonder对发送等副作用的独立确认。

通用`input-text`先寻找唯一搜索文本框并`set_value`；不存在唯一搜索框时才对同一绑定应用调用`type_text`，且以后置应用状态变化确认。消息引用输入不走该退路。

Client错误正文只在Worker内做固定码/固定标记分类；`cgWindowNotFound`映射为`target-window-unavailable`，不把应用状态、AX正文或原始错误写入宿主日志。该阶段触发带界交回，不重放动作。

`activate-control + click(x,y)`属于不含业务副作用的界面导航。Worker使用原始截图像素只调用一次`click`，并在签名Client的标准动作等待期内保持同一执行环境。调用超时或返回旧窗口失效时不关闭Client、不再次调用`click`，而是继续只读Observe同一绑定应用；若Observe确认窗口代次已变化，可重建Client并按可信bundle id重新解析唯一运行窗口。截图差异本身不能证明动作已投递，因此超时分支仍以`UnknownObserved`携带新鲜证据交回。发送消息及其他副作用动作不进入该分支。

协议1.42为通用控件激活增加唯一受限参数`observed_element_index`。Application只接受当前新鲜AX transcript中的`1..=65535`整数；Worker动作前重新Observe确认索引仍存在，再内部转换为Sky `element_index`并只执行一次。元素和坐标激活动作后transcript不变均交回，不能用鼠标光晕或悬停截图变化证明目标完成。该索引不持久化、不透传为通用SDK参数，其他动作拒绝该字段。

外部协议和SQLite不新增transcript字段。`ComputerObservation`继续只暴露有界元素数量、临时截图与可见性。2026-10-05协议1.41仅为通用视觉`activate-control + click(x,y)`增加可选`click_count=1|2`，并为`activate-control + press_key`增加唯一`ARROWDOWN`导航，用于固定Sky已公开的自绘控件激活；其他参数边界不变。包构建删除trycua模块与旧Worker，只复制Node、Sky Worker和Jev依赖；签名MCP Client继续来自外部固定Sky安装，不复制进Yonder包。

任务完成/失败按实际运行时归属分派：内存Runtime能读取该任务时只走事件驱动终结；返回`NotFound`说明该计划片段任务尚未迁入内存Runtime，此时才回到原有已Observe/已推进/桌面租约三重门禁的终结路径。其他Runtime错误保持失败关闭，不能用SQLite结果覆盖活跃内存事实。
