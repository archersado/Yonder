# Yonder Agent 实施围栏

本文件对整个仓库生效。所有产出文档使用中文。

详细架构以 [`_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md`](_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md) 为准；研发与变更流程以同目录的 [`DEVELOPMENT-AND-CHANGE-MODE.md`](_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/DEVELOPMENT-AND-CHANGE-MODE.md) 为准。发生冲突时先停止实施并更新架构决策，不得自行绕过。

## 开工门禁

- 采用 SDD：`技术模块 Epic 目录 → Story 三份设计 → OpenSpec Proposal → 实现 → 独立 Verification Goal → Archive`。
- 规划唯一入口为 `docs/specs/README.md`，目录采用 `docs/specs/epic-<模块>/story-<ID>/`；Epic 不按月份组织。
- Epic/Story 必须从 `_bmad-output/planning-artifacts` 的既有产品简报、补充材料和架构材料拆解。每份 Story 产品需求须写明来源章节与验收映射；区分原始需求、后续用户变更、架构约束和待审设计建议，不得从已有代码反推产品范围或自行删减原需求。
- 每个 Story 必须含 `README.md`、`product-requirements.md`、`architecture-design.md`、`visual-interaction-design.md`；无 UI 也须定义调用方交互、可观察状态与错误反馈。设计缺失、验收不清或前置门禁未满足时不得实施。
- Story 设计明确后才能生成关联它的 OpenSpec proposal/design/tasks/delta spec；不得先写代码再补设计，不得以 OpenSpec 代替 Story 文档。详见 AD-DEV-01-MODULE-EPICS.md。
- 技术路线未验证时先建有期限的 Spike；Spike 必须有统一样本、淘汰门槛、双平台证据和 ADR。ADR 定案前不得开始依赖该技术路线的产品 Story。
- 首批必做验证：Tauri/IPC 跨平台基础栈、Qwen 与 trycua CUA Driver、ego-lite 集成、Rust 与 Node OOXML 实现、原生上下文采集、SQLCipher/FTS5/附件加密。
- 改变系统边界、依赖方向、状态所有者、协议、持久化或技术栈时，必须先更新 Architecture Decision，再修改 OpenSpec 和代码。
- 每个 Story 默认一个短分支、一个 OpenSpec Change、一个 PR；主干始终可构建。

## 系统边界

- Yonder 是 Windows/macOS 常驻桌面 Agent Tool，不是通用 Agent、负责首次计划/语义 replan 的慢脑规划器、云端 Agent 平台或云端个人上下文服务。按 Accepted AD-EX-01，Yonder 可内置有界 Jev 快脑模型决策循环；具体技术接入仍受 AD-EX-02 Spike 门禁约束。
- Yonder 只实现云端客户端 Connector；不得在本机开放公网服务、端口映射或 P2P。
- BUA 直接复用 ego-lite Browser Task Space；不得在 Yonder 内复制 Task Space。
- CUA 仍只实现模型无关执行 Driver；Jev 快脑位于 Application 执行协调层、四类执行 Driver 之上，不得把模型循环塞入 Driver。

## 代码与依赖

- 架构采用模块化单体、六边形 Port/Adapter 和受监管的按需 Worker。
- 目标 Workspace：`apps/desktop`、`apps/yonder-cli`、`crates/domain`、`crates/application`、`crates/adapters`、`crates/protocol`。
- 依赖只能是：desktop→adapters/application，adapters→application/protocol，application→domain/protocol，CLI→protocol。
- Domain 不得依赖技术库；Application 不得引用具体 Adapter；Adapter 不得互调；React UI 不得成为核心状态所有者。
- Rust 协议类型是 JSON Schema 和 TypeScript 类型的唯一来源。不得手写第二份协议模型。
- 优先标准库、系统原生能力和已有依赖；不为单一实现创建抽象，不为未来需求搭脚手架。

## Gateway 与任务语义

