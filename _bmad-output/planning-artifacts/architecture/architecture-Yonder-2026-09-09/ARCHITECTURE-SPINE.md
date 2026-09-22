---
name: Yonder
type: architecture-spine
status: draft
updated: 2026-09-09
paradigm: 模块化单体、六边形端口/适配器、受监管执行进程
companions: [DEVELOPMENT-AND-CHANGE-MODE.md]
---

# Yonder 架构围栏

研发规划依据 [AD-DEV-01](AD-DEV-01-MODULE-EPICS.md)：技术模块 Epic 目录 → Story 产品需求/架构设计/视觉交互设计 → OpenSpec Proposal → 实现 → 独立验证 → Archive。规划入口为 `docs/specs/README.md`，月份仅作里程碑。

## 产品边界

Yonder 是 Windows/macOS 上轻量常驻的 Agent Tool，提供本地系统操作、用户操作感知和个人上下文采集。它可被桌面 Agent、Agent CLI 或外部云端 Agent 驱动，但不是 Agent、规划器或云端服务。BUA 直接复用 ego-lite Browser Task Space，Yonder 不复制该模型。

## 模块与依赖

```text
React 桌宠/UI ──> Tauri 组合根
                        │
CLI/MCP ──Local IPC──> Agent Gateway <──WSS── 外部云端平台
                        │
                   Application ──> Domain
                        ▲ ports
 Adapters: CUA / ego-lite BUA / Command / File / Document / Context
```

最小 Cargo Workspace：`apps/desktop`、`apps/yonder-cli`、`crates/domain`、`crates/application`、`crates/adapters`、`crates/protocol`。

依赖只能是 desktop→adapters/application，adapters→application/protocol，application→domain/protocol，CLI→protocol。Domain 无技术依赖；Application 不引用具体 Adapter；Adapter 不互调；React 不拥有核心状态。CI 检查 Cargo 图、前端 import 和协议生成。

## Agent Gateway

- 本地 Agent 经 MCP stdio 或 CLI，通过共享 Rust IPC Client 接入；IPC 使用 `interprocess 2.x`、Tokio、版本化 JSON-RPC，映射 Unix Domain Socket/Windows Named Pipe，不开放本地 TCP。
- Yonder 主动建立单一 WSS 连接云端，不开放公网入口、端口映射或 P2P。断线按 `last_sequence` 补传 Outbox，且不影响本地能力。
- 本地与云端请求统一为 `AgentRequest + AuthContext`。Rust 协议类型生成 JSON Schema 和 TypeScript；hello 协商版本与 capabilities。
- IPC 限当前 OS 用户，设备凭据进入系统 Credential Store。MVP 不做细粒度授权；支付、发送、删除、安装、提权仍需确认。

## 任务、状态与恢复

- Execution Session 是 Yonder 内部执行实体；ego-lite Task Space 是 BUA 外部实体；两者映射为统一 Task Status。
- SQLite 当前状态表是唯一当前事实源，配合追加事件日志与同事务 Outbox，不采用完整 Event Sourcing。每任务 sequence 严格递增。
- 持久化状态、步骤、结果、错误、等待原因、恢复信息和外部引用；句柄、连接、窗口对象及订阅仅驻内存。
- 重启后 running 转 interrupted；重新 Observe 后由外部 Agent 决定继续、跳过或局部 replan，禁止自动重试副作用未知动作。
- 提供 task.get、task.events(after_sequence)、CLI status/watch、MCP 查询/长轮询与云端事件推送。
- 每一步后增量 Observe；窗口切换、导航、目标丢失时完整 Observe。Yonder 校验预期条件，不负责语义规划。
- CUA 使用唯一前台租约，用户输入即暂停；BUA MVP 单并发。后台读取可并行，同一文件禁止并发写。

