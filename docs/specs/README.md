# Yonder 技术模块 Epic 总规划

唯一规划入口。流程：Epic → Story 产品需求/架构设计/视觉交互设计 → OpenSpec Proposal → 实现 → 独立验证 → Archive。月份不是 Epic。

拆解输入为既有产品简报、补充材料与架构材料，见 [需求来源与覆盖差距](REQUIREMENTS-TRACEABILITY.md)。本目录是派生规划，不能替代原需求；当前 33 个 Story 尚未覆盖完整产品，禁止按代码现状缩减主干目标。

依据 [AD-DEV-01](../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-DEV-01-MODULE-EPICS.md)，旧记录保留追溯，新增目录不代表设计已通过。

| Epic | 技术模块 | 职责 |
|---|---|---|
| [DS](epic-DS/README.md) | 桌面宿主与桌宠 | 透明常驻窗口、桌宠生命周期、任务总览展示；不拥有任务状态。 |
| [AG](epic-AG/README.md) | Agent Gateway 与接入协议 | 本地 IPC、CLI/MCP、云端客户端接入和 Rust 派生协议；不实现云端服务。 |
| [TM](epic-TM/README.md) | 任务生命周期与资源调度 | 任务事实源用例、状态机、事件、准入和执行生命周期；不内置规划器。 |
| [CU](epic-CU/README.md) | Computer Use Driver | 模型无关桌面动作、Observe、停止及用户输入接管。 |
| [BU](epic-BU/README.md) | Browser Use 集成 | 复用 ego-lite Task Space 和状态映射，不复制浏览器执行模型。 |
| [CM](epic-CM/README.md) | 命令执行 | 结构化命令、进程树停止、超时和输出限额。 |
| [FI](epic-FI/README.md) | 文件操作 | 规范路径与文件身份、受控读写、回收站及原子替换。 |
| [DO](epic-DO/README.md) | 文档处理 | Document Port 与 OOXML 保真读写、文件锁和另存。 |
| [CX](epic-CX/README.md) | 上下文采集与检索 | 授权窗口/浏览/文档上下文、隐私排除与有界采集。 |
| [VI](epic-VI/README.md) | 语音交互 | Windows/macOS显式音频采集、转写会话与可见语音状态；不实现Agent或任务规划。 |
| [RC](epic-RC/README.md) | 手动录制与回放 | 手动开始、不可变时间线、轨迹审阅与确认回放。 |
| [ST](epic-ST/README.md) | 存储与数据安全 | SQLCipher/FTS5/附件、事务和凭据；密钥相关工作暂停。 |
| [EN](epic-EN/README.md) | 工程规范与交付 | 研发围栏、规划关联、CI、验证及发布证据。 |

## 当前研发顺序

ST-S3已按AD-ST-01实施显式未加密任务存储，10项Adapter回归通过，本机核心验证PASS；DS-S2可复用该入口，可信桌面身份/恢复与正式UI仍待接线。加密与迁移保留MVP之后ST-S2待办。

2026-09-14用户变更：MVP暂不加密，按Accepted AD-ST-01采用未加密SQLite；加密、系统凭据与明文迁移归ST-S2，MVP之后实施。DS-S2真实数据接线不再依赖密钥，仍须满足可信身份、恢复与既有任务事实源要求。

先完成 EN-S1 规划与门禁迁移。其余历史实现进入所属 Story 的 design-review，补齐设计差距后才能继续 Proposal；不再往 OCT-S1 混合 Change 追加跨模块功能。DS-S3 菜单与 ST-S2 密钥保持暂停。

后续主链为 TM-S1/AG-S1 设计审阅 → TM-S2 与 FI-S1 资源身份/执行契约 → DS-S2 统一展示；桌面仍依赖 DS-S1 的 E0 门禁。TM-S3/S4 为暂停接管与显式恢复，必须按前置顺序实施。

本轮补拆 [TM-S5 时间线、产物与审计](epic-TM/story-TM-S5/README.md)，与 TM-S1 同期联审当前快照和历史写入边界。只有需求分工和设计初稿完成，没有生成新实施 Proposal。

## 历史迁移

