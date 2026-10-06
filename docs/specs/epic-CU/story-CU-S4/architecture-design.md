# CU-S4 架构设计

## 边界与依赖

CU-S4只改变macOS CUA Adapter及其受监管Sky Worker，不改变Application/Domain依赖方向、Agent Gateway外部协议或SQLite。Rust Adapter拥有同任务可信应用绑定，Node Worker与签名MCP Client是可重建执行资源；依赖固定`@oai/sky@0.7.1`与同发行物签名Client。Windows路线继续暂缓。

## 状态与契约

成功`launch_app`以后，Adapter内存运行态保存`task_id + bundle_id`并只读注入后续内部Worker请求。每步仍使用新鲜AX transcript和动作后Observe；外部Agent/UI不能提交或覆盖该绑定。绑定不持久化，任务会话结束、任务切换或新启动尝试时清除。

## 失败与验证

窗口不可观察、目标不唯一、Worker/MCP断连或后置事实不足均保持unknown并交回，不自动重试动作。验证必须覆盖Adapter生命周期、Worker重建、跨任务隔离、Sky语义动作回归和正式macOS Gateway样本；Windows证据不得由macOS外推。

## 边界与候选

Spike在`spikes/cua-foreground-delivery/`中运行，固定比较产品当前0.25.0与候选0.30.4。两者均只通过官方进程内SDK读取工具Schema和运行隔离样本；不连接Yonder Gateway、不读任务库、不复用产品Worker进程。

0.30.4原生类型新增`ClickInput.target + position + deliveryMode`；实际`listToolsJson/callTool`目录进一步为`click`、`hotkey`和`type_text`公开精确target、窗口坐标与`delivery_mode`。Spike优先验证`type_text(x,y,text,foreground)`能否原子建立焦点并输入，同时保留分离click路径的Schema证据；不能从字段存在推导可用。

## 统一样本

隔离fixture包含一个可读焦点和值的文本框，并由独立进程打开诱饵前台窗口。探针记录诱饵身份，取得目标PID/窗口和新鲜窗口截图，从截图坐标执行一次原子前台输入固定标记`YONDER_SDK_INPUT_A`，再以独立原生控件读和SDK Observe双重验证。最后检查诱饵前台恢复、诱饵未变化、Worker关闭和进程清理。

失败注入覆盖错误窗口、过期截图或坐标、目标关闭、`unverifiable`和动作后Observe失败。任何失败停止样本，不重放点击或输入。

## 数据与依赖

证据只保存版本、Schema字段布尔、效果枚举、焦点/值匹配布尔、前台恢复布尔、耗时和进程清理布尔；不保存截图、窗口标题、用户输入或完整SDK Payload。Spike依赖独立package-lock，不修改`apps/desktop/cua`。

## 淘汰门槛

缺少以下任一项即淘汰升级路线：固定包可验证安装、精确窗口前台click、明确焦点后置事实、同窗口文本Observe、原前台恢复、错误目标fail-closed、关闭清理。只在fixture通过而真实自绘应用仍无确认时，结论缩小为“原生fixture兼容”，不得解除EX-S2正式样本门禁。

## 架构影响

关联Accepted [AD-CU-07](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-07-TRYCUA-FOREGROUND-DELIVERY.md)。Spike阶段Architecture Impact为`none`；产品接线通过独立Apply Change修改协议与Adapter Worker，不替换SDK版本。

## 产品Apply（协议1.39及AD-CU-07修订）

Accepted AD-CU-07最初决定保留0.25.0；正式快捷键样本补充后已修订为固定唯一0.30.4。Rust协议与Application只为`enter-target-query`、`draft-message-ref`的`type_text`接受恰好两个有限坐标参数；既有视觉`click`仍只接受同样的`x/y`。`delivery_mode`、target、正文和会话身份不得来自Agent。

Worker在展开意图引用后，为坐标文本或点击注入SDK工具目录声明的`delivery_mode=foreground`、精确窗口target和受监管session。坐标文本是单个Driver动作，不先发独立click；动作后Observe同一窗口并保留截图。只有`confirmed + observe_valid`可推进，其他结果交回且不得建立视觉焦点凭据或重试。

## AX优先与视觉降级补充（AD-CU-08）

Worker首次Observe仍以`include_screenshot=false`读取已绑定窗口的AX元素。若元素树为空，或动作结果不是`confirmed`，只对同一可信PID/window补采一次截图；正常元素路径不增加截图成本。该截图是当前失败边界的新鲜Observation，不触发动作重放，也不改变任务结论。

`UnknownObserved`携带的Observation必须由TM-S9内存运行时原样返回Gateway，不能因为持久化异步化而丢弃。通用`computer.step`坐标文本复用同一精确窗口foreground注入；受保护消息计划仍从内存引用展开正文并保留发送确认，不因通用搜索场景放宽。

## 通用桌面计划片段补充（协议1.40）

