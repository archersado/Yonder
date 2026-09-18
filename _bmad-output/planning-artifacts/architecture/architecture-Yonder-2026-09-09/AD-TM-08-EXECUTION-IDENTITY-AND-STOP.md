# AD-TM-08 执行身份与步骤边界停止联合设计

状态：Accepted（TM-S2尝试、结果与连续步骤边界；TM-S3控制停止与macOS接管定位；Recording仍未授权）；日期：2026-09-17。
Architecture Impact：architecture-change（执行上下文、控制确认、持久化与内部Port）；关联TM-S2/S3/S5、CU-S2、DS-S2、AG-S2、RC-S1。

## 来源与设计边界

- 原始需求：产品简报「MVP 主干链路」第6～9项、「Task Space 与权限模型」；补充材料「CUA 与 BUA 的 Task Space」「执行原则」。对应逐任务控制、动作后观察、时间线和审计。
- 后续用户变更：仅Agent创建任务；人工接管记录用户行为并交回Observe；接管时前置正在操作的工作；仅SDK、轻量桌宠menu、多任务并行。
- 已接受约束：AD-CU-02步骤边界停止、AD-CU-03原生精确定位、AD-TM-03/07人工交接与工作定位、AD-ST-01 MVP未加密SQLite。Windows暂缓不代表通过。
- 下述身份字段、阶段名和事务结构均为技术设计建议，不称为原产品字段。既有任务状态保持，不以新增执行阶段代替任务事实源；AD-TM-01全量详情/历史及AG生产认证门禁不被本AD越过。

## 唯一所有者与身份

Application持有任务控制与派发准入；SQLite当前表是事实源。CU Adapter持有SDK调用及系统对象，Supervisor监管自身Node Worker。UI只消费已提交快照，不接受PID、路径或任意窗口命令。

| 标识 | 生成者与用途 | 失效规则 |
|---|---|---|
| task_id / step_id | 任务既有身份；步骤由授权Agent声明并经Application接受 | 重投递不生成新步骤；未实现声明契约前不派发 |
| attempt_id | Application为一次明确接受的动作尝试生成 | 与请求ID分开；未知副作用禁止自动生成下一尝试 |
| worker_instance_id | Supervisor每次启动生成，不以PID替代 | Worker重启必变；旧实例结果不能确认当前实例 |
| host_session_id | 单实例宿主本次启动生成 | 重启后旧回调/句柄不可用 |
| control_id | Application一次接受的暂停/接管/取消控制生成 | 同一执行上下文重复接管返回既有控制，不重复定位或录制 |
| work_ref_id | CU从本次可信Observe生成的不可伪造内部引用 | 窗口关闭、对象替换、Worker/宿主重启均失效 |

所有动作、结果、Observe、停止确认同时绑定task/step/attempt/Worker/宿主身份。控制再绑定control_id，当前快照提交仍比较task sequence。身份一致不是认证替代；只允许受监督通道提交，Agent不能自报已停止。

## 派发与停止的线性化顺序

单任务最多一个未确认停止的尝试，桌面全局租约仍唯一，独立后台任务不因接管被停止。

1. 取得资源准入；检查权限、deadline、步骤和当前控制。在同一宿主派发临界区中持久化本次尝试及准备派发事实，再交付唯一一次动作。控制与交付共用此排序点，不能在锁外先检查、稍后任意派发；不跨SDK等待持有互斥锁。
2. 同步接受控制时先冻结新增派发，再同事务提交当前控制、递增sequence、追加控制事件及Outbox。未能提交控制也保持保守冻结并提示失败，不能继续执行。排序点前已交付的步骤属于在途，后来的控制等待它结束；排序点后不再交付动作。
3. 在途步骤等SDK返回后必须Observe。无在途步骤也必须重新Observe和复核工作目标。返回动作成功不能跳过Observe；目标丢失或退化空树不能当有效观察。
4. CU报告动作结论已知、Observe有效、当前身份一致且派发已冻结。Application同事务提交边界停止事实、对应任务paused/cancelled状态、事件及Outbox；确认提交后才释放该Agent执行占用。人工接管继续保持全局桌面接管闸，不向另一桌面任务放行。
5. 暂停只停止；取消保留全部任务数据，不定位、不Recording。接管在停止提交后才调用定位Port，定位完成必须新鲜核验真实焦点/可见状态，再在事务中发布定位结果。Recording仍须RC显式开始、隐私、来源与配额前置，不由本设计自动启用。

