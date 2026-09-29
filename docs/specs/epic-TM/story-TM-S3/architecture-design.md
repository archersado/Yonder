# TM-S3 架构设计

2026-09-14停止设计前置：CU-S1 STOP-SUB01–04只读Goal通过，只有JS提交/完成边界证据，未确认原生动作准入、中断或Worker退出。并发shutdown读被拒绝不能证明执行中动作排空。接管仍依赖CU-S2真实停止契约，未满足时不开始记录或释放租约；完整实现门禁不变。

## 边界与依赖

调用执行器停止并确认；停止前不释放租约；依赖 TM-S2、CU-S2，更新控制协议 ADR 后实现。

## 状态与契约

schema9将当前 observed attempt 与任务边界停止原子绑定：pause/takeover为running→paused，cancel为running→cancelled；attempt转stopped并记录stop_sequence/control_id/control_kind。Application按当前sequence生成稳定control_id。Permit封装先提交事务再释放，失败返回Permit。首批无外部协议/UI、WorkRef定位或Recording。

按 AD-TM-01 联合矩阵：控制请求与确认停止分别记录，不能用 request_id 代替已派发 attempt_id。`prepared`尚未派发副作用，可登记控制以冻结后续派发；`observed`或已停止的安全步骤边界也可登记控制。只有`unknown`必须返回结果待核实，保留占用且不写入无法确认的pending控制。撤销/暂停后禁止新输入，但宿主仍处理原尝试的可信停止和结果，不因权限撤销丢弃安全确认；外部 Agent 不因此恢复读取或执行权。停止信号、确认和资源释放顺序在本 Story 具体定义，当前仅是跨 Story 约束。

MVP 按 Accepted AD-ST-01 使用未加密 SQLite 保持任务当前事实源，SQLCipher 延期至 ST-S2；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

## 失败与验证

尚未定义控制协议和停止确认契约。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 显式 CUA 接管增量（2026-09-28）

按 Accepted AD-CU-07，`CuaWorker` 删除 HID generation 轮询与 `UserInput` 结果生成；超时、Worker/SDK 故障、身份不符和 Observe 失败保持原语义。桌面宿主增加仅内存存在的 `CuaControlHub`，持有当前 CUA task、步骤摘要和一次性接管请求，不进入 SQLite，也不成为任务状态源。

本地 Gateway 在已校验的 `computer.step`、`computer.execute` 或 `plan.execute` 进入同步执行前发布控制卡；Hub 以 task_id 跨 RPC 保留，后续同任务 CUA 请求只刷新步骤投影。单次动作返回和 Observe 不清理 Hub；成功的任务终态响应、显式接管成功或确认的执行会话终止才清理 Hub 并隐藏窗口。`cua-control` Tauri 窗口只提交 task_id，Rust 校验窗口 label 与 Hub 当前 task 后立即登记接管，并用异步阻塞任务排队取得同一个 `TaskHost` 互斥边界：动作进行中自然等到其动作与 Observe 返回，步骤间则立即处理。窗口命令是接管信号的唯一消费者，在锁内读取最新 task sequence 并调用既有 `user_takeover_current`；Gateway 不再等待下一请求消费信号。Application 在单步声明前、计划片段每槽位派发前只读信号并返回 `takeover-requested`/停止，保证新请求不能越过已登记控制。

控制条窗口与圈选入口都以 pet 的 current monitor（回退 primary monitor）为锚点，并按 monitor work area 计算；横向居中且距顶部 16 逻辑像素。控制条固定逻辑尺寸、置顶、全工作区可见，但窗口本身只占控制条边界。前端不缓存任务事实、不提交 Agent 身份、sequence、PID/窗口号或定位结果。

### CUA 规划与执行步骤投影（2026-09-28用户变更）

`TaskHost` 在收到已校验的 `PlanExecute` 提示时，从 SQLite 的不可变计划片段读取至多四个槽位标签，形成只读 `CuaControlPresentation`；`CuaControlHub` 只暂存 task_id、步骤列表与当前 prepared attempt 的 step_id。`CuaControlPort` 在实际 Driver 派发前将执行 step_id 写入 Hub；控制条窗口通过受限 Tauri 命令读取该投影，不能修改它。计划片段仍由 Application/SQLite 管理，Hub 不是持久化或并发协调事实源。