- `E0-S1-DESKTOP-FOUNDATION` → [DS-S1](epic-DS/story-DS-S1/README.md)，保留 `e0-validate-desktop-foundation` 证据。
- `E0-S2-CUA-DRIVER` → [CU-S1](epic-CU/story-CU-S1/README.md)，保留 `e0-compare-cua-drivers` 证据。
- `E0-S3-EGO-LITE-BUA` → [BU-S1](epic-BU/story-BU-S1/README.md)，保留 `e0-defer-ego-lite-bua` 证据。
- `E0-S4-OOXML-ADAPTER` → [DO-S1](epic-DO/story-DO-S1/README.md)，保留 `e0-compare-ooxml-adapters` 证据。
- `E0-S5-WINDOWS-CONTEXT-NATIVE-MESSAGING` → [CX-S1](epic-CX/story-CX-S1/README.md)，保留 `e0-validate-windows-context` 证据。
- `E0-S6-ENCRYPTED-CONTEXT-STORAGE` → [ST-S1](epic-ST/story-ST-S1/README.md)，保留 `e0-validate-encrypted-context-storage` 证据。
- `OCT-S1-TASK-STATUS` → [TM-S1](epic-TM/story-TM-S1/README.md)，保留 `oct-s1-task-status` 证据。

TM-S6 承接 TM-S1 AC11 的全量数据库查询子范围，按 AD-TM-02 独立实施；未关闭完整忙碌/隐藏门禁。

2026-09-13 用户明确暂缓 Windows 验证。当前继续 macOS 可独立推进的验证；Windows 证据门禁保留，暂缓不等于通过，不据此将 DS-S1 标记 Done 或将基础栈 ADR 转为 Accepted。

2026-09-14：AG-S1首批可信会话到TaskHost查询接线实施，关联ag-s1-host-gateway-query；AG-S2拆解Agent专属任务创建，依据AD-AG-01禁止人工创建。创建协议/认证/幂等门禁尚待联审，不以测试任务代替真实接入。

人工接管/交回补充按AD-TM-03分工到TM-S3/S4/S5、RC-S1及DS/AG：只Agent创建不限制用户接管，接管记录与交回Observe必须成闭环，协议/采集/停止确认设计定稿前不实施。

AG-S2首批ag-s2-agent-create-guard已落实Application仅Agent创建门禁，19项核心回归通过；真实task.create协议、幂等、认证传输继续设计，不将内部创建能力宣称为外部接入完成。

AG-S2当前实施ag-s2-local-task-registration，Accepted AD-AG-02定稿协议1.1及schema2→3幂等事务；私有stdio本地测试Agent实际创建两任务，25项分层回归及正式桌面构建通过。首批核心PASS，生产认证/正式宿主IPC和原生桌宠联动尚未验证，Windows继续暂缓；不Archive完整Story。

AG-S1增量ag-s1-desktop-private-stdio已按AD-AG-03完成macOS Debug私有Agent接入正式小龙，真实登记两任务，原生悬停/面板刷新/移出收起通过；仅研发入口，不替代生产本地认证。下一步继续AG-S1生产连接身份设计及AG-S2正式注册闭环，再实施对应执行/取消能力。

2026-09-14当前卡片操作ds-s2-task-card-actions及数据保留tm-s3-cancel-retain-data首批PASS：接管/取消直接位于卡片，用户明确删除只取消、数据不删除，AD-TM-06撤销清理实验。26项核心回归/原生卡片历史验证通过；正式两任务已取消且说明/事件/Outbox/幂等保留。接管停止/Recording仍待实施，生产认证和Windows继续保留门禁。

2026-09-14接管前置推进：CU-S1固定trycua原生macOS只读生命周期、已提交调用取消/关闭竞态及自管SDK子进程监督/断连/恢复三个独立Goal PASS，证据位于e0-compare-cua-drivers。用户明确仅SDK集成，Accepted AD-CU-01撤回上游App/可执行文件前置；完整包已下载但未运行，裸构件拒绝证据保留。下一研发项为SDK路线隔离原生动作/停止/Observe及Yonder宿主权限验证，随后定稿CU-S2/TM-S3/RC-S1联合契约，不能直接启用接管Recording。

SDK隔离原生输入推进：STOP-IN01–03当前macOS测试宿主权限、固定标记后台输入/动作后Observe、动作间关闭后拒绝新输入及750ms目标状态稳定独立Goal PASS，首次测试窗未构建启动失败证据保留。当前继续执行中动作中断/Worker残留/交付来源及正式Yonder权限归属验证，不再核验上游App，未开始用户Recording或修改真实任务。

