# TM-S1 架构设计

来源校正：产品简报与补充材料要求完整 Task Space 资源、时间线、产物及审计；架构主干要求步骤、结果、错误、等待原因、恢复信息与外部引用。下文已有核心契约不等于需求全覆盖。AD-TM-01 已撤回“仅最新详情”，保留字段/限额/版本待决，不授权实施。完整来源映射见本 Story 产品需求「需求来源」。

## 边界与依赖

Application→Domain/Protocol，SQLCipher 当前状态/事件/Outbox 同事务；承接旧 OCT-S1 核心记录。

依据 AD-OCT-01/02/04：Domain 纯计算迁移，Application 定义用例和 TaskStore，Adapter 完成 SQLCipher 持久化。AG 校验请求与绑定身份，DS 只展示结果。代码对应 crates/domain/src/lib.rs、crates/application/src/lib.rs、crates/application/src/query.rs、crates/adapters/src/task_store.rs、crates/protocol/src/lib.rs。

## 状态与契约

SQLCipher 保持任务当前事实源；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

### 状态迁移

| 当前状态 | 动作 | 结果 |
|---|---|---|
| created | Start | running |
| running | Pause / WaitForUser | paused / waiting-for-user |
| paused / waiting-for-user / interrupted | 显式 Resume | running |
| running | Interrupt | interrupted |
| running | Complete / Fail | completed / failed |
| 任一非终态 | Cancel | cancelled |
| 任一终态 | 迁移请求 | 拒绝 |

迁移只是状态计算，Cancel 不证明执行器已停止，Resume 不授权自动输入。unknown 是动作结果不确定语义，不擅自新增为任务终态；实际停止与派发归 TM-S2/S3/S4。

### 持久化与序号

tasks 含 id、owner_agent_id、state、sequence；events 含 previous/state/sequence；outbox 引用对应事件。创建和迁移各自同事务写三表；迁移同时比较旧序号和旧状态，提交成功后才返回快照，不提前发布事件。

数据库序号为正 i64，对外为十进制字符串，避免 JS 精度损失。Domain 当前使用 u64，Adapter 拒绝超过数据库范围的写入；验收保留边界拒绝，不能承诺 u64 全范围持久化。

### 查询契约

- list：按 ID BINARY 升序，after_task_id 排他游标，limit 1..100，至多读 limit+1；默认排除 completed/failed/cancelled，include_finished 可包含历史。
- 归属过滤先于分页；下一游标只基于授权结果。翻页不是长事务快照，新增/筛选变化需从首页刷新。
- get/events：先校验可读归属，他人和不存在统一 NotFound；events 从 after_sequence 之后有序返回，逐任务分页。
- AuthContext 来自可信宿主，不能从请求 agent_id 构造。真实认证与 hello 属 AG-S1，本 Story 不对外开放接口。

### 恢复顺序

宿主获取单实例所有权 → 阻断执行 → 每批恢复 1..100 个 running → 每任务 Interrupt/CAS 事务 → 重复至 0 → 开放执行。中途失败阻断启动，已提交 interrupted 保留，不重放动作。调用 open 不自动执行恢复；库函数通过不证明宿主顺序正确。

### Agent 等待用户首批增量（2026-09-18）

按 Accepted AD-TM-10，协议 1.17 只在最后 attempt 已 Observe并推进为 stopped、无 pending control 时接受 `task.wait_for_user`。SQLite schema 14 将规范化后的等待原因写入同一状态事件；状态、事件、Outbox 原子提交后才释放 Admission。旧会话剥离 `wait_reason`。Resume、用户回答路由和等待外部响应不在本增量。

## 失败与验证

AC01–09 对照 Domain/Application 和真实 SQLCipher 双连接、Outbox 故障注入与恢复测试。非法参数拒绝；Conflict 先重读再决策；StorageUnavailable 不返回成功。所有验证用临时合成库，不碰用户数据。Windows/macOS 库层与原生宿主证据分开记录。

AC10 已按 AD-TM-01 子范围定案：`name/source/current_step/observation/next_intent` 分作者写入，缺失为 `null`，同事务更新当前值、事件与 Outbox；`task.get` 返回完整快照，`task.list` 只返回有界摘要。AC11 已按 AD-TM-02 定案：`running_state/activity_state` 全量读取任务表和唯一 Admission，任一已知忙为 Busy，读取失败为 Unknown。不能用空字段、假步骤或当前页面计数填补；失败不得隐式重试未知副作用。

## 架构影响

本次仅细化设计，运行时无变化。新增元数据/汇总如改变持久化或协议，先更新 AD-OCT-01/02，再生成对应 OpenSpec 增量。旧 Change 的跨模块证据仅保留追溯，后续 AG、DS、TM-S2 另建 Proposal。

## 设计就绪门禁

### 保留与去重边界

AD-TM-01 明确：当前任务/审计元信息不自动套用附件 7 天或上下文 90 天 TTL；去重随任务记录保留。附件过期保留事件并报告引用失效，不让旧提交变成新动作。用户明确删除历史后保留无正文的最小防重放标记，禁止 task_id 复用；其字段/隐私仍待 ST/AG 联审，不能直接实现。

删除活动任务历史前确认执行停止；本地逻辑删除、索引/Outbox 处理与清理登记同事务，物理附件/远端删除异步完成但必须显示真实进度。总配额未定前不开放无限创建或自动清理；本轮没有修改现有数据库生命周期。