2026-09-18 按 Accepted [AD-TM-10](AD-TM-10-AGENT-WAIT-FOR-USER.md) 增加协议 1.17 等待用户提交：仅归属 Agent 在已 Observe 并推进的安全步骤边界写入 `waiting-for-user`，等待原因、事件与 Outbox 同事务，提交后释放准入资源；不提供自动 Resume。

2026-09-18 按 Accepted [AD-TM-11](AD-TM-11-AGENT-FAIL-TERMINATION.md) 增加协议 1.18 失败终结：仅归属 Agent 可把最新已 Observe 为动作失败并推进至 stopped 的 Desktop 任务提交为 `failed`；unknown、动作成功、pending control 或未停止均拒绝。失败依据复用既有 attempt 事实，不新增自由文本或持久化字段；状态、事件与 Outbox 原子提交后释放资源。

2026-09-18 Proposed [AD-TM-12](AD-TM-12-RESUME-INPUT-ASSOCIATION.md) 补充多任务恢复输入关联门禁：会话输入不是任务输入，恢复前必须由用户明确选择任务冻结关联，再由 Agent 新鲜 Observe 后显式请求恢复；不得以最近任务、语音或当前桌面猜测对象。RC-S1 与 AG-S5 门禁通过前不实施。

2026-09-18 Proposed [AD-DS-04](AD-DS-04-MASCOT-GENERATION-CONTRACT.md) 定义用户自定义形象的生成边界：Yonder 固定九个状态动画契约与本地校验，Hatch Pet 式 Agent/Skill 仅生成并质检素材包；用户确认前不外发参考图，桌宠不内置模型、密钥或生成 Runtime。

## 执行能力

2026-09-14字段级联合设计见[AD-TM-08](AD-TM-08-EXECUTION-IDENTITY-AND-STOP.md)（Proposed）：执行身份、派发/冻结排序、停止确认、WorkRef失效与事务候选契约。尚未授权协议/迁移或产品接管；与Accepted AD-CU-02/03边界一致，未决门禁保留。

### CUA 与 BUA

2026-09-14MVP接管控制语义见Accepted AD-CU-02：冻结新增动作、等待当前步骤返回并Observe，结果已知且执行身份一致后确认步骤边界停止，再定位工作并满足Recording前置。超时/断连/观察失败保持unknown与占用，不自动重试。具体协议/字段仍待CU/TM联合设计，不以SDK关闭或进程退出代替控制确认。

2026-09-14任务体验补充（Accepted AD-TM-07产品边界）：名称由Agent创建并进入统一任务事实源；接管确认停止后按可信当前工作引用前置对应窗口/系统桌面/显示器，目标失效保持暂停、不模糊打开同名工作。名称协议/安全迁移与原生目标定位契约待联合定稿，不自动记录普通切屏。

2026-09-14用户变更（Accepted [AD-CU-01](AD-CU-01-SDK-ONLY-INTEGRATION.md)）：仅集成已选trycua SDK和匹配原生库，不附带上游CuaDriver.app；由Yonder监管的按需Node Worker加载SDK，权限与用户入口归属Yonder。监督/停止/原生动作和宿主权限责任链仍需独立验证，不使用SDK的上游可执行文件宿主作为产品前置。

2026-09-15首批受监管派发见 Accepted [AD-CU-04](AD-CU-04-SUPERVISED-DISPATCH-OBSERVE.md)：Application 从任务事实源取得完整 prepared attempt，CU Adapter 监管单次 SDK Worker，后台 AX 输入返回后强制 Observe；身份不符、超时、崩溃、断连、非法响应或观察失败均为 unknown 且不重试。首批不新增外部动作协议或任务持久化字段。

2026-09-16按 Accepted AD-TM-08 增加结果事务：schema8将 prepared attempt 原子转为 observed/unknown，同事务递增任务sequence、追加 `running→running` 事件及Outbox；结果不释放租约。协议1.5以可选 `attempt_result` 向Agent提供有界分类，旧版本剥离字段，不传输输入、窗口树、截图或完整Payload。

