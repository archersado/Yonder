# TM-S5 架构设计

## 边界与依赖

Application 定义历史/产物查询与事实提交用例，Adapter 持久化，Protocol 统一派生，DS 展示，AG 鉴权。与 TM-S1 共用任务标识、当前状态与事件事务；不新建 Task Space 数据库，不从历史事件重建整套事实源。

## 状态与契约

终态结果确认采用 Accepted AD-TM-22：`task.result.confirm` 仅允许可信 LocalUser，并绑定当前不可变产物清单版本与结果序号。首次确认同事务递增任务序号、追加审计事件、写确认记录与 Outbox；相同确认幂等返回既有事实，不改执行状态、不触发 Driver。协议 1.20 与 schema 18 统一承载产物清单和确认投影；Agent Gateway 不宣告或接受该本机控制能力。

早期 AD-TM-14 的 `reviewed_sequence / terminal_result_sequence / task_result_confirmations` 模型已被 AD-TM-22 取代，不得与清单确认并存。历史 Observe、控制、定位与创建来源从协议 1.21～1.24 依次扩展，schema 19 只新增定位历史表，不重写 schema 18 审计事实。

保留设计遵循 AD-TM-01：任务证据附件到就绪后 7 天仅成为清理候选，未同步/固定/活动引用仍保护；用户文件/外部对象不归该清理器。任务结构化历史不自动过期，受总配额和显式删除约束。附件过期只改变引用可用性，不覆写旧事件正文；整任务删除返回不可访问，不伪装完整空历史。

清理先事务标记不可解析并登记持久化工作，物理清理确认后才完成；用户删除已同步历史还需要独立远端删除记录与确认，不继续投递旧正文。新清理/投递实现、容量和防重放标记格式须 ST/AG 定案，本 Story 不自建密钥或云端服务。

清单发布后版本不可变，新增产物用新版本；游标绑定清单版本与内部顺序键，不使用任务事件序号。条目失效保留条目，不能跳过。产物元信息随历史保留，实际内容遵循所属附件/原应用规则。版本无法验证时明确说明，不沿用已确认旧版本的保证。

2026-09-25 核心版本化增量按 Accepted AD-TM-22 复用 schema 18：Application 接受完整清单快照并分配 `1..N` 顺序键，SQLite 在单个 `IMMEDIATE` 事务内创建下一版本、写全部条目、递增任务序号并追加事件/Outbox。每版最多 4096 项，同版 `reference_id` 唯一；空清单仅表达完整集合确实为空。读取用例先按当前 `AuthContext` 校验任务，再以固定 `manifest_version + after_ordinal + limit` 读取 `limit + 1` 项，limit 为 1..100。核心 Port 不暴露路径或正文，不允许 Agent 直接发布。

该增量暂不增加 wire 方法：协议 1.20 的快照/事件仍只投影清单版本和条数。Gateway 查询、真实能力 Adapter 的发布时机、受控引用解析及 Task Space 条目展示需后续 Change，不能以核心表存在替代 AC05 完整验收。

AD-TM-01 的 2026-09-12 补充列出各类最小历史事实；本 Story 复用该单一设计表，不维护另一事件枚举。Observe 通过 attempt_id + 宿主登记的 observe_index 关联，重投递不产生新轮次。终态可引用已完整准备的产物集合清单，不能把部分产物列表当完整结果。

引用由任务库分配 reference_id 并绑定任务、类别、来源与版本依据；路径只用于可信 Adapter 定位，不由事件接收者直接执行。哈希/对象版本须来自实际验证，否则明确 unknown。记录时版本与解析时可用性分离，文件变化不能改写产出时引用。

预算设计：编码后单事件 ≤8 KiB、完整响应 ≤256 KiB；历史页最多 100 条且按字节提前结束，首项超限报错，不跳过或空页循环。事件预算已由 AD-TM-15 实现；核心清单每版 4096 项、内部分页每页 100 项已由 AD-TM-22 后续增量定案，wire 响应字节预算与公开续页字段仍是后续设计。