2026-09-28用户追加要求把快慢脑过程同时透出。Application在每槽位进入选择前，通过既有`ComputerUsePort`的只读投影钩子发布候选数量；选择后只发布封闭`CuaActionKind`对应的用户可理解动作语义，单候选明确为慢脑直接授权，多候选明确为Jev已选择，HandBack明确为交回慢脑。该钩子不改变选择、CAS或Driver接口，不携带候选参数、置信度、模型原文或思维链。Hub在同一任务内保留`slow_brain_summary`与`fast_brain_summary`，UI和Driver均不是事实源。

窗口事件只用于低延迟提示，不能是唯一初始化路径。`cua-control.js`加载后必须主动调用受限`cua_control_presentation`取得当前投影；事件先于监听器、WebView重载或隐藏后重显都从同一Hub恢复。读取失败只保持安全默认文案，不影响执行。

单步 `computer.step`/`computer.execute` 没有计划列表时只提供当前标签；Plan 读取失败明确标记不可用而不影响 Application 调用。UI 只渲染纯文本标签、索引和 `completed/executing/pending` 状态；不读取 action arguments、Agent payload 或任何 Adapter 原始观察数据。卡片开始、每次 Driver 派发、接管和结束均重取/清理投影，避免旧任务残留。

2026-09-29步骤状态图标增量：`mark_executing`只把当前步骤置为`executing`，不再通过“已开始后续步骤”推断前序成功；Application在动作结果有效且步骤边界提交完成后显式调用默认空实现的完成投影钩子，Hub才把对应步骤置为`completed`。未知、失败、HandBack和仅能读取截图的结果均不得打勾。Hub保留已完成集合，后续同任务刷新不得把成功图标重置；列表外当前步骤仍更新独立“正在执行”文案。

真实企业微信样本暴露现有 Adapter 把 `!action.isError` 错当动作成功。trycua 的 `launch_app` 只承诺后台启动，`press_key/type_text` 还会返回 `confirmed/partial/unverifiable/suspected_noop/refused` 的 effect；Adapter必须消费该有界 effect，只有 `confirmed` 可进入成功结果，`partial/unverifiable/suspected_noop`按结果待核实停止，`refused`按已知失败处理。动作后截图成功只代表 Observe 载体可读，不能提升动作结论；计划的语义后置条件未匹配前不得显示成功或连续推进。需要可见前置时计划必须显式包含`bring_to_front`，不能把后台`launch_app`伪装成已唤起。

## 架构影响

本文件为规划迁移，运行时无变化；具体实施按关联 ADR 和 Proposal 声明影响，draft/design-review 不授权实施。

## 人工接管记录与交回（2026-09-14用户变更）

来源：本次用户明确执行中支持人工接管并记录用户行为，作为交回Agent的Observe依据；架构Recording与桌宠/任务恢复约束，Accepted AD-TM-03。仅Agent创建任务，人工接管不创建新任务。

验收映射：TAKE-01显式接管阻止Agent新动作，停止未确认不称已移交；TAKE-02本次手动接管开启可见、可停止的记录，只录user输入，密码/安全界面/排除应用不采；TAKE-03原始时间线不可变并关联原task_id；TAKE-04交回保存证据并执行新鲜Observe，向归属Agent交付轨迹/证据引用及当前状态；TAKE-05交接失败/配额缺口明确反馈、保持暂停，不自动续跑；TAKE-06Agent重新Observe并显式恢复，重新准入后才executing。

TM-S3管接管及停止确认，RC-S1管用户记录/不可变证据，TM-S5管任务时间线引用，TM-S4管交回及显式恢复，AG管归属Agent交接协议，DS-S2展示已有任务接管/记录中/交回。自动输入干预只暂停，不无提示开启Recording；显式接管作为用户手动开始。已有每设备Recording时明确冲突，不覆盖。