### 有界读取设计

AD-TM-01 已补最小历史事件与受控引用。list 仅摘要，get 仅当前有界详情，历史/大量产物分别分页。工程限额建议为事件编码后 8 KiB、完整 JSON 响应 256 KiB；历史页先限制 1..100 条再限制字节，不能只靠字段字符数估算内存。元数据与状态一次写事务可能有多事件，各分配连续序号，任务当前序号为最后一条。

reference_id 仅是当前授权下的受控解析入口，不是原始路径或凭据。快照不能把历史产物已生成解释为当前位置/版本仍有效。事件 payload、引用版本与具体协议字段继续在统一 ADR/Rust 来源中定案，不在 UI 定义第二套类型。

### 标识与提交设计补充

依据 AD-TM-01「执行标识设计」「作者与状态允许矩阵」「重复投递、冲突与迟到结果」：task 是持久任务，step 是已登记声明，attempt 是宿主派发前登记的一次结构化动作，request 是一次提交，sequence 是该任务已提交版本。没有第二个同义 action_id。单任务至多一个未确认停止的动作尝试，多任务仍按资源并行；这是明确的本期设计限制，不是产品原文要求。

去重键由 task_id、可信作者和 request_id 组成，同事务记录；先查当前权限再返回原回执，首次提交才检查 expected_sequence。不能按摘要相同折叠真实观察，不能因换请求 ID 重写同 attempt 的结论。创建幂等仍归 AG。具体观察子操作 payload、去重寿命和 schema 仍待定，不代表已实现。

### TM-S1/TM-S5 联审结论

最新值存当前记录，影响审计的事实存有类型的事件或版本引用；两者同事务及同任务序号，Outbox 不复制正文。收到事件 n 后读取详情 n+1 是合法的新快照，不能用它冒充事件 n 的历史。原事务未提交时不能对外发布成功。

用户确认可以发生在任务终态之后，保持执行终态不变，独立增加确认事实及最新序号。真实重复投递与相同内容的两次真实观察不同，幂等不能仅靠文本比较。Agent 声明、Driver 观察、用户确认分别校验作者，不允许任意 metadata patch。

外部文件动作与数据库不构成原子事务；产物成功而记录失败进入待核实流程，不自动重放/删除。迟到事实不得覆盖当前步骤，也不得静默丢弃，具体步骤/执行尝试标识和操作允许矩阵仍待 TM-S2 联审。

详见 [联合设计复核](../TM-S1-TM-S5-DESIGN-REVIEW.md) 与 AD-TM-01；这些是设计复核，不是实现验证。

### 必须同步审阅的接口责任

| 协作方 | TM 接收/提供的信息 | 验证要求 |
|---|---|---|
| AG | 可信发起者、授权引用和经校验请求；提供授权范围内快照与事件 | 撤销后拒绝继续访问，历史授权记录不能绕过当前校验；具体协议归 AG |
| TM-S2/能力 Adapter | 资源稳定引用、步骤执行/观察结果、准入/等待事实 | 引用失效不换目标，迟到结果不能覆盖新步骤；结果未知保持未知 |
| TM-S3/S4 | 接管、停止确认、显式继续和恢复事实 | 原子记录状态和控制事件；不把取消请求当停止完成 |
| TM-S5 | 与快照对应的历史事实、结果/错误及产物引用 | 最新状态与历史同任务序号关联；Outbox 故障整体回滚，不提前发布 |
| DS-S2 | 列表摘要、详情、等待/恢复信息、全量忙碌查询 | 非最新/不可用明确，前端没有写真实状态的权限 |

这些是信息职责，不是已接受的 Rust 字段定义。授权引用格式、稳定资源 ID、步骤与动作的层级、历史 payload/AAD/保留策略及汇总查询仍须 ADR；本轮不新增 Port/表/协议类型。TM-S1/5 必须一起确定持久化信息，不能先丢弃事实再期望事后重建审计。

仍为 design-review：AC10 元数据与 AC11 全局忙碌契约未关闭，宿主集成边界待联审。已有核心测试不能替代本 Story 完整验收，也不授权继续功能代码。

## Agent命名与接管工作引用（2026-09-14用户变更）

按AD-TM-07，名称作为Agent提供的任务元数据进入统一状态事实源，可信归属、同事务事件/Outbox/幂等不变；不能由UI概括说明代替名称。接管工作引用来源可信执行/Observe，生命周期和当前步骤绑定，不能仅持标题或Agent自报PID。协议/字段/迁移仍待技术ADR，不据产品边界先建表。

## 名称子范围设计审阅通过（2026-09-14）

依据用户“任务名称根据对应任务让Agent创建”及Accepted AD-TM-07名称技术定稿；承接前节来源与NAME验收映射。名称字段、1.3兼容、schema5备份迁移与幂等行为按该AD实施，不扩大到尚未定稿的接管目标。菜单/详情显示Agent名称，历史记录显示“未命名历史任务”并保留任务ID；标题支持完整无障碍文本，不引入创建/改名入口。协议错误-32602、版本错误-32010、幂等冲突-32009可观察，任何错误不写任务/事件/Outbox。名称子范围ready，完整Story既有门禁不变。
