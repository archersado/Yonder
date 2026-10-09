# EX-S2 架构设计

## 边界与依赖

AG-S1 的慢脑接入边界已由 Accepted [AD-AG-07](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-07-SLOW-BRAIN-GATEWAY-INGRESS.md) 固定；以下计划字段与事务仍须本 Story 联审后才能实施。

本子范围的配置界面已由 Accepted [AD-EX-03](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-EX-03-JEV-CONFIG-INTERFACE.md) 定案；它只交付非敏感配置的读取、校验和保存，不授权 Jev 执行。

当前实施基线保持 AD-EX-01 的“每步 Jev 决策”。[AD-EX-04](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-EX-04-BOUNDED-ACTION-RECIPE.md) 保持 Proposed，仅作为后续对照观察路线；在 Accepted 前不进入 EX-S2 实施、协议、迁移或运行时代码，且须计划片段契约修订后才能另行创建对照 OpenSpec。

2026-09-24 用户变更：先实施 macOS-only 远端 Jev 决策接线，Windows 后补。Application 拥有有界候选请求、置信策略和交回策略；macOS Adapter 从系统凭据入口读取 API Key，经固定 `@typesafe-ai/sdk@0.6.0` 调用配置 base URL 下的 `/v1/systemone`。2026-10-09 后模型标识、置信度字段和阈值均由 Application 配置快照提供；本地模式允许经校验的 HTTP/HTTPS base URL，远端模式仍只允许 `https://api.typesafe.ai`。API Key 不进入 Jev 配置、SQLite、任务数据、日志或 UI；Windows 路径不编译、不注册、不验证。

## 跨 Story 联审前置

本 Story 的设计不重排既有 Epic：AG-S1 的传输与身份、AG-S3 的归属 Agent 步骤声明、TM-S2/TM-S7 的任务事实、CU/BU 的模型无关 Driver 与 Bridge、DO/FI/CM 的文件锁与命令围栏都先保持现状。EX-S2 定稿计划片段、版本/CAS、内部快脑步骤来源、事件/Outbox 交回和可区分错误矩阵。AD-AG-09 已授权 macOS CUA 的同步有界连续消费基线：一次 Gateway 调用可消费片段剩余槽位，但每个动作仍独立 Observe 并在下一副作用前复核控制；TM-S7 的统一启动门禁继续适用，其他 Driver、Recipe、后台循环与 Windows 不进入本次实施。

慢脑从现有 Agent Gateway 提交首次计划与 replan；本地仍经 MCP stdio/CLI 与 Local Socket，云端仍经单一出站 WSS，两者汇入同一 Application 用例。计划至少绑定 `task_id`、归属 `agent_id`、`request_id`、计划版本、允许能力/资源范围、目标完成判据、截止时间及决策/动作预算。字段以 EX-S1 和 TM-S7 联审为准，最终仅在 Rust 协议类型定义并派生 Schema/TS。Yonder Application 复用统一执行入口，定义一次「从当前 Observe 枚举候选→决策→授权/准入→派发→Observe→提交结果」用例及一个快脑决策 Port。Jev 只由 Adapters 实现此 Port；Adapter 不调用另一 Adapter。快脑需要 replan 时，将有界 Observe/失败依据经既有事件与 Outbox 交给归属 Agent，新计划仍须由 Gateway 进入；不直连慢脑或创建第二会话。

与 AG-S1 的联审候选保持同一契约基线：Gateway 先校验可信会话、身份一致、任务归属、能力、`expected_sequence`、计划版本和 `deadline`，再交给 EX-S2 的计划用例。`request_id` 用于幂等；重复请求在未启动前返回原结果，启动后返回进行中或明确冲突。EX-S2 不新增第二套身份、连接或状态源。

当前 AG-S3 `task.step.declare` 只接受归属 Agent，CU-S2 `computer.step` 也以 Agent 调用推进。快脑连续执行需由 Application 在已验证计划内使用独立可信内部来源写下一步骤/attempt，事件同时保留计划版本、归属 Agent 和实际决策来源；不能复用 Gateway 的 Agent 身份来伪造请求。新增来源及其幂等、事务和冻结排序先与 AG-S3、TM-S2、TM-S7 联审并更新对应 ADR/OpenSpec。片段完成只产生待验证结果/交回事件；现有 `task.complete`/`task.fail` 仍由归属 Agent 经 Gateway 提交。