2026-09-14最新产品变更见Accepted AD-TM-07：Agent创建任务名称，菜单按真实名称展示；接管确认停止后定位正在操作的工作，在所属显示器/Space前置，小龙锚点不移动，失效保持暂停并明确提示。AG-S2、TM-S1/S3、DS-S2三份设计已回写；名称协议/迁移与目标定位技术契约仍待联审，不以记录规格冒充已实现。

MVP接管停止语义按Accepted AD-CU-02：冻结新动作→等待当前步骤返回及Observe→确认步骤边界停止→定位对应工作→满足记录前置后人工接管；unknown保留占用。合成延迟控件排空Goal失败，证据保留，不作为额外前置或宣称通用原生中断已验证。下一可独立实施项为AG-S2 Agent任务名称协议/迁移定稿，完整接管仍有控制/定位/Recording门禁。

2026-09-14：当前AG-S2名称子范围已完成设计审阅，按Accepted AD-TM-07技术定稿实施Rust1.3/SQLite5/轻量菜单名称。接管目标与Worker/attempt协议仍是下一项，不将名称通过等同完整接管通过；Windows按用户暂缓。

2026-09-14 Agent名称子范围已验证通过：真实本地Agent创建→SQLite→卡片/详情，全链路名称一致；31项回归和macOS原生证据完成。下一项继续TM-S3/CU-S2的接管停止确认与可信工作目标定位设计/协议接线，Windows仍暂缓，名称不等同完整任务执行通过。

2026-09-14 接管工作定位macOS隔离基线已通过（verification-focus-macos.md），Accepted AD-CU-03：SDK动作+CU内部原生窗口焦点，禁止frame调整冒充定位。下一项定稿并实施TM-S3/CU-S2的attempt/Worker/当前WorkRef与步骤边界停止确认；真实工作所属Space/显示器、宿主权限与Windows仍保留验证门禁。

2026-09-14 当前接管主线已补齐TM-S3/CU-S2三份设计与AD-TM-08执行身份/步骤边界停止候选契约：派发冻结排序、可信停止确认、WorkRef失效及失败矩阵明确。AD仍Proposed，未新增协议/迁移或启用接管。下一项为AG步骤/动作接入去重与Port/事务联审，以及进程启动身份/保留AX对象复核限时Spike；原生权限、多Space/显示器及Windows门禁保持。

2026-09-14 工作身份复核macOS隔离Goal PASS：保留AX对象与进程启动身份，关闭/同名同frame替换/进程重启旧引用拒绝，正常及最小化恢复通过。失败诊断保留；真实PID复用/系统权限变更仅合约负样本，正式宿主权限/多Space/显示器/Windows仍待验。下一项继续AG步骤/动作接入去重及AD-TM-08 Rust Port/控制事务/迁移联审，不把Spike通过当完整接管完成。

2026-09-15：AG-S3首批步骤声明已按Accepted AD-AG-04进入验证。协议1.4、SQLite6及私有stdio Agent的创建→声明→读取→取消后重试通过；步骤只是不可变声明，不派发动作、不制造running/Observe或接管事实。下一项继续AD-TM-08的动作尝试与停止事务联审。

2026-09-15用户调整优先级后先完成AG-S1本地接入增量：Accepted AD-AG-05、macOS生产UDS、安装包内`yonder mcp`和真实Codex CLI创建/读回任务均PASS，当前处于verifying。Windows Named Pipe按用户要求暂缓；完整AG-S1不Archive。下一项可回到AD-TM-08动作尝试与停止事务联审。

2026-09-15已按Accepted AD-TM-08完成TM-S2执行尝试准备子范围：Application完整执行身份、schema7原子Start/事件/Outbox/attempt、准入失败释放与35项回归PASS；正式库6→7保留14任务/1步骤且0虚构attempt。下一项进入CU-S2真实Driver派发与动作后Observe，随后TM-S3才能实施步骤边界停止。

2026-09-15 CU-S2首批真实派发/Observe已通过：真实 prepared attempt 经产品 Rust CU Port、Yonder监管的trycua 0.25.0无界面Worker完成隔离后台AX输入，后置Observe与原生控件一致，36项回归PASS。下一项先把动作/Observe结论事务写入attempt阶段，再进入TM-S3步骤边界停止；产品Node/SDK打包、正式宿主权限与Windows门禁保留。

