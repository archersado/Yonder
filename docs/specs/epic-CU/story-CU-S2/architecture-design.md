# CU-S2 架构设计

## 边界与依赖

按需 Worker、单桌面租约，依赖 CU-S1、TM-S2；禁止全应用提权。

## 状态与契约

SQLCipher 保持任务当前事实源；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

## 失败与验证

Worker 停止/Observe 契约与平台证据未齐。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本文件为规划迁移，运行时无变化；具体实施按关联 ADR 和 Proposal 声明影响，draft/design-review 不授权实施。

## 首批受监管派发设计（AD-CU-04）

Application 定义 CU 请求/结果 Port，Adapter 监管按需 Node Worker；Worker 只加载固定 trycua SDK，经私有 stdio 接收一次内部可信目标，前置 Observe 取得唯一编辑元素，执行后台 AX 文本输入并强制后置 Observe。完整执行身份必须原样回传并由 Rust 校验。超时、崩溃、断连、非法响应或后置 Observe 失败均为 unknown，不重试。首批不新增 Gateway 协议或数据库字段，不启用停止、接管、定位或 Recording。

## macOS停止契约审阅输入（2026-09-14）

来源：原产品执行原则、架构CUA/恢复与用户接管，沿用AD-E0-02/AD-TM-03。CU-S1只读Goal证明JS提交后Abort拒绝及后续调用可用，shutdown并发读被拒绝；未观测原生准入或原生输入中断。SDK shutdown声明为关闭准入并等待排空，不能按强制停止实现。

后续联合设计顺序：TM先冻结新动作并保留当前attempt/租约；CU接收绑定当前Worker实例的停止请求；CU只提交可信原生停止结果，无法证明则unknown；TM提交控制结果后才允许RC显式接管记录。旧Worker/旧attempt回调不得释放新租约，SDK关闭或Node退出不得单独产生“已停止”结果。此处为审阅约束，不新增已定案协议或持久化字段。

当前集成以Accepted AD-CU-01 SDK-only为准；上游createPrivateWorker/App构件路线撤回。由Yonder Supervisor管理自身Node Worker，Worker仅加载SDK及匹配原生库；STOP-SDK01–04已验证只读就绪/监督终止/断连退出/恢复。下一Spike验证SDK隔离原生输入、停止后不再输入、动作后Observe、异常退出/残留与Yonder宿主权限责任链，不附带上游App，不把只读进程退出替代执行中动作停止，不引入本地HTTP/TCP。

SDK首批STOP-IN02–03原生验证已通过：隔离目标后台输入后SDK Observe与原生字段匹配，动作完成后shutdown拒绝后续输入，新SDK Observe及750ms字段状态稳定。证据见verification-input-macos。真实执行中动作中断、SDK后台输入具体交付路径/来源标记、正式Yonder权限归属和崩溃残留仍待验证；不启用接管Recording。

## MVP步骤边界控制（AD-CU-02，2026-09-14）

来源为原产品执行原则/逐任务控制、架构每步Observe及用户SDK-only人工接管/工作定位变更。冻结新动作→等待当前步骤返回并Observe→结果已知且attempt/Worker一致后确认停止→前置对应工作→满足隐私及显式记录条件后接管。边界等待显示正在停止，不提前显示已接管。超时/断连/Observe失败保持unknown与占用，不自动重试或录制；旧身份回调不释放新租约。合成延迟控件测试未成立，不作为额外产品前置；具体控制协议/持久化/目标及Recording仍待联合定稿，真实步骤竞态/停止/Observe和两平台证据门禁保留，Windows暂缓。

## 工作定位原生路线审阅输入（2026-09-14）

来源：本Story既有产品简报Task Space/逐任务权限映射、用户接管时前置正在操作工作与SDK-only变更、Accepted AD-CU-02/AD-TM-07/AD-CU-03。验收FOCUS-01停止确认后才定位；FOCUS-02可信当前任务目标唯一前置并保持工作屏幕原位置；FOCUS-03关闭/失效/不唯一保持暂停，提示“未能定位任务工作，请手动打开”，不前置同名工作；FOCUS-04小龙不移动，BUA仍外部引用。

macOS隔离正常前置、最小化恢复与关闭拒绝已验证；SDK本身无精确focus接口，产品沿CU内部原生AX/CoreGraphics路线。WindowServer可见和AX就绪分开；界面显示“正在定位工作”直至实际焦点/可见状态确认，返回成功不能提前显示已移交。定位失败保持暂停，显式记录条件仍按RC门禁；UI不接受PID/路径/任意Agent命令。