Jev 配置由 Application 拥有并通过既有桌面命令提交；独立 Jev 设置窗口只渲染和收集，不直接读写 Adapter、配置文件或模型端点。非敏感配置包含启用状态、AD-EX-02 定稿的服务形态与端点、受限模型标识、全局步数/时间/token 上限、CUA/BUA/Document/Command 能力上限；实际片段取计划授权值与全局上限的较小值。配置在片段开始时形成快照；片段运行中修改不改变已冻结片段，但关闭 Jev 等同用户控制，必须立即冻结下一次决策并交回慢脑。无效配置拒绝保存/启用并给出字段级错误。API Key、证书与刷新令牌不属于本配置模型；API Key 仅允许设置面板以非空密码框提交给 macOS Keychain Adapter，读取接口只回传布尔状态。UI 与日志不得展示或持久化完整端点查询串、密钥和模型请求正文。Task Space 不承载配置交互。

macOS-only 接线不新增任务状态。Application 校验配置、能力范围和 2..10 个有界合法动作候选；Jev 只能在这些已由本轮 Observe 校验合格的候选 ID 中选择并返回置信度。AX/UIA 自然候选选择请求由 Application 补齐任务名称形成的整体目标、片段内全部步骤标签、当前步骤序号与标签；缺少合法任务名称时不得调用 Jev，必须交回慢脑补全上下文。协议声明且 Driver 可点击的文本框、组合框与既有按钮、链接、列表项均可进入自然候选；超过上限时，Application 优先按完整标签词命中、再按连续字符相关度稳定排序，因此同时支持拉丁分词与无空格 CJK 文本，相同分数保留 Driver 原始顺序。Application 不把 `ax-element-<index>` 等 Driver 句柄直接作为模型选项，而是从已脱敏短标签中提取与任务/步骤相关的词形成唯一语义别名；只在当前调用栈维护别名到动作候选的映射。Jev 返回别名后必须先通过同一映射复核，再向 Driver 注入原 `observation_ref + element index`；未知、重复或失效别名一律交回。该过程不硬编码应用、控件索引、坐标或业务词。Adapter 不自行查询任务或拼接历史。该上下文、别名和映射只用于本轮选择，最多十个步骤，不进入 SQLite、事件、Outbox 或日志，也不得携带任务说明、正文、坐标和完整 transcript。`handback` 是 Application 拥有的唯一安全边界，不作为可与合法动作竞争的模型候选；低置信、未知候选、不可派发候选或缺少发送确认一律交回，不自动重试。交回与任务序列、事件、Outbox 及受限 `next_intent` 同一事务提交；归属慢脑消费 Outbox 后经既有 Gateway 重新 Observe、规划或执行，Yonder 不作本地兜底。模型失败、凭据缺失、依赖缺失或超时仅返回可区分错误，仍由未来片段执行用例统一映射到既有状态与事件。API Key 从设置页密码框一次性进入 macOS Keychain Adapter；状态查询与写入不依赖 `TaskHost` 或 SQLite，UI 只能查询是否存在，禁止经 `security` CLI 参数、环境变量或临时文件传输。

配置字段定稿为：

| 字段 | 类型与范围 | 说明 |
|---|---|---|
| `enabled` | bool | 是否允许新片段使用 Jev。 |
| `service_mode` | enum：`local` / `remote` | 服务形态，首版只开放 AD-EX-02 定稿的取值。 |
| `endpoint` | URL | SDK base URL；macOS-only 远端固定为 `https://api.typesafe.ai`，SDK 再调用 `/v1/systemone`；不保存查询串、fragment、用户名或密码。 |
| `model` | string，1..128 字节，字符集 `[A-Za-z0-9._:/-]` | `/v1/models` 发布的模型标识；候选选择与步骤核验使用同一配置快照值，Worker 不设第二默认值。 |
| `confidence_field` | string，空或 `[A-Za-z_][A-Za-z0-9_]{0,63}` | 决策门禁读取的响应顶层字段名；空值运行时解释为 `confidence`，已配置字段缺失或非法时交回，不读取其他字段。 |
| `confidence_threshold` | number，`0..1` | 候选选择与步骤核验共用的最低置信度。 |
| `step_limit` | integer，`1..100` | 每片段最大步数。 |
| `time_limit_ms` | integer，`1000..600000` | 每片段最大执行时长。 |
| `token_limit` | integer，`1..100000` | 每片段最大模型 token 数。 |
| `capabilities` | array，仅可含 `CUA` / `BUA` / `DOCUMENT` / `COMMAND` | 全局允许的执行层上限。 |

非敏感配置持久化为 SQLite `jev_config` 单行 JSON，由 Application 在同一事务内校验并原子写入；不写任务、事件或 Outbox。保存成功仅表示配置已通过结构校验，不代表模型可用，运行时仍须在派发前检查当前配置和外部服务可用性。