2026-09-16 TM-S2结果事务子范围PASS：schema8原子提交observed/unknown、`running→running`事件与Outbox，协议1.4隐藏/1.5展示有界attempt_result；真实macOS动作落库且占用保留，正式库7→8保留14任务/0 attempt并备份。下一项进入TM-S3步骤边界停止，unknown仍不得释放或重试。

2026-09-16 TM-S3内部步骤边界停止子范围PASS：schema9只允许当前observed attempt原子暂停/接管或取消，unknown/旧身份/Outbox失败保留Permit，事务成功后才释放；正式库8→9保留14任务/0 attempt并备份。下一项实现外部控制协议和任务卡片接线，定位与Recording继续后置。

2026-09-17新增[AG-S4 Yonder Agent Skill](epic-AG/story-AG-S4/README.md)规格：统一BUA、CUA、Document与Command用法，BUA嵌入ego-browser规则并只使用Yonder关联的ego-lite Task Space。按用户要求当前不生成Skill或OpenSpec；等待四类Gateway能力全部实现并验证后再实施。

2026-09-17 TM-S3接管定位增量PASS：协议1.13、SQLite13、宿主WorkRef与任务卡片已接通；真实UDS Agent经trycua执行后，接管提交`paused/stopped/focused`并前置原任务窗口，Recording未启动。下一项按围栏进入RC-S1手动录制前置设计审阅；跨Space/多显示器与Windows证据继续保留。

2026-09-17 RC-S1已完成首轮设计审阅并建立Proposed AD-RC-01与`rc-s1-recording-capture-spike`：先用无正文统一样本验证macOS用户来源、隐私排除、有界队列和停止边界，不接产品协议/SQLite/回放。Windows按用户决定暂缓，双平台门禁与完整Story保持未完成。

RC-S1首轮macOS来源验证FAIL：真实CUA注入的51个事件全部被CGEvent来源字段误判为用户候选，纯Event Tap启发式已淘汰且未接产品。下一候选验证IOHID硬件事件与CGEvent单调时间关联；不匹配事件归`external_unknown`且不生成轨迹。

RC-S1第二候选也未通过：IOHID关联可保守拒绝CUA注入，但三轮真实物理正样本在20/100ms窗口内均不能稳定证明用户来源。来源门禁未满足，停止后续Secure Text/持久化实现，产品Recording保持关闭；Proposed AD-RC-01等待来源契约重审。

2026-09-17 RC-S1显式接管意图入口PASS：任务卡片改用打包Task Space专用`user_takeover`命令，Rust宿主生成控制身份和参数，通用JSON入口拒绝takeover；真实macOS操作提交`paused/stopped`。本增量不启用Recording或用户控制租约。

2026-09-17下一Story进入CM-S1设计审阅：TM-S4仍受RC-S1 Recording门禁阻塞，AG-S4又依赖Command能力。建立Proposed AD-CM-01与`cm-s1-command-executor-spike`，先验证结构化参数、输出限额及双平台进程树停止；当前不开放产品Gateway或Shell。

CM-S1 macOS Spike子范围PASS：字面参数不经Shell解释，独立进程组父子进程均停止，无限输出只保留64 KiB并标记截断。Windows Job Object按用户决定暂缓，AD保持Proposed，产品Command Gateway仍未开放。

2026-09-17 BU-S2真实用户任务闭环PASS：Yonder本地MCP创建“查看今日微博热搜”，持久化`ego:49`，Agent在关联ego-lite空间操作后由Yonder完成Observe/推进/finish；任务`completed@14`、引用finished、事件与Outbox均14条，外部空间已关闭。BU-S2进入verifying。

2026-09-17 DO-S1选型证据复核完成并进入verifying：Accepted AD-E0-04已唯一选择Rust进程内实现，Node Worker淘汰。产品Document Port未混入选型Story，新增DO-S2三份设计；其文件写入与Gateway实施以前置FI-S1文件身份、互斥及锁冲突为门禁。

2026-09-17 FI-S1进入design-review：建立Proposed AD-FI-01与`fi-s1-file-identity-spike`，先验证软/硬链接归并、授权根逃逸拒绝、同文件锁及同目录原子提交；Office/WPS真实锁和Windows证据仍是产品接线门禁。