通用桌面任务由归属慢脑从同一Gateway一次提交1～10个槽位。协议新增`focus-control`、`input-text`、`activate-control`三种封闭动作语义：聚焦只允许窗口内点击或`cmd+f`，输入只允许有界文本与可选窗口局部坐标，激活只允许窗口内点击或`ENTER/RETURN/SPACE`。Application在派发前复用同一参数校验，并把语义标记注入Worker；Agent不能提交Driver身份、session、target或投递模式。

`focus-control/focus-target-search`的封闭`cmd+f`由Worker根据SDK Schema注入`delivery_mode=foreground`和精确窗口target，语义与既有坐标foreground动作一致。该投递属于单个Driver动作，不新增`bring_to_front`步骤；若Driver不支持foreground或后置Observe未确认，立即携带同窗口视觉证据交回。

计划接受后，宿主从已验证片段建立全部槽位的只读内存投影，顶部浮窗展示片段总数与当前附近至多四个槽位标签；执行只移动当前槽位和完成状态，不能用`computer.step`的“慢脑单步”覆盖整个片段。每步后仍Observe；AX事实不足才走同窗口视觉降级，越界或失败则交回同一归属慢脑重规划。

## Sky产品单栈补充（AD-CU-09）

macOS组合根只构造固定`@oai/sky@0.7.1` Worker，删除`YONDER_CUA_DRIVER`选择、trycua生产依赖和包内旧Worker。Sky包与Computer Use App由已安装的Codex/ChatGPT产品提供，Yonder从固定包相对定位官方签名Node与`SkyComputerUseClient mcp`，校验Team ID、Client identifier及App Group后，由签名Node运行受监管Worker并派生Client；不复制、不重新签名、不直连Computer Use Socket、不长期维护双栈。

Rust Adapter持有`task_id + bundle_id`的唯一CUA目标绑定；Node Worker与签名MCP Client均为可重建执行资源，不持有跨步骤权威状态。`launch_app`经动作后Observe确认后才更新绑定；后续请求由Adapter只读注入匹配当前任务的bundle id，Worker每步重新解析唯一运行App/窗口。无匹配绑定时只能使用宿主本次可信`WorkTarget`，不得读取旧任务绑定或改投当前前台。Worker/Client异常只销毁执行进程，不销毁绑定；`end_session`、任务切换或新启动尝试同时清除绑定。绑定不持久化、不进入外部协议、事件、Outbox或日志。

每步经签名MCP Client调用`get_app_state`取得新鲜完整transcript，从行首元素index解析可操作元素。封闭语义优先映射到MCP `click(element_index)`、`set_value(element_index,value)`和`press_key`；元素缺失或多义时才返回同应用截图供慢脑重规划。transcript不跨进程返回、不持久化、不记录日志，动作后再次Observe并只回传元素数量、截图引用和可见性等有界事实。

官方`list_apps`只保证运行中或近期应用，不是完整安装目录。`launch_app`收到合法bundle id且该目录零命中时，Worker可只枚举`/Applications`、`/System/Applications`及其`Utilities`一级应用包，读取`Info.plist`并在唯一命中后使用规范完整路径；不递归全盘、不读取用户提供目录、不接受环境覆盖、不以本地化显示名兜底。多命中、路径逃逸、符号链接或损坏包均安全交回。

Sky的应用使用elicitation只能在`launch_app`已把计划目标收敛为唯一bundle id和路径后，在该目标的单个串行MCP调用范围内按固定空对象Schema接受；调用范围外或Schema不匹配的请求拒绝，不使用本地化显示名作身份。该会话级应用授权不等于发送等副作用确认，后者仍由Yonder Gateway确认引用约束。

通用`input-text`优先对唯一搜索文本框执行`set_value`；没有唯一搜索框时可退回同一已绑定应用的MCP `type_text`，并必须由动作后应用级Observe证明状态变化。消息正文仍只走受保护引用语义，不使用该通用退路。

签名Client的错误正文不得进入日志或Gateway；Worker只把已知`cgWindowNotFound`收敛为`target-window-unavailable`等有界阶段。锁屏、登录窗口或目标没有可观察窗口时必须安全交回，不得把transport存活误报成动作成功。

协议1.41为自绘控件视觉激活增加唯一受限扩展：只有`activate-control + click`且使用有限`x/y`时可选`click_count=1|2`；坐标层穿透时，`activate-control + press_key`只额外允许`ARROWDOWN`选择下一个候选。字段直接映射固定Sky公开参数；焦点、输入、元素点击、发送及其他语义均拒绝，不能用多个分离单击冒充双击，也不开放其他导航键。动作后仍读取同一应用状态，截图只有悬停变化时不得据此推导业务目标完成。

Gateway的终结路由以任务是否存在于内存执行Runtime为判据，而不是以组合根是否配置了Runtime为判据。内存Runtime持有的任务保持事件驱动终结；尚未迁入该Runtime的计划片段链只在最新attempt已Observed、步骤已推进、任务仍持有桌面租约时走既有终结函数。`RuntimeError::NotFound`只触发这条兼容路由，冲突、背压或不可用不得降级。