既有 JSON 缺少 `model` 时以 `jev-latest` 反序列化；旧 `confidence_metric` 字段兼容迁移到 `confidence_field`，两者都缺失或界面留空时按用户约定使用 `confidence + 0.75`。Application 的运行时校验接受合法 `local` 配置，不再重复施加远端固定端点门禁；`remote` 白名单仍由 `JevConfig::validate` 统一执行。Adapter 将 `endpoint + model + confidence_field` 原样放入一次性 Worker 请求，选择与核验不得分叉；Application 使用同一快照中的阈值做门禁。

## 状态与契约

2026-09-30 增量将敏感消息意图放入Application拥有的有界内存Registry。Agent通过协议1.35提出意图，得到`intent_ref`与`confirmation_ref`；计划片段只引用二者。协议1.36补齐`focus-message-composer`，与`focus-target-search`、`enter-target-query`、`activate-target`、`draft-message-ref`、`send-message`共同形成封闭动作语义。协议1.37为窗口动作注入SDK支持的精确窗口target。只有Driver明确确认且同次窗口Observe有效的视觉聚焦才登记一次性焦点凭据；`unverifiable`直接交回且不得让后续引用文本消费。Driver工具名和参数仍由Rust协议校验，Jev不接收敏感值或元素/坐标。

协议1.38为企业微信会话搜索增加封闭`focus-target-search + hotkey(["cmd","f"])`候选。Rust协议同时校验动作语义、工具名和唯一参数值；Worker跳过元素解析但仍注入精确窗口target并强制后置窗口Observe。确认结果可在同片段继续；`unverifiable`直接交回且不登记焦点。该能力不是通用键盘宏，其他组合键仍被拒绝。

协议1.39依据Accepted AD-CU-07开放两个受保护文本语义的有限`x/y`。Worker瞬时解析引用，为坐标`type_text`及视觉`click`强制注入精确窗口target和`foreground`；Agent不能控制delivery mode或正文。动作级前台投递由Driver恢复原前台，Yonder仍以后置同窗口Observe与confirmed共同决定推进。

Desktop组合根先成功取得`host.lock`并完成TaskHost初始化，之后才允许绑定`agent.sock`。初始化失败直接终止本实例，不创建、覆盖或删除Socket，避免失败实例切断仍在运行的正式Gateway。

Application在每个槽位派发前按任务归属和期限解析引用，把目标查询或正文仅注入本次内存动作。发送槽位首先把确认切换为本机可批准；未批准时返回`awaiting-confirmation`且不派发，顶部执行浮窗展示对象与正文预览。批准后引用在派发前消费；动作成功、失败、unknown、取消或接管都不会恢复。

2026-09-28 用户变更将 CUA 候选从“工具名与参数”扩展为受限语义声明：`action_kind`、不含敏感值的 `target_ref`、`preconditions`、`expected_observe` 与可选 `confirmation_ref`。Rust 协议仍是唯一来源；Jev 请求只获得动作语义、当前 Observe 的有界摘要及预期条件，不获得接收人、正文、截图或 Driver 参数。Application 在选择前复核前置条件，在选择后解析一次性本地确认并交给既有 CUA Driver；确认不存在、过期、取消或 Observe 不符均交回归属 Agent。发送等敏感副作用不得把确认值、接收人或正文写入片段 JSON、任务事件、Outbox、日志或模型请求。

执行时以有界内存日志承载交回原因、慢脑唤醒和展示投影，后台按序写 SQLite/Outbox；这些异步写入不阻塞 CUA 槽位或慢脑消费。SQLite 已投影状态仍是重启后唯一可恢复事实；未投影记录在重启后按 `unknown`/`interrupted` 处理，禁止重放副作用。快脑持有的 Observe/目标引用只在当前执行身份和有效期内使用。派发前复核最小持久化检查点、授权、租约、目标新鲜度及用户控制标记。动作结束强制 Observe；目标达成须独立验证，不能只信 Jev `DONE`。偏离、低置信、无候选、参数不足、预算耗尽转为「待慢脑 replan」的内存交回事实，具体状态映射复用 TM，不增第二状态机。

慢脑下发单一路径时，该路径是确定性执行授权，不调用 Jev 再次“规划”或允许其无理由交回；Application 直接派发并在动作后 Observe。仅当同一 Observe 产生多个已验证候选时，Jev 才在候选编号中选择下一动作。动态 CUA 动作空间必须来自 Driver 的当前 Observe 并由 Application 最终复核；Jev 不接收或生成控件标识、坐标、正文、接收人或自由参数。