2026-09-24 AD-TM-15 先对既有 `task.events` 查询投影定案单事件与整响应预算：Application 在版本投影后测量 JSON 字节，返回可容纳的完整前缀；沿用现有 `after_sequence`，不新增 `has_more` 或显式游标字段。上段产物清单、详情预算和探针设计仍未定案。

2026-09-24 Accepted AD-TM-16 定案当前未裁剪历史的连续性保护：Application 先按当前授权读取任务快照，再检查最多 `limit` 条已提交事件是否从 `after_sequence + 1` 连续；不足 `limit` 时以任务当前 `sequence` 判断尾部是否缺失。缺口返回局部查询错误，不返回不完整成功页、不触碰 SQLite/Outbox。未来合法部分裁剪须先增加显式范围协议，不复用本错误作为清理许可。

2026-09-24 Accepted AD-TM-17 定案历史 Observe 投影：Adapter 按 `(task_id, sequence, kind='observation')` 关联既有 `task_presentation_events` 并严格解析有界 payload；Application 只在协商协议 1.21 或可信同版本查询中输出 `TaskEvent.observation`。旧版本字段缺省，事件仍按原序号和字节预算分页。UI 将结果和获准摘要以纯文本标为 Observe，不把动作成功当观察匹配，也不从最新快照补旧事件。此增量不加表、不改变写入方或补造缺失事实。

2026-09-24 Accepted AD-TM-18 定案控制历史投影：Adapter 仅把 `task_controls.accepted_sequence` 和 `stopped_sequence` 关联到相同序号的已提交事件。不同序号的接受事件固定展示 pending、停止事件固定展示 stopped；直接停止同序号仅展示 stopped，不从当前控制阶段反推过去。Application 仅在协议 1.22 投影可选 `TaskEvent.control_event`，旧会话字段缺省；沿用授权、连续性检查及 8 KiB/256 KiB 预算。定位/交回/录制另议，不新增表或写入者。

2026-09-24 Accepted AD-TM-19 定案接管定位历史：schema 19 新增 `task_focus_events`，定位开始和最终成功/失败与既有任务事件、Outbox及当前控制投影同事务追加。迁移不从 `task_controls` 当前值回填旧历史。Application 仅在协议 1.23 投影可选 `TaskEvent.focus_event`；Adapter 按任务与事件序号读取，不从最后阶段反推。定位事实不代表录制或交回完成。

2026-09-24 Accepted AD-TM-20 定案创建来源历史投影：Adapter 只把既有 `task_presentation_events(kind='source')` 关联到同一 `#1` 创建事件，并结合任务不可变归属列形成 `owner_agent_id + source`；没有 source 历史的旧任务不从当前 `legacy` 快照回填。Application 仅在协议 1.24 投影可选 `TaskEvent.creation_event`，旧会话字段缺省；沿用实时授权、连续性检查及编码预算，不输出描述、幂等键或凭据。

2026-09-24 Accepted AD-TM-21 定案执行尝试开始历史投影：Adapter 仅把 `task_attempts.accepted_sequence` 关联到同序号 Start 事件，读取不可变 step/attempt/worker/host 身份并固定表达 prepared 接受事实；不从当前 attempt phase 或任务 running 状态反推。Application 仅在协议 1.25 投影可选 `TaskEvent.attempt_started`，旧会话字段缺省。Task Space 只显示步骤与尝试标识，不显示内部 Worker/宿主身份，且不把 prepared 写成已派发或已观察。

标识与作者规则引用 AD-TM-01 的联合契约：历史步骤/动作按 task_id/step_id/attempt_id 关联，request_id 只负责提交去重，sequence 负责已提交顺序。终态后的用户确认和合法迟到事实可追加序号，但不能重新执行任务或覆盖当前步骤。unknown 后证据作为新关联事实保留，不改写原结论。