SDK shutdown/Node退出仅是生命周期事实，不能制造停止证明。动作准备已持久化但不能证明是否交付时按unknown处理，不自动补发；准备阶段不能只靠内存“没看到返回”判为未执行。

## 可观察阶段与失败矩阵

执行控制阶段与任务status分开，作为当前控制字段及事件存储：等待边界停止、停止已确认、正在定位、定位成功、定位失败、结果待核实。阶段不是第二套任务状态机，也不允许UI写入。

| 情况 | 接受结果 | 占用与用户反馈 |
|---|---|---|
| SDK已返回且Observe有效 | 等待停止提交，提交成功才确认 | 提交前显示正在停止；随后按控制种类处理 |
| 动作超时/断连/崩溃/Observe失败 | unknown，不确认安全停止 | 保留占用并冻结新动作；显示结果待核实，不自动重试 |
| 旧attempt/Worker/宿主/control回调 | 拒绝，不追加当前任务事件 | 不释放新占用，不切换窗口 |
| 停止提交I/O或CAS失败 | 不发布停止成功 | 保留占用；读回当前事实后处理，不重放副作用 |
| 定位目标失效/不唯一/权限失败 | 任务保持暂停，记录定位失败 | 提示“未能定位任务工作，请手动打开”；不改选同名窗口 |
| 定位超时且焦点结果不明 | 不显示工作已打开 | 不自动重发定位；只读复核，不把窗口操作当任务恢复 |
| 宿主重启 | running恢复为interrupted，未完成控制待核实 | 旧WorkRef不可用；恢复前新鲜Observe并由Agent显式决策 |

普通用户输入立即冻结新动作，但不自动定位或开启Recording。终态/新控制与原步骤结束竞态须由当前sequence和控制身份裁决；迟到完成不能把等待接管改成completed。

## WorkRef与原生定位Port

内部调用语义为“Observe当前尝试→返回工作引用”“确认当前尝试步骤边界停止”“前置已确认停止控制绑定的工作”。最终类型只在Rust定义，JSON Schema/TS在实施时派生；本文不手写第二套协议模型。

CU内存引用绑定当前执行身份、进程PID及系统进程启动身份、WindowServer窗口ID、保留的AX窗口对象及最近有效观察。标题/几何仅用于初次双侧唯一匹配，不能作为长期身份证明；重用PID/同名窗口/相同frame不构成有效目标。副作用前再次核对原对象存活、双侧唯一映射与进程身份，失效拒绝；不能重新按同名选择替代窗口。

定位恢复最小化并前置所属Space/显示器，保持frame；返回前核验新鲜AXFrontmost、FocusedWindow和可见性。小龙锚点不变，不另开App、不使用AppleScript，不复制BUA Task Space。BUA工作引用另由既有Bridge契约负责，未支持时返回能力不可用，不回退CUA。

持久化只存必要执行身份/引用标识及可用性事实，不序列化AX对象、句柄或连接；重启后一律不得把持久化引用当可操作对象。

## 事务、审计与兼容

建议新增每任务当前执行/控制记录，与tasks当前状态、追加事件、Outbox同事务提交；历史保存已接受尝试、控制请求/停止确认/定位结果的有界核心事实。不得仅覆盖最后控制而丢掉历史。引用不含正文/截图/输入/完整命令输出，正常日志也不写这些内容。

首批TM-S2子范围不增加外部协议minor：Agent协议1.4的步骤声明仍只表达意图，不能自报attempt或执行结果。Application内部`ExecutionAttempt`固定包含task_id、step_id、attempt_id、worker_instance_id、host_session_id、phase和accepted_sequence；首批phase只有`prepared`，不伪造dispatched、observed或stopped。

SQLite由schema6升级7，新增`task_attempts`；主键(task_id,attempt_id)，外键(task_id,step_id)指向已接受步骤，accepted_sequence外键指向同事务Start事件，部分唯一索引保证每任务最多一个phase不为stopped的尝试。`prepare_attempt`在单个IMMEDIATE事务校验created、当前步骤、期望sequence和无其他活动尝试，原子提交running、Start事件、Outbox及prepared尝试；同一完整身份重试返回原记录且不新增事件，不同身份或旧sequence冲突。旧任务不补造attempt；升级前备份，未知/已有加密库不得覆盖。