2026-09-16步骤边界停止增量：schema9仅允许当前observed attempt原子转stopped；暂停/接管提交paused，取消提交cancelled并保留数据。Permit在事务成功后才释放；unknown、旧身份或提交失败继续占用。外部控制协议、工作定位与Recording仍需后续规格。

CUA 只含模型无关 Driver，不内置 Agent/Planner/Model。Qwen cua-driver 与 trycua cua-driver 使用相同 Windows/macOS 黑盒用例验证，只交付胜者；OSWorld 仅作基准。Driver 由 Supervisor 按需启动；崩溃或超时将动作标为 unknown，Observe 后交由 Agent 决策。

BUA Bridge 直接调用 ego-lite/ego-browser，保存 external_task_ref 并映射状态，不复制 Task Space。只有经 Yonder 调用才保证桌宠收到状态。由于 ego-lite 当前没有 Windows Runtime，Windows 首版 BUA 为 `capability_unavailable`；不得自动回退到 CUA 或另一套浏览器引擎。

### Command、File 与 Document

Command Executor 与 CUA 分离，默认结构化 program/args/cwd/env；Shell 必须显式启用。支持超时、取消、进程树终止、输出上限、最小环境及脱敏审计。

File Tools 提供打开、定位、元数据、列表、读、复制、移动、重命名、建目录、受控写入和移入回收站；路径绝对化、规范化并阻止穿越。

Document Port 不暴露 XML。OOXML Adapter 处理 docx/xlsx/pptx 的结构读取与局部修改，写请求携带 expected_hash，多操作原子提交。默认另存；覆盖须显式要求；临时写入并校验后原子替换。不得绕过 Office/WPS 文件锁；复杂排版、图表、宏、旧格式和宿主 UI 操作降级 CUA。Rust 进程内与按需 Node Worker 经 Spike 选出唯一实现，达到保真门槛时优先 Rust。

## Recording 与桌宠

Recording 仅手动开始，每设备最多一个。原始时间线不可变，派生版本化 CUA Trajectory；AX 目标优先、相对坐标兜底。输入标记 user、agent_cua 或 replay；只有 user 事件生成轨迹，避免递归录制。

桌宠导入声明式 ZIP，状态含 idle、listening、recording、thinking、executing、waiting_for_user、success、failed、paused。MVP 仅 PNG/WebP，禁止脚本和可执行内容；隔离解压并校验大小、尺寸、帧数和路径。

## 个人上下文

2026-09-14用户变更（Accepted AD-ST-01）：下述SQLCipher、附件加密与Credential Store要求延期至MVP之后；当前使用未加密SQLite，不以密钥接线阻断MVP。原始加密需求保留，事务、授权、隐私和日志约束继续有效。

- 当前窗口：Windows WinEvent Hooks/UI Automation；macOS NSWorkspace/AXObserver。只采元数据并由事件驱动。
- 浏览：BUA 复用 ego-lite 事件；日常记录由 Chrome/Edge 扩展经 Native Messaging Host 上报，不读取 History DB；隐私窗口排除。
- 文档：只处理明确授权或任务选择的文件，由实际活动触发；不全盘扫描。OOXML 复用 Document Adapter，文本/PDF 使用只读 Extractor；MVP 无 OCR。
- 操作序列：仅手动 Recording 采集。密码框、安全界面与排除应用始终忽略。

2026-09-18显式区域捕获候选见Proposed [AD-CX-01](AD-CX-01-EXPLICIT-REGION-CAPTURE.md)：只在用户启动“圈选提问”后建立短生命选择层与临时截图，未启动时不监听全局输入或截图。截图与问题不入日志、任务事件、Outbox或普通索引；双平台Spike和ADR接受前不形成产品能力。

2026-09-20圈选提交附件见Accepted [AD-CX-02](AD-CX-02-SESSION-INPUT-ATTACHMENT.md)：在同一已认证`AgentSession`内以有界分块暂存附件，`agent.input`只引用会话内附件标识；不发送本机路径、不扩大单帧、不持久化截图。隔离Spike与独立复核已通过，产品协议与确认卡发送仍须独立Change验证。