请求去重与业务结果冲突分别判断：相同请求内容不同拒绝；相同 attempt 的终结结论不同也不能因请求 ID 不同而覆盖。重复回执可能落后于最新快照，读取方不能回退状态。权限撤销后只由可信宿主接收已派发尝试的结果，不向撤销的 Agent 继续开放查询。观察子操作和去重记录寿命仍须定稿。

联合复核已明确：TM-S1 当前记录保存最近值，TM-S5 历史保留授权范围内的真实事实/稳定版本引用。历史不能只留字段变更通知，也不能从稍后 get 得到的新快照反推旧事件正文。持久化仍是一套按 Accepted AD-ST-01 的 MVP 未加密 SQLite 状态/事件/Outbox，不引入 Event Sourcing。

计划区分声明信息、执行/观察事实、用户控制与结果确认，明确可信写入者。外部 Agent 提供的意图不能写成执行器观察成功；缺失事实不靠模型补齐。每任务序号负责顺序，不以客户端时间戳决定提交顺序。

状态相关的审计事实必须与状态/Outbox 原子记录。元数据变化哪些需要历史正文、哪些可仅留引用，须和 TM-S1 一起决策；AD-TM-01 的失效通知不能替代全部审计。正文不进入应用日志，附件保持加密与受控引用。

设计选择：身份/授权引用保留创建事实；状态、错误、等待/恢复变化保留事件；步骤/意图保留有作者的声明；动作/观察保留执行身份与结论；产物保留产出时版本依据；用户确认引用被确认的结果/产物版本。句柄、连接、租约对象不写入任务库。具体 payload 和版本格式仍待定，不把此表当作已生成协议。

幂等按明确提交标识、作者与内容处理，重复投递不产生第二条事实，同标识不同内容冲突；两次真实观察不能因摘要相同合并。重试前仍校验当前权限。完成后的用户确认及合法迟到事实可增加序号，不重新进入执行状态。

任务完成和用户确认分开记录，不能将确认动作复用为 Resume 或重新迁移已完成任务。具体确认事件、幂等键与当前确认投影未定，不在本轮修改状态枚举。

产物引用由对应能力用例提交，需要区分原应用对象、输出产物与录制引用；路径只是定位信息，不自动代表版本身份。访问引用必须再次检查授权和对象可用性，禁止绕过 Office/WPS 文件锁。

外部产物与数据库不是一个事务：先获得可校验产物/已就绪加密附件，后提交引用及事件；数据库失败不撤销已经发生的外部动作。保留待核实结果，不自动重做或删除用户产物。准备失败不得记录为“产物可用”，临时附件清理与配额须有独立策略。

## 失败与验证

以临时合成任务、可控文件和故障注入覆盖 AC01–09；分别测试重复提交、迟到事件、旧序号、越权、Outbox 回滚和产物失效。历史配额达到上限时不得无提示丢失事实或删除未同步/固定记录；处理策略需明确后才能实施。

事件枚举、payload 限额、附件/版本标识、事务落点、查询分页与删除语义必须先 ADR。保留策略不意味着可以修改仍保留的原始事实；删除后如何表达历史不完整需显式定义。此文件不决定新 schema 或协议版本。

## 架构影响与前置

当前仅设计拆解；实施预期涉及协议/持久化，Architecture Impact 将为 architecture-change，先更新 ADR，再创建 Proposal。真实时间线需 TM-S2/S3/S4 产生事实；本 Story 不伪造执行。录制默认关闭、双平台证据与现有隐私围栏保持。

## 首批只读时间线增量（2026-09-18）

该增量为 conforming：DS-S2 通过现有可信 `task_query` 调用 Rust 派生的 `task.events`，固定 `after_sequence=0`、`limit=100`。不修改协议、SQLite、Outbox、Application 状态所有权或依赖方向；React/页面只保存当前选择和加载轮次。