2026-09-17 FI-S1 macOS合成样本子范围PASS：软/硬链接归并、重复写锁拒绝、授权根逃逸拦截和同目录原子提交均通过。Office/WPS真实锁及Windows统一样本未完成，AD-FI-01保持Proposed，FI-S1与DO-S2不接产品Gateway。

2026-09-17 DO-S2内存语义转换子范围PASS：Accepted AD-DO-01下新增Application Document Port和Rust OOXML Adapter，DOCX/XLSX/PPTX有界读取、唯一文本替换、未修改part保真及稳定冲突拒绝通过，48项Workspace回归通过。FI-S1完成前不接文件写入或Gateway。

2026-09-17 FI-S1 macOS WPS真实锁对照PASS：WPS打开DOCX时原生文件引用可见且Rust写锁被拒绝，关闭标签后引用消失且锁可取得。macOS子范围完成；Windows统一样本继续按用户决定暂缓，AD-FI-01保持Proposed且产品Gateway保持关闭。

2026-09-17 CU-S2跨Space应用前置增量PASS：保持trycua `launch_app`后台语义，以SDK已校验bundle id处理主进程PID交接，并仅为同任务显式`bring_to_front`注入可信身份。真实企业微信任务验证`target_visible=false→true`并完成；Windows继续暂缓，完整CU-S2保持verifying。

2026-09-17 VI-S1进入design-review：建立Proposed AD-VI-01与`vi-s1-voice-input-spike`，先用不采集音频的双平台能力探针核验原生框架、中文识别器和权限状态，再验证显式采集与停止释放；不复制参考插件密钥或Node状态机，不提前开放Gateway。

2026-09-18 VI-S1 macOS已授权会话子范围PASS：正式桌宠内显式开始、真实中文转写、无确认自动投递、完成后重新取得麦克风、手动停止和取消关闭均通过；正文与PCM未写入证据。首次权限拒绝/撤权、设备切换与Windows样本仍待，AD-VI-01保持Proposed。

2026-09-18 VI-S1 macOS撤权/设备切换取证未关闭：adhoc预览包在应用级权限重置后仍沿用授权，采集中重置没有产生撤权事件；本机仅有一个输入设备。两项继续保留真实环境门禁，不以模拟结果替代。

2026-09-18 AG-S5 Codex当前turn投递Goal FAIL：Codex CLI 0.154的`codex queue`仅确认持久排队，活动turn没有收到steer，现有桥接不能据此返回`accepted`。不使用内部数据库轮询或第二Agent绕过；等待受支持的当前会话`turn/start|turn/steer`连接入口后返回实施。

2026-09-18 AG-S5失败关闭修复：排队探针仅在前一turn结束后作为下一turn到达；`yonder agent-bridge`已删除`codex queue`误报路径，不支持当前会话直接提交时明确退出，桌宠保持Agent未连接。

2026-09-18 AD-VI-02证据修正：撤回Codex当前会话样本通过结论，模型无关`AgentSession`决策仍Accepted。云端产品WSS由AG-S1先定稿端点、认证、TLS和重连；AG-S5只复用已认证会话，不复制连接或使用Spike的不安全TLS配置。

2026-09-18 AG-S1云端Connector设计补齐：单一出站WSS、认证绑定、系统TLS校验、有界重连及本地能力隔离已写入AC13～19。外部平台配对契约与持久设备凭据仍缺失，Credential Store接线按用户决定延期；当前不生成产品WSS Proposal，不开放匿名或明文凭据连接。

2026-09-18 AG-S1建立Proposed AD-AG-06与`ag-s1-cloud-connector-spike`：先验证Rust进程内WSS、系统TLS、帧上限、断线释放与有界重连；不接产品账号/凭据，不注册产品AgentSession。Windows按用户决定暂缓，单平台证据不接受ADR。

2026-09-18 AG-S1云端Connector Spike的macOS子范围PASS：Rust单进程WSS样本完成可信回显、Ping/Pong、主动关闭后重连、64 KiB帧配置、无效TLS拒绝和有界退避；AD-AG-06因Windows暂缓继续保持Proposed，产品Connector仍受平台契约与凭据门禁阻塞。