当前 Observe 只在一次 `task.plan.execute` 调用栈内以封闭布尔事实传递。Application 先以这些事实过滤候选：无候选满足时直接交回，唯一候选满足时确定性派发，至少两个候选满足时才把合格候选与事实摘要交给 Jev；不得把候选自行声明的前置条件当作已经成立。每次动作成功仍须由 Driver 完成后置 Observe，Application 再复核候选的 `expected_observe`，不满足即停止片段并交回慢脑。跨调用不复用瞬态事实，恢复执行时须由单候选慢脑授权或重新 Observe。

2026-10-08 修订后，上述调用栈还保留最近一次 `ComputerObservation` 的 `observation_ref + elements`。对慢脑声明的通用控件语义模板，Application 按封闭角色筛选最多十个当轮句柄，不把这些瞬态候选写回不可变片段。唯一句柄由 Application 直接注入并执行；多句柄时 Jev 获得步骤标签、角色、脱敏短标签、由标签派生的临时语义别名和已有条件摘要，但不获得真实 Driver 句柄。选中后 Application 通过本轮唯一映射找到原动作，只在本次内存 Driver 参数中注入索引与引用；Worker 动作前重新 Observe 复核。完整 transcript、截图、坐标、窗口身份和敏感文本不进入 Jev、事件、Outbox 或日志。

连续推进由 Application 在一次 `task.plan.execute` 调用栈内完成，最多消费当前不可变片段的剩余槽位。每轮只派发一个动作，成功 Observe 与 attempt 停止边界提交后才推进 `current_slot`；下一轮重新读取片段、任务 sequence 与控制事实。返回响应后不保留执行线程或定时任务。片段完成返回 `fragment-complete`，但不替代归属慢脑提交 `task.complete`/`task.fail`。

2026-10-08 按 Accepted AD-EX-05 增加动作后的快脑步骤核验。Application 先检查
Driver 的 `action-succeeded / observe-valid / target-visible` 等封闭事实；通过后，将步骤
标签、动作语义、预期条件、结构事实与本轮 transcript 派生的有界脱敏摘要交给同一
Jev Port，取得 `complete / incomplete / uncertain` 三值结论。只有高置信 complete 才
推进槽位；incomplete、uncertain、低置信、依赖失败或摘要不可用均停止片段并交回慢脑。
观察摘要只驻留当前调用内存，不含截图、坐标、Driver 参数、消息正文、接收人、密码框
或完整 transcript，也不得进入事件、Outbox、SQLite 和日志。该结论只决定当前步骤，
片段结束后仍须由归属慢脑按 AD-TM-24 提交最终目标核验。

产品 MCP 必须从同一 Rust 协议类型构造 `task.plan.submit` 与 `task.plan.execute` 请求，直接转发到既有私有 UDS Gateway；不得在 CLI 重写计划校验、执行循环或 Jev 选择。MCP schema 只镜像协议字段，Gateway 继续负责身份、版本、CAS、期限、候选和能力检查。该入口使 Codex 成为真实慢脑，不创建第二 Agent 通道。

取消、接管、用户输入先冻结快脑新决策，再按既有步骤边界停止/Observe/事务确认。副作用超时、崩溃或断连为 `unknown`，原样保留结论且不推进旧片段；handback 与旧 attempt 的停止边界在同一事务提交，避免已结束 Worker attempt 永久占据执行槽。慢脑读取新鲜事实后只能以新 step/attempt 提交新片段，不能恢复旧候选、旧计划或自动重发。重启 running→interrupted。敏感操作继续走显式用户确认，不因高置信豁免。

交回提交成功后，Desktop组合根从Gateway响应提取受限的任务ID、sequence和交回原因，在释放TaskHost锁且把响应写回调用方后，通过AG-S5现有`AgentInputHub`向唯一匹配`owner_agent_id`的已连接会话投递`source=replan`输入。输入只要求慢脑调用`task.get/task.events`取得新鲜事实并经原Gateway提交新片段，不夹带截图、正文、联系人、Driver参数或完整事件。无连接、多会话、拒绝或超时不改变任务事实、不重试副作用，只在顶部浮窗展示“等待慢脑连接/慢脑唤醒失败”。同一交回序号只投递一次；慢脑的新片段仍受身份、CAS、期限和能力校验。

## 失败与验证

macOS-only 子路线验证 Application 候选/置信策略、Keychain 凭据边界、SDK Worker、组合根和 Windows 不注册。计划片段执行则另行验证 Domain、协议、SQLite/Outbox/Gateway 以及 Windows/macOS 原生取消、接管、权限、网络断连与恢复；它仍以 EX-S1、TM-S7 与完整执行层门禁为前置。