SQLite 存元数据并使用 FTS5；大内容存加密附件。Agent 只能经 Context Port 查询，默认返回摘要与引用。SQLite 使用 SQLCipher，附件文件级加密，主密钥只存 Keychain/Credential Manager。MVP 无本地向量库、Embedding、知识图谱和跨设备密钥同步。

上下文默认保留 90 天，Recording 原始素材 30 天，任务附件 7 天；轨迹与固定内容长期保留。删除同步清理索引、附件和未发送 Outbox。

## 非功能与发布

2026-09-14后续用户变更：当前研发阶段以功能闭环为通过依据；下列性能数值保留为优化目标，暂不阻断功能推进，发布前重新评审。具体阶段决定见AD-E0-01；安全、协议及数据一致性约束继续有效。

- 空闲 CPU 平均低于 1%，内存目标低于 150MB，托盘至可用低于 3 秒，窗口变化至状态更新低于 300ms。
- 重型 Worker 按需启动、空闲退出；解析、加密和索引不得阻塞 UI。
- 日志不得记录正文、截图、输入、完整命令输出或 Agent Payload。
- 一次发布包含 desktop、CLI、MCP、IPC 协议、迁移和 Driver manifest；版本固定，不用 latest。
- 数据库仅向前迁移且升级前备份；更新签名，macOS notarize，Windows code-sign；只设 dev/stable 通道。

## MVP 明确不做

云端服务、内置 Agent/Planner、完整 Event Sourcing、通用 HTTP Server、全盘采集、本地语义知识库、Office COM/AppleScript/WPS 插件和多个 CUA 同时控制桌面。

2026-09-14创建权限补充（Accepted AD-AG-01）：任务只由已连接且认证/握手的Agent经统一Gateway创建，不支持人工手动创建；用户仍可确认和操作已有任务。具体创建协议/幂等存储按AG-S2联审，未定稿不开放写入口。

2026-09-14人工接管补充（Accepted AD-TM-03）：显式人工接管已有任务时记录user来源行为；交回归属Agent前保存证据并新鲜Observe，Agent显式决策后重新准入，禁止自动恢复。普通输入自动暂停不自动开始Recording，隐私排除与单设备一Recording约束不变。

任务登记首批协议/幂等设计见Accepted AD-AG-02：Gateway 1.1的task.create仅登记created，schema2→3追加原子幂等表；本地联调用私有stdio测试夹具，生产连接认证/IPC仍独立门禁。

2026-09-17 Agent输入补充（Accepted AD-VI-02）：显式用户输入通过已绑定`AgentSession`的双工信道发送`agent.input`；本地UDS/Named Pipe与云端主动WSS复用同一协议及确认语义，不使用MCP反向注入，不把输入正文写入任务库、事件、Outbox或日志。

研发私有stdio正式宿主接入见[AD-AG-03](AD-AG-03-DESKTOP-STDIO-DEVELOPMENT.md)，不替代生产本地Socket认证。

未开始任务取消子范围按[AD-TM-04](AD-TM-04-PENDING-CANCELLATION.md)，执行中停止确认仍待TM-S3。

用户已结束任务删除按[AD-TM-05](AD-TM-05-TERMINAL-TASK-DELETION.md)，保留防重放及无正文删除Outbox事实。

2026-09-14用户澄清覆盖AD-TM-05：当前删除入口仅取消、数据保留，按[AD-TM-06](AD-TM-06-CANCEL-RETAIN-DATA.md)撤销清理并兼容未使用实验格式。

2026-09-14 接管工作定位：Accepted AD-CU-03确认macOS隔离基线使用CU内部原生AX/CoreGraphics前置/恢复目标；trycua SDK没有精确focus接口，不能用frame调整替代。生产WorkRef/控制事务/宿主权限和多Space/显示器仍未定稿/验证，不提前启用任务接管或Recording。