2026-09-18 下一可实施增量进入 TM-S5 `tm-s5-readonly-timeline`：复用现有正式 `task.events` 在轻量 Task Space 详情展示状态、步骤声明与执行结果；Architecture Impact 为 conforming，不新增协议/存储。产物、确认、配额、清理和完整分页继续保留设计门禁。

2026-09-18 TM-S5首批只读时间线macOS子范围PASS：正式桌宠详情显示真实状态与Agent步骤事件；ego-browser覆盖observed/unknown、未完整提示和局部失败。同期修复Enter/Space入口按悬停规则自动收起的问题。Windows与完整TM-S5不Archive。

2026-09-18新增[CU-S3 后台原生动作与显式前台切换](epic-CU/story-CU-S3/README.md)：公开原生API/App Intent/SDK不经过CUA Driver，AX/UI与键鼠仍走trycua；所有动作进入同一任务事件链并由Task Space展示。Proposed AD-CU-06与限时macOS Spike已建立，暂不接产品Gateway或通用Adapter，Windows继续暂缓。

2026-09-18 CU-S3 macOS系统API子范围PASS：`NSWorkspace`后台fixture未激活目标应用，Observe、无trycua/无隐式兜底、Task Space最小顺序事实及进程清理均通过。通用App Intent、真实第三方应用SDK、产品事件接线和Windows仍未验证；AD-CU-06保持Proposed。

2026-09-18 CU-S3真实系统计算器对照PASS：应用此前未运行，`NSWorkspace`非激活启动后前台保持，Observe、零新增trycua Worker和正常清理通过。范围仍限于真实应用后台启动，不外推应用内业务动作或App Intent。

2026-09-18 CU-S3 App Intents公开SDK面结论：真实计算器metadata包含动作，但Yonder没有按外部metadata标识执行Intent的公共API；通用App Intent Adapter路线淘汰。Shortcuts CLI保持Command边界，不能作为原生动作隐式兜底。

2026-09-18 CU-S3 EventKit真实副作用子范围PASS：首次提醒事项授权会改变前台，因此必须作为显式等待用户流程；权限已授权后，临时提醒创建、identifier Observe、删除清理与前台保持通过。证据不含提醒内容，产品Gateway仍未开放。

2026-09-18 CU-S3 EventKit最终边界PASS：同一临时App先执行无副作用`authorize`并提交权限事实，再由预先授权的`background`步骤完成创建/Observe/删除，输出四条Task Space最小顺序事实。后台内部请求权限的旧结果已拒绝。

2026-09-18 DS-S2首批任务总览状态校正：macOS可信宿主、真实两任务、轻量菜单及列表/详情/失败保留已有独立证据；新增响应竞态回归证明旧刷新不覆盖新视图。该OpenSpec子范围PASS，完整Story仍保留外部ego-lite关联、完整控制、Windows和PR/Archive门禁。

2026-09-18 CX-S2进入design-review：为“圈选提问”建立Proposed AD-CX-01与`cx-s2-region-capture-spike`，先用无敏感检查图形验证坐标/像素、权限与清场。不复制常驻指针模式，不接产品Gateway；Windows依用户决定暂缓，双平台门禁保留。

2026-09-18 CX-S2 macOS区域捕获子范围PASS：公开ScreenCaptureKit对自绘无敏感窗口完成点/像素与四色校验，连续三次通过，截图不落盘且无运行残留。全新临时bundle的未授权预检返回`permission-required`，未请求权限或截图；非激活选择层的合成Esc与超时清场通过。本机仅单显示器，副屏/负坐标、运行中撤权、显示器变化、物理键盘/真实选择与Windows门禁继续保留，AD-CX-01仍为Proposed。

2026-09-18旧`oct-s1-task-status`混合Change按AD-DEV-01冻结：已实施基础与验证历史保留，15条跨模块待办已分别映射到AG/TM/CU/FI/DS/ST/EN Story，禁止继续在旧Change追加功能。迁出动作完成不代表目标Story验收完成，旧Change不Archive。

2026-09-18旧`e0-compare-cua-drivers`选型Change冻结：trycua 0.25.0唯一结论已进入正式桌面依赖清单、Adapter/Worker与macOS预览资源，Qwen不进入产品依赖树。宿主权限、派发/Observe、停止、工作定位和平台验证由CU-S2/TM-S3独立Change承接；旧Change仅保留对照与失败证据，不Archive。