详情快照与事件可独立读取，但必须用同一选择轮次丢弃迟到响应。事件按服务端序号展示，页面不补写、合并或推断事件；`step_declaration` 标为声明，`attempt_result` 按协议结论显示。事件读取失败保留任务详情并在时间线局部报错。最后事件序号小于详情快照序号时显示未完整加载。

2026-09-18 按 Accepted AD-TM-09 增加既有事件分页：复用 `after_sequence`，Task Space 每页使用 `limit=20`，游标只取最后实际追加事件，用户显式加载下一页；选择轮次继续阻止迟到响应。当前有界事件字段无需新协议。未来产物、附件或可变正文仍受编码后 256 KiB 响应预算门禁，不因本增量解除。

## 完整审计闭环增量（2026-09-23）

依据 Accepted [AD-TM-22](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-22-AUDIT-COMPLETENESS-AND-QUOTA.md)，本子范围补齐用户结果确认、不可变产物清单、审计容量门禁和版本迁移。

### 确认与清单

- `TaskUserConfirmation` 绑定 `task_id`、被确认的 `result_sequence`、`manifest_version`、1..2048 UTF-8 字节确认意见和稳定 `confirmation_id`。只有可信本机控制用例可写入，Agent 不获得确认能力。
- `TaskArtifactManifest` 以 `(task_id, manifest_version)` 唯一，首次为 1；清单发布后不可修改，条目仅包含不可复用顺序键、`reference_id` 和可用性结论。新增产物生成新版本，不篡改旧清单。
- 确认、清单、事件、任务序号和 Outbox 在同一个 `IMMEDIATE` 事务内提交；重复 `confirmation_id` 返回既有回执，不同内容返回冲突，不追加事件。

### 容量与迁移

- 任务审计库上限 2 GiB；写入前可用磁盘必须保留 1 GiB。校验在创建、执行启动和确认前执行，失败返回 `QuotaExceeded`，不触发自动清理。
- schema 15→16 只新增 `task_user_confirmations`、`task_artifact_manifests`、`task_artifact_manifest_items`、`task_audit_quota_state`；迁移前创建备份，事务失败整体回滚，未知版本拒绝。
- 协议 1.20 增加可选 `user_confirmation` 与 `artifact_manifest` 投影；1.19 及以下会话继续收到旧形状。Rust 协议类型仍派生 Schema/TS，不手写第二份模型。

本增量复用已有正式能力，不触发 Architecture Decision；产物、确认、保留清理和新事件 payload 仍须先更新 ADR。

## 人工接管记录与交回（2026-09-14用户变更）

来源：本次用户明确执行中支持人工接管并记录用户行为，作为交回Agent的Observe依据；架构Recording与桌宠/任务恢复约束，Accepted AD-TM-03。仅Agent创建任务，人工接管不创建新任务。

验收映射：TAKE-01显式接管阻止Agent新动作，停止未确认不称已移交；TAKE-02本次手动接管开启可见、可停止的记录，只录user输入，密码/安全界面/排除应用不采；TAKE-03原始时间线不可变并关联原task_id；TAKE-04交回保存证据并执行新鲜Observe，向归属Agent交付轨迹/证据引用及当前状态；TAKE-05交接失败/配额缺口明确反馈、保持暂停，不自动续跑；TAKE-06Agent重新Observe并显式恢复，重新准入后才executing。

TM-S3管接管及停止确认，RC-S1管用户记录/不可变证据，TM-S5管任务时间线引用，TM-S4管交回及显式恢复，AG管归属Agent交接协议，DS-S2展示已有任务接管/记录中/交回。自动输入干预只暂停，不无提示开启Recording；显式接管作为用户手动开始。已有每设备Recording时明确冲突，不覆盖。