独立verification-focus-macos.md仅技术基线通过。生产当前WorkRef/attempt/Worker/启动身份、双侧唯一映射、事务事件和多Space/显示器及权限责任链尚未完成，不据此把控制协议设计标ready或启用接管。Windows用户暂缓，完整Story保持原状态。

## 执行身份与定位Port联合设计

[AD-TM-08](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-08-EXECUTION-IDENTITY-AND-STOP.md)定义候选执行/控制身份、停止事实与定位调用顺序。CU通过受监督通道报告当前动作和Observe结果，不修改任务状态、不释放Application租约；SDK生命周期退出不等于安全停止。WorkRef由可信Observe产生，绑定进程启动身份、WindowServer与保留AX对象，标题/几何不是长期身份；副作用前复核，关闭/对象替换/退化空树拒绝，不改选诱饵窗口。

原生对象仅在CU内存持有，重启后的持久引用不可操作。Rust Port、步骤接入和事务字段未定稿；需限时Spike验证进程启动身份/保留AX对象校验、正式宿主权限及所属Space/显示器后再审阅AD，不从现有隔离焦点基线外推。MVP存储按AD-ST-01，不新增执行栈或App。

## 生产 WorkRef Adapter 增量

Accepted AD-CU-03规定Application的`WorkRef`与`WorkFocusPort`是唯一调用边界。macOS Adapter内嵌原生实现并持有AX对象；捕获只接受可信attempt与WorkTarget，定位复核完整身份、进程启动时间、WindowServer及AX双侧唯一映射。失败只返回分类，不改任务状态。TM定位事务、UI和Recording不在本增量。

## Agent Computer Gateway增量

Accepted AD-CU-05新增协议1.9/1.10。Application Supervisor生成完整执行身份并复用Desktop Admission；macOS Adapter从WindowServer有界解析当前最前方非Yonder layer-0窗口，Agent请求不含原生身份。固定SDK Worker监测动作期间的新HID输入；检测到后结果为unknown、任务interrupted并释放桌面控制，不自动重试。任务完成仅允许最近Observed attempt已经普通边界停止且无pending控制。
2026-09-16用户修订：CUA Gateway只定义通用SDK调用信封，不复制动作枚举和参数Schema。SDK `listToolsJson()`是动作契约来源；Yonder递归拒绝Agent提供的目标、Session、snapshot和元素身份，由可信宿主注入后调用`callTool`。Yonder仍负责Desktop租约、attempt身份、用户输入中断、动作后Observe和结果事务。

2026-09-16桌面作用域补充：SDK `scope`属于动作语义而非宿主身份，Gateway允许其进入通用参数对象。Worker仅在`scope != desktop`时按SDK Schema注入PID或窗口号；desktop动作由SDK定位当前桌面。PID、窗口号、target、session、snapshot及元素身份仍由Yonder保护。

2026-09-16连续会话补充：`CuaWorker`在任务执行期内惰性启动一个Node子进程，Node只创建一次trycua Driver并逐行处理动作；Rust在唯一Desktop租约下串行请求并逐动作校验身份、超时、用户输入和后置Observe。正常步骤间不关闭Driver；任务完成或unknown边界立即终止子进程，下一次准入重新创建。原生WorkRef只服务已停止后的人工接管，不与Agent CUA序列交错。

协议1.12的`computer.step`是Application组合用例，不新增执行栈：依次复用既有declare、execute/result和advance事务，任一步失败都保留已提交事实供重试或恢复。MCP默认只暴露该粗粒度CUA入口，1.11接口仅兼容旧客户端。Worker后置Observe优先调用SDK `get_desktop_state`并请求截图；仅把元素数量及最多4MiB图片写入受控临时文件后返回路径，完整结构化结果不跨Gateway。Node仍是固定SDK Adapter，不能接受Agent代码。

## 跨Space应用前置增量

保持SDK动作原义：`launch_app`只启动，`bring_to_front`只前置。Worker在同一任务与SDK会话内缓存最近一次成功启动返回的bundle id、PID及最大普通窗口；每次Observe先用SDK应用清单按bundle id解析当前主进程，以容纳启动器向主进程交接PID，再仅为后续显式`bring_to_front`注入。缓存不进入协议、SQLite或日志。跨Space精确前置若被SDK拒绝，Agent可显式调用SDK的系统应用切换或Dock键盘动作并再次Observe。`target_visible`只在启动与显式前置步骤返回：后台启动按SDK契约为false；前置要求SDK已确认`bring_to_front`成功且后置窗口在屏幕上才为true。其他动作保持null，不用瞬时active或单独的on-screen状态猜测前台。进程交接、窗口重建或权限导致的Space三态未知不产生假失败。前置动作最多等待2秒吸收系统Space动画。Yonder不推断应用、不硬编码坐标、不调用系统脚本，也不把动作提交当成窗口可见。