- 本地 Agent 通过 MCP stdio/CLI 和 Local Socket 接入；macOS 使用 Unix Domain Socket，Windows 使用 Named Pipe，不使用本地 HTTP/TCP。
- 云端通过 Yonder 主动建立的单一 WSS 连接接入。本地与云端请求必须进入同一 Agent Gateway 和 Application 用例。
- 外部慢脑的首次计划和 replan 均从既有 Agent Gateway 进入；快脑交回依据通过同一任务事件/Outbox 和 Gateway 对归属 Agent 可见，不得直连慢脑或建立第二条 Agent 通道。
- 快脑连续步骤使用绑定已验证计划的可信内部来源，不得伪造 Agent 会话；任务完成/失败仍由归属 Agent 经 Gateway 提交。
- 所有调用携带 `request_id`、`agent_id`、`capability`、`deadline`；创建任务支持 `idempotency_key`。
- 长任务统一返回 `task_id`，并支持 `task.get`、`task.cancel`、`task.events(after_sequence)`。
- SQLite 当前状态表是当前状态唯一事实源；状态更新、追加事件和 Outbox 写入必须同事务。每任务 `sequence` 严格递增。
- 副作用操作超时、崩溃或断连时标记 `unknown`，不得自动重试。重启后的运行中任务转为 `interrupted`，重新 Observe 后由外部 Agent 决策。
- 每一步 CUA/BUA 执行后必须 Observe；Yonder 可在外部计划片段内以 Jev 选择下一受支持动作并验证预期条件，片段外的语义 replan 仍由经 Gateway 接入的外部 Agent 负责。
- CUA 同一时间只允许一个前台桌面租约；用户输入立即暂停。BUA MVP 单并发；同一文件禁止并发写。

## 执行与数据安全

- Command 默认使用结构化 `program + args + cwd + env`；Shell 必须显式请求。提权、安装、删除、支付和发送必须用户确认。
- File Tools 必须规范化绝对路径并阻止路径穿越；删除默认进入系统回收站。
- Document Port 不暴露 XML。OOXML 修改默认另存，覆盖须显式请求；使用 `expected_hash`、临时文件、结构校验和原子替换。不得绕过 Office/WPS 文件锁。
- Recording 默认关闭且仅由用户手动开始；原始时间线不可变。只把 `user` 输入派生为轨迹，不得重新录制 `agent_cua` 或 `replay` 输入。
- 密码框、隐私窗口、系统安全界面和用户排除应用始终不采集。禁止全盘扫描和读取浏览器内部 History 数据库。
- 2026-09-14用户变更（AD-ST-01）：MVP暂不加密，SQLite/FTS及附件加密、系统Credential Store密钥接线延期至MVP之后；不得自动修改或覆盖已有加密库。状态/事件/Outbox仍同事务。日志不得记录正文、截图、输入、完整命令输出或完整 Agent Payload。
- 采集、索引、同步使用有界队列和本地配额；不得写满磁盘，不得自动删除未同步数据、轨迹或用户固定内容。

## 验证与完成

- 最小测试层级：Domain 单元、协议/Adapter 合约、SQLite/Outbox/Sidecar/Gateway 集成、Windows/macOS 原生 E2E。
- UI、Driver、系统权限和原生采集变更必须提供 Windows/macOS 各自的截图、视频或结构化日志证据。
- 每个 Story 实施后必须创建独立 Verification Goal；验证失败返回实施阶段，Goal 通过后才能 Archive 和 Done。
- CI 必须检查依赖方向、协议单一来源、Story/OpenSpec/PR 关联和要求的验证证据。

## MVP 禁区

不得引入完整 Event Sourcing、通用本地 HTTP Server、本地向量库/Embedding/知识图谱、Office COM/AppleScript/WPS 私有插件、跨设备密钥同步、多个 CUA 同时控制桌面或长期维护双执行栈。