2026-09-18 CU-S3门禁复核：后台原生动作macOS Spike已完成并停止扩展；AD-CU-06仍为Proposed，按其双平台门禁与用户暂缓Windows的决定，当前不开放产品Gateway、不创建通用NativeActionPort。恢复条件为Windows对等Spike和ADR接受，不以macOS样本绕过。

2026-09-18 TM-S5既有时间线分页macOS子范围PASS：复用现有`task.events(after_sequence)`，Task Space每页20条，失败保留/重试及迟到选择保护通过；正式54条历史任务从`#1..#20`追加到`#40`。未来产物、确认、新payload字节预算与Windows证据仍保留门禁，完整TM-S5不Archive。

2026-09-18 TM-S1进入`tm-s1-agent-wait-for-user`实施：Accepted AD-TM-10定稿协议1.17与schema14，只允许归属Agent在已Observe并推进的安全步骤边界提交等待用户原因；不实现Resume或等待外部响应。

2026-09-18 TM-S1等待用户macOS子范围PASS：协议1.17、schema14、MCP工具与Task Space原因展示完成；活动/unknown步骤不释放占用，1.16隔离有效。正式库110条任务迁移并保留schema13备份；Windows和Resume继续保留门禁。

2026-09-18 TM-S2已观察失败终结macOS子范围PASS：协议1.18 `task.fail`、Gateway、既有SQLite attempt事实、CLI/MCP与桌宠failed出口完成；complete/fail只接受各自对应的Observe结论，unknown不终结。52项Workspace测试和ego-browser Task Space 83通过；Windows继续按用户要求暂缓，完整TM-S2不Archive。

2026-09-18 TM-S4进入design-review：Proposed AD-TM-12明确多任务下用户输入必须从具体任务入口建立关联，通用语音输入不得猜测恢复对象；RC-S1来源与AG-S5当前会话投递通过前不创建OpenSpec或实现Resume。

2026-09-18 新增[DS-S4 桌宠动画资源包导入](epic-DS/story-DS-S4/README.md)：承接产品简报的资源包导入要求，限定为声明式 PNG/WebP ZIP 的本机校验、暂存与原子切换，不触碰任务、Agent 或录制状态；OpenSpec 已建立，下一步定稿限额后实施。

2026-09-18 DS-S4 扩展为自定义形象生成契约：Yonder 定义九个状态及运行时校验，用户上传参考图后由 Hatch Pet 式 Agent/Skill 生成状态动画并输出 Yonder manifest 包；生成外发须明确确认。用户决定当前只保留设计，等待 AG-S4/FI-S1 后单独排期，不创建实现 Proposal 或代码。

2026-09-18 CX-S2进入macOS Preview实施：用户要求先测试“圈选提问”，Accepted AD-CX-01授权当前显示器的显式选择、临时内存截图、确认卡与取消清场；不发送、不持久化、不支持常驻指针、多显示器或Windows。

2026-09-20 CX-S2 macOS单显示器Preview经非实现者独立复核七行矩阵全部PASS，OpenSpec `2026-09-20-cx-s2-macos-preview`已归档。完整CX-S2保持verifying；Agent提交、多显示器、运行中撤权和Windows仍由后续Change承接。

2026-09-20 CX-S2 Agent会话临时附件Spike经非实现者六行矩阵复核PASS，AD-CX-02更新为Accepted：同一AgentSession内保持64 KiB帧和4 MiB附件上限，跨会话隔离及所有结束路径清零；不传本机路径、不持久化截图。产品协议与确认卡发送仍须独立Change。

2026-09-20 CX-S2圈选附件产品提交经非实现者复核PASS并归档：macOS正式bundle覆盖accepted/rejected/unknown/unsupported，同会话分块、哈希、引用和清理通过，任务/事件/Outbox不变且正文截图不落盘。Windows、多显示器与云端WSS仍保留后续门禁。

2026-09-20 CX-S2无截图文字提交经非实现者复核PASS并归档：直接点击、不足最小选区及临时未授权bundle均进入“仅提问”；只声明`user_input`的会话每次仅收到1条无附件`agent.input`，unknown无自动重试。语音组合与CUA占用时先暂停仍待后续Change。