UI流程：任务面板“接管”→“正在停止Agent控制/记录中”→停止确认后人工控制；小龙按既有暂停表现并保留独立录制提示。用户可以停止记录，任务仍保持人工接管；交回时如果停止后有未记录操作标注证据缺口，最终Observe仍必须新采集。“交回Agent”不等于立即继续，不用点击桌宠自动恢复任务。

协议/持久化/Driver停止与授权/隐私字段未定稿，相关Story保持设计阶段，不提前实现采集或向外发送用户记录。Windows暂缓、真实双平台证据保留。

## 首批未开始取消设计定稿

# AD-TM-04 未开始任务取消

状态：Accepted（首批created取消）；日期：2026-09-14。Architecture Impact：architecture-change（协议控制入口）；关联TM-S3、AG-S1/S2、DS-S2。

来源：产品简报Task Space与权限模型逐任务取消、架构任务生命周期、用户允许操作已有任务。仅取消created任务，不派发动作、不释放执行租约，不启动Recording。TM-S2/CU停止前置仍约束所有执行过的任务；首批未开始分支不依赖Driver停止技术路线。

Rust协议1.2的task.cancel线格式保持不变。按2026-09-28用户变更，可信LocalUser可取消任意非终态任务，Agent仅所属；未开始排队与进行中任务统一直接取消，不再要求副作用回收或安全停止证明。

Application对created/running/paused/waiting_for_user/interrupted复用同一CAS状态/事件/Outbox事务提交Cancel，已cancelled幂等，completed/failed拒绝。TaskHost串行化当前调用与取消：未派发任务立即提交，已进入同步Driver的调用返回后提交；成功后清理准入、CUA投影与宿主会话，外部副作用回滚不作为前置。

本机控制与Agent Gateway均使用task.cancel；UI对全部非终态任务走同一取消入口，不再将running分流到task.control。成功刷新列表并保留终态数据；失败可见且不自动重试。

验证：实际SQLite身份隔离、版本拒绝、序号冲突、重复取消单事件、Outbox失败回滚、Start竞态、创建重试不复活；macOS真实按钮取消一条本地测试Agent任务，另一条保留，全部可见取消态。Windows暂停，完整暂停/接管/执行中取消仍待停止确认与双平台证据，不Archive完整TM-S3。

验收映射：CANCEL-01仅目标created变cancelled，其他任务不变；CANCEL-02可信身份、版本、序号验证拒绝不落写；CANCEL-03重复取消不追加事件，失败回滚三表；CANCEL-04列表刷新/全部保留终态、按钮可操作与错误可见。来源分别对应原产品逐任务取消、架构Gateway/唯一事实源、架构事务以及原Task Space交互。完整暂停接管与Recording原需求保留。

## 卡片取消保留数据定稿

# AD-TM-06 删除入口取消并保留数据

状态：Accepted；日期：2026-09-14。Architecture Impact：architecture-change（撤销实验清理协议，格式兼容）。用户明确“取消任务，任务数据不删除”，覆盖本次AD-TM-05清理默认设计。卡片统一接管/取消任务按钮，不创建含义相同的两取消按钮；取消仅更新合法状态并追加事件Outbox，任务说明/历史/归属/幂等映射保留，全部可查。执行过任务仍须停止确认，接管记录门禁不变。

撤销task.delete/TaskDelete/Deleted响应和所有清理用例，不保留不可达的危险清理实现；tm-s5-terminal-delete变更withdrawn，不Archive/Done。AD-TM-05保留历史但Superseded。原产品未来用户明确删除历史的独立需求不自动套用当前按钮。

当前研发正式库已升级实验schema4，但只读检查证明两任务、deleted标记0，未发生实际清理。兼容迁移在同一IMMEDIATE事务检查所有tasks.deleted=0且events.kind均transition，移除未使用的两实验列并回到schema3；说明、任务、事件、序号、Outbox、幂等不变。若检测已删除标记/非transition事件或不匹配格式则拒绝启动并保持数据，不能凭空恢复或抹掉删除事实。未知版本仍拒绝。新库及schema2按原schema3初始化迁移，不再升级4。变更只允许去掉未使用列，不清理记录。