## 验收与实施门禁

| 验收 | 对应来源 | 最小验证 |
|---|---|---|
| EXEC-ID01可信结果只属于当前完整身份 | 架构步骤/恢复、AD-CU-02 | 旧Worker/attempt/host/control及伪造Agent确认全部拒绝 |
| STOP-01控制与派发有唯一排序点 | 原产品逐任务暂停、AD-CU-02 | 控制先到不派发；动作先交付只排空这一动作，其他后台任务继续 |
| STOP-02动作和Observe完成才停止 | 补充执行原则、AD-CU-02 | 在途/空闲/观察失败/超时，租约不提前释放 |
| STOP-03控制与状态/事件/Outbox同事务 | 架构事实源、审计 | CAS、Outbox、I/O故障不发布成功，取消不删数据 |
| FOCUS-05精确引用且定位后核验 | 用户对应工作、AD-CU-03 | 进程/窗口替换、诱饵、关闭、最小化及多Space/显示器 |
| TAKE-07停止/定位不等于记录开始或恢复 | 用户接管记录、AD-TM-03 | RC拒绝/证据缺口可见；普通输入不录制，交回不自动恢复 |

AG步骤与动作去重结论：步骤声明只提供经Gateway接受的step_id；attempt只能由Application在真实准入后生成，Agent request_id/idempotency_key不得充当attempt_id。首批可生成TM-S2尝试准备Proposal；真实派发、CU回调、停止控制、Yonder宿主辅助功能权限、定位与RC联合前置仍须后续Proposal和证据。Windows验证暂缓保留；本AD不宣布CU-S2/TM-S3 ready、Done或Archive。

## 2026-09-16 尝试结果事务定稿

在 AD-CU-04 的真实派发与强制后置 Observe 通过后，TM-S2 可实施结果子范围。`prepared` 只允许一次转为 `observed` 或 `unknown`：`observed` 表示动作结论已知且后置 Observe 有效，并单独记录动作成功/失败；`unknown` 表示超时、崩溃、断连、非法回包、身份不符或 Observe 失败。只保存分类事实与完整执行身份，不保存输入、窗口树、截图或 SDK Payload。

结果提交在单个 IMMEDIATE 事务内比较当前 running 任务 sequence 与完整 attempt 身份，递增任务 sequence，追加 `running→running` 事件及 Outbox，并更新 attempt phase/result_sequence。相同结论重投返回原事实，不新增事件；不同结论、旧身份或旧 sequence 冲突。Outbox 失败全部回滚。`unknown` 和 `observed` 均仍是未停止尝试，不能释放租约；TM-S3 提交停止事实后才能结束活动尝试。

SQLite schema7→8 重建 `task_attempts` 约束，保留全部 prepared 记录，不补造结果。协议 1.5 在 `task.events` 增加可选 `attempt_result`；1.4 及更低响应必须剥离该字段，避免旧客户端收到未知字段。该结果是外部 Agent 的 Observe 分类依据，不包含语义 replan。Windows仍暂缓，停止/接管/WorkRef/Recording未获本增量授权。

## 2026-09-16 步骤边界停止事务定稿

首批停止仅接受当前 `observed` attempt；`prepared` 表示动作尚未形成结果，`unknown` 表示副作用待核实，两者都拒绝停止确认并继续占用。暂停与接管把 running 任务转为 paused，取消转为 cancelled 且保留全部数据。接管的工作定位和 Recording 必须等待本事务成功，首批不自动执行。

Application 由当前任务 sequence 生成任务内稳定 control_id，并携带完整 task/step/attempt/Worker/host 身份调用 Store。SQLite schema8→9 为 attempt 增加 `stopped` phase、stop_sequence、control_id 与 control_kind；在单个 IMMEDIATE 事务内 CAS 当前 running/observed 身份，提交任务迁移、事件、Outbox 与 attempt stopped。相同 control/kind 重投返回原事实，不追加事件；不同控制、旧身份或 Outbox 失败拒绝/回滚。

执行 Permit 只有在停止事务提交后显式释放；提交失败、identity mismatch 或 unknown 时通过错误值交还原 Permit，Drop 仍不释放。当前单次 Worker 已在返回结果前完成动作和后置 Observe，故 observed 是首批步骤边界事实；不得从 SDK shutdown 或进程退出单独制造 observed。此增量不新增外部协议或 UI 入口，不声称接管定位/Recording完成，Windows继续暂缓。