UI流程：任务面板“接管”→“正在停止Agent控制/记录中”→停止确认后人工控制；小龙按既有暂停表现并保留独立录制提示。用户可以停止记录，任务仍保持人工接管；交回时如果停止后有未记录操作标注证据缺口，最终Observe仍必须新采集。“交回Agent”不等于立即继续，不用点击桌宠自动恢复任务。

协议/持久化/Driver停止与授权/隐私字段未定稿，相关Story保持设计阶段，不提前实现采集或向外发送用户记录。Windows暂缓、真实双平台证据保留。

## 用户卡片删除首批定稿

# AD-TM-05 用户删除已结束任务

状态：Accepted（MVP本机删除）；日期：2026-09-14。Architecture Impact：architecture-change（协议/持久化）。来源：产品简报Task Space权限模型、架构任务事实源/保留边界、用户要求任务卡片删除；删除语义澄清未获其他选择，采用显式默认清理任务及历史并保留最小防重放标记。不是仅隐藏列表。

用户确认后仅可信LocalUser可删除终态任务；Agent不得删除，活动状态先取消/停止，宿主Busy/Unknown拒绝。UI浏览器原生确认说明任务/历史清理、不能恢复及防重放标记保留；不自动确认真实数据删除。接管/Recording前置不因新增按钮解除。

Rust协议新增task.delete/TaskDelete及DeleteParams（统一身份/能力/deadline、task_id、expected_sequence），复用kind=deleted/task_id响应。此为本机用户控制，不在Agent hello宣称删除能力，不升级Agent1.2。普通query没有真实宿主活动证据时拒绝；正式TaskHost提供自身ActivityState。唯一Rust模型派生Schema/TS。

SQLite4从3同事务加tasks.deleted（默认0）、events.kind（默认transition）。删除同一IMMEDIATE事务：校验原状态/expected_sequence及终态，删除原Outbox/事件，清空task_creations.description；保留task id/owner/state并置deleted=1、sequence加1，插入无正文deleted事件（执行状态不变）及Outbox。其为删除控制事实，不伪装Domain状态迁移。用户明确手动删除历史时处理旧Outbox，新的删除通知仍保留，未来同步消费者必须识别kind=deleted；不自动清理未同步数据。没有轨迹/附件的首批任务仅清理当前已有任务说明/事件；未来附件关联建立前必须更新清理登记协议，不擅自清理文件。

正常list/get/events/running均不可读deleted标记；commit排除deleted。创建幂等遇deleted标记返回-32013任务已删除，不恢复旧描述、不创建新任务。Agent需新幂等键才创建不同任务。终态删除保留最小id/owner/key/state/sequence及删除事件/Outbox，不TTL/自动抹除；不是完整Event Sourcing。

重复删除仅expected_sequence等于删除前或当前时返回既有deleted回执，不增事件；其他旧序号-32011。非终态及Busy/Unknown-32012；权限-32003、不可见-32004；迁移失败全部回滚，错误格式/未知版本拒绝。现有schema2→3迁移后同事务再到4，旧数据保留。无需新依赖/密钥/加密，AD-ST-01保持。

验证：实际文件迁移、四表删除/回滚、序号/身份/活动状态、重复删除与创建不复活；原生卡片删除打开确认后取消，不代替用户确认清理真实任务。实际删除使用独立测试库；macOS截图/日志，Windows仍暂缓，完整TM-S5与接管需求保留。

验收DELETE-01用户终态删除说明与历史，非目标不变；DELETE-02同事务删除事实及Outbox，故障完整回滚；DELETE-03创建重试不复活，重复删除不加事件；DELETE-04用户确认/活动状态/权限门禁不可绕过。来源依次为本次用户变更/原始任务权限模型、架构事实源/事务、原始幂等与历史保留、用户控制与执行停止约束。

> 本轮删除清理设计已由用户明确澄清及AD-TM-06撤销，当前卡片只取消并保留数据；上文AD-TM-05不是实施许可。时间线/产物/审计的原始需求保留。