验证：临时schema4样本迁移前后六项事实一致、带标记格式拒绝无写、取消仍保留正文/事件且创建重试返回cancelled；原生卡片接管禁用/取消入口与历史可见。本机两真实任务及说明计数保持，Windows暂缓。完整停止接管/记录Story不Done。

AC RETAIN-01卡片仅接管/取消任务、不清理记录；RETAIN-02格式4无标记兼容回3保留全部事实；RETAIN-03标记/错误格式拒绝不写；RETAIN-04取消后的全部历史和创建幂等仍有效。来源为用户本次澄清、架构事实源/幂等及AD-TM-06。

## Agent命名与接管工作定位（2026-09-14用户变更）

接管工作定位按AD-TM-07：请求先冻结准入并确认Driver停止，再按可信当前步骤Observe/资源引用定位窗口，交由对应CU/BUA Port前置与系统桌面切换。TM拥有流程状态、CU/BUA拥有目标操作结果、DS只显示；不直接由UI执行任意Agent路径/命令。工作关闭或身份不唯一时保持暂停和明确失败，不自动创建文档或释放未知租约。多显示器/Space身份协议与原生Adapter能力待CU/TM联合设计，不直接实现。

## MVP步骤边界控制（AD-CU-02，2026-09-14）

来源为原产品执行原则/逐任务控制、架构每步Observe及用户SDK-only人工接管/工作定位变更。冻结新动作→等待当前步骤返回并Observe→结果已知且attempt/Worker一致后确认停止→前置对应工作→满足隐私及显式记录条件后接管。边界等待显示正在停止，不提前显示已接管。超时/断连/Observe失败保持unknown与占用，不自动重试或录制；旧身份回调不释放新租约。合成延迟控件测试未成立，不作为额外产品前置；具体控制协议/持久化/目标及Recording仍待联合定稿，真实步骤竞态/停止/Observe和两平台证据门禁保留，Windows暂缓。

## 工作定位原生路线审阅输入（2026-09-14）

来源：本Story既有产品简报Task Space/逐任务权限映射、用户接管时前置正在操作工作与SDK-only变更、Accepted AD-CU-02/AD-TM-07/AD-CU-03。验收FOCUS-01停止确认后才定位；FOCUS-02可信当前任务目标唯一前置并保持工作屏幕原位置；FOCUS-03关闭/失效/不唯一保持暂停，提示“未能定位任务工作，请手动打开”，不前置同名工作；FOCUS-04小龙不移动，BUA仍外部引用。

macOS隔离正常前置、最小化恢复与关闭拒绝已验证；SDK本身无精确focus接口，产品沿CU内部原生AX/CoreGraphics路线。WindowServer可见和AX就绪分开；界面显示“正在定位工作”直至实际焦点/可见状态确认，返回成功不能提前显示已移交。定位失败保持暂停，显式记录条件仍按RC门禁；UI不接受PID/路径/任意Agent命令。

独立verification-focus-macos.md仅技术基线通过。生产当前WorkRef/attempt/Worker/启动身份、双侧唯一映射、事务事件和多Space/显示器及权限责任链尚未完成，不据此把控制协议设计标ready或启用接管。Windows用户暂缓，完整Story保持原状态。

## 当前联合契约设计

[AD-TM-08](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-08-EXECUTION-IDENTITY-AND-STOP.md)补齐候选task/step/attempt/Worker/宿主/control身份、控制与派发的唯一排序点、动作后Observe、停止提交前不释放占用、工作引用失效和事务回滚矩阵。Application/SQLite持有当前执行与控制，CU仅报告可信事实；本机UI不能自报停止或窗口身份。MVP存储遵循Accepted AD-ST-01未加密SQLite，既有SQLCipher原要求延期，不覆盖加密库。

评审尚缺AG步骤/动作接入去重、Rust Port与迁移、原生进程启动身份/保留AX对象复核Spike及宿主权限、RC记录前置。共享设计保持Proposed，本次无协议或数据迁移，不启用接管。