## 2026-09-16 外部控制请求定稿

协议1.6新增 `task.control`，种类为pause/cancel/takeover。可信LocalUser可请求已有运行任务，Agent仅可请求所属任务；请求只登记pending控制并冻结该attempt后续派发，不确认停止。响应明确返回pending和当前running快照，UI显示“正在停止”。created取消继续使用1.2 `task.cancel`。

SQLite schema9→10新增 `task_controls`；请求与任务sequence、`running→running`事件、Outbox同事务。control_id由当前attempt accepted_sequence稳定生成；同种控制重投幂等，不同控制冲突。schema9停止事务必须匹配pending控制并把它同事务更新为stopped。外部请求不持有或释放Permit；执行器确认成功后才释放。首批按钮仅接通pending请求，不启用定位或Recording。

## 2026-09-16 连续步骤边界定稿

没有pending控制时，当前attempt只有在`observed`且动作结论已知后才能推进到普通完成边界；`prepared`或`unknown`拒绝推进。推进在同一IMMEDIATE事务内比较完整执行身份与task sequence，追加`running→running`事件和Outbox，并把attempt标记为`stopped`、记录stop_sequence但不写control_id/control_kind。它不改变任务running状态、不释放任务级资源Permit，也不等同暂停、取消或接管。

推进成功后，所属Agent可为running任务声明下一步骤；声明必须确认最新attempt已普通停止、没有pending控制，并以同事务追加步骤事件/Outbox。随后Supervisor使用同一任务Permit准备新attempt；prepare在running状态只接受最新声明步骤、无活动attempt和当前sequence，追加`running→running`事件/Outbox。首个created步骤仍沿用原created→running事务。

SQLite schema10→11只放宽stopped attempt的约束：普通边界允许control字段全空，控制边界仍要求control字段齐全；历史控制停止记录原样保留。Application新增`advance_after_observe`与`prepare_next_attempt`内部用例，不新增外部Agent协议。该增量是CUA/BUA连续动作的共同前置，Gateway动作、外部Task Space引用持久化和Worker监管另走后续Story。

## 2026-09-14 工作身份子范围证据

macOS隔离复核采用proc_pidinfo启动秒/微秒、保留AX对象并核对当前AX列表成员、WindowServer新鲜几何/标题双侧唯一映射；原生正常/最小化、关闭、同名同frame替换与进程重启拒绝全部通过。映射不唯一与同PID不同启动时间/权限拒绝的合约负样本通过，后两项不是系统权限切换或真实PID复用实测。只读AX就绪/几何匹配可有界等待，不重发副作用、不改选同名对象。

独立Goal：openspec/changes/e0-compare-cua-drivers/verification-work-identity-macos.md；证据spikes/cua-driver-comparison/evidence/work-identity-macos-20260914/fresh-mapping/result.json。限于原生身份路线隔离子范围，不接受整个AD-TM-08字段/Port/事务，也不外推正式宿主、多Space/显示器或Windows。产品接管/Recording仍未启用。

## 2026-09-17 接管定位技术定稿

协议1.13在既有`ControlRecord`增加可选`focus_phase`与`focus_failure`，只投影Yonder提交的定位事实；旧协议移除新增字段。SQLite schema13在`task_controls`保存`locating/focused/failed`当前阶段、失败分类及对应序号，任务sequence、同状态事件和Outbox同事务递增。阶段不改变paused任务状态，也不启动Recording。

CUA每次有效Observe后由单实例宿主从同一个可信WorkTarget捕获WorkRef，并只在内存保留原生对象；不持久化PID、窗口ID或AX对象。宿主重启、Worker/host身份变化或新attempt会释放旧引用。当前同步CUA步骤在Observe后已处于安全边界，因此用户控制可直接原子登记为stopped并暂停任务；在途attempt仍沿用pending→停止确认，不借此伪造停止。

接管停止提交后，宿主先释放该任务执行占用，再设置全局桌面接管闸，提交`locating`，调用CU WorkFocusPort，最后提交`focused`或稳定失败分类。定位失败仍保持paused与桌面接管闸，提示用户手动打开；副作用结果不明不得自动重发。暂停/取消不定位，取消仍保留数据。Recording与交回清除接管闸归RC-S1/TM-S4。
