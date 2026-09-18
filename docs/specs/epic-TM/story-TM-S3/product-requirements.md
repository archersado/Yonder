# TM-S3 产品需求

## 问题与目标

暂停取消仅作用指定任务，用户接管阻断新桌面动作，独立后台任务继续。

## 范围与非目标

本 Story 仅负责“逐任务暂停取消与接管”。尚未定义控制协议和停止确认契约。

## 验收条件

- 目标行为有可复现成功样本，失败不得伪报成功。
- 暂停取消仅作用指定任务，用户接管阻断新桌面动作，独立后台任务继续。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。
- STOP-04：只有当前 observed attempt 能确认步骤边界停止；prepared/unknown/旧身份均拒绝且保持占用。
- STOP-05：暂停/接管转 paused，取消转 cancelled并保留数据；状态、停止身份、事件和Outbox同事务。
- STOP-06：停止事务成功后才释放Permit；提交失败必须交还原Permit，不能因Drop或错误路径放行。

## 待决事项

尚未定义控制协议和停止确认契约。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：Task Space 与权限模型、MVP 主干链路；补充材料 CUA 与 BUA 的 Task Space。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：任务、状态与恢复。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。

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

Rust协议1.2新增task.cancel，capability=task.cancel，参数agent_id/deadline/task_id/expected_sequence（沿用序号字符串）。Gateway协商minor>=2才能调用；1.0/1.1行为不变。响应复用snapshot。可信LocalUser可取消任意created任务，Agent仅所属，越权与不存在同为-32004。请求ID仍JSON-RPC id，不产生第二套模型。

Application校验身份、归属与expected_sequence，再检查created并通过原有CAS状态/事件/Outbox同事务提交Cancel。已cancelled仅expected_sequence等于当前或当前减1时返回既有快照，不新增事件；其他旧序号-32011冲突。非created/非cancelled返回-32012需要执行停止确认（包括paused/interrupted/终态），不得把取消请求当停止证据。并发Start与Cancel由同一CAS序号保护，Start提交失败不派发并释放本次准入，已有未知占用不清除。创建幂等重试返回真实cancelled，不复活任务。

本机控制沿现有固定LocalUser桌面query入口，仅增加task.cancel分支；Agent只走已握手Gateway。UI选择created详情后显示“取消任务”，提交中禁用，成功刷新进行中列表；失败保留并提示刷新，不自动重试，不新增人工创建入口。终态在全部中保留事件和快照；取消不删除数据，不调用系统删除操作。普通点击小龙仍不打开菜单。

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

后续用户变更：点击接管需打开/前置当前任务正在操作的工作，并切到对应屏幕/系统桌面。来源本次用户、原Task Space逐任务控制和架构执行目标身份，Accepted AD-TM-07。FOCUS-01停止新Agent动作并确认安全停止；FOCUS-02可信工作窗口前置/最小化恢复并切到所属显示器/Space；FOCUS-03目标丢失或切换失败保持暂停、提示人工定位，不模糊打开同名工作；FOCUS-04不移动小龙或复制BUA空间，不自动录制普通切屏。停止/目标/权限/Recording技术前置仍保留。

## MVP步骤边界控制（AD-CU-02，2026-09-14）

来源为原产品执行原则/逐任务控制、架构每步Observe及用户SDK-only人工接管/工作定位变更。冻结新动作→等待当前步骤返回并Observe→结果已知且attempt/Worker一致后确认停止→前置对应工作→满足隐私及显式记录条件后接管。边界等待显示正在停止，不提前显示已接管。超时/断连/Observe失败保持unknown与占用，不自动重试或录制；旧身份回调不释放新租约。合成延迟控件测试未成立，不作为额外产品前置；具体控制协议/持久化/目标及Recording仍待联合定稿，真实步骤竞态/停止/Observe和两平台证据门禁保留，Windows暂缓。

## 工作定位原生路线审阅输入（2026-09-14）

来源：本Story既有产品简报Task Space/逐任务权限映射、用户接管时前置正在操作工作与SDK-only变更、Accepted AD-CU-02/AD-TM-07/AD-CU-03。验收FOCUS-01停止确认后才定位；FOCUS-02可信当前任务目标唯一前置并保持工作屏幕原位置；FOCUS-03关闭/失效/不唯一保持暂停，提示“未能定位任务工作，请手动打开”，不前置同名工作；FOCUS-04小龙不移动，BUA仍外部引用。

macOS隔离正常前置、最小化恢复与关闭拒绝已验证；SDK本身无精确focus接口，产品沿CU内部原生AX/CoreGraphics路线。WindowServer可见和AX就绪分开；界面显示“正在定位工作”直至实际焦点/可见状态确认，返回成功不能提前显示已移交。定位失败保持暂停，显式记录条件仍按RC门禁；UI不接受PID/路径/任意Agent命令。

独立verification-focus-macos.md仅技术基线通过。生产当前WorkRef/attempt/Worker/启动身份、双侧唯一映射、事务事件和多Space/显示器及权限责任链尚未完成，不据此把控制协议设计标ready或启用接管。Windows用户暂缓，完整Story保持原状态。

## 执行身份与步骤边界停止验收补齐

联合字段设计见[AD-TM-08](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-08-EXECUTION-IDENTITY-AND-STOP.md)（Proposed，不授权实施）。原始需求映射：产品简报「MVP 主干链路」6～9及「Task Space 与权限模型」→STOP-01逐任务停止、STOP-03历史保留；补充材料「执行原则」→STOP-02动作后Observe。架构步骤/恢复→EXEC-ID01当前执行身份校验。后续用户对应工作变更→FOCUS-05精确窗口、所属Space/显示器与定位结果核验；人工接管记录变更→TAKE-07停止/定位/记录/恢复分开。具体字段与阶段均属待审技术方案，非新增用户需求。

接管不能把SDK返回、进程退出或取消请求当安全停止：当前动作结果已知、Observe有效且当前执行身份一致，SQLite停止提交成功后才能定位；取消仅合法更新状态，全部任务数据保留。身份失效、Observe失败或副作用未知保持占用并提示，独立后台任务继续。
