# EN-S3 产品需求

## 需求来源

- 后续用户变更（2026-09-28）：用户在架构审查后明确授权"创建重构 Story 修正"以下结构性问题——
  1. `TaskStore` 单一端口膨胀为约 74 个方法、8 类职责混杂（"上帝端口"），唯一 SQLite 实现单文件超 8000 行；
  2. 协议 minor 版本六周内递增至 1.31，能力协商由 30+ 个 `can_*` 布尔位与两条手写嵌套版本链承载，且存在重复赋值与链不一致；
  3. 桌面组合根 `TaskHost` 单把互斥锁串行全部职责，本地 Socket 在 Tokio worker 上长临界区执行 CUA 连续动作，阻塞 UI 查询与展示路径；
  4. 协议单一来源被侵蚀：`valid_id` 双实现、`UnknownReason` 双枚举加手工映射、CLI 硬编码 minor 版本；
  5. Gateway 请求分发为早返回 `if let` 串而非穷举 `match`，新增协议变体无编译期覆盖保证；同一请求存在重复校验；
  6. CI 缺少围栏承诺的"前端 import 检查"；`apps/desktop` 根目录被 60+ 个验证脚本占用。
  该清单是用户明确变更，不是从产品简报推导的新产品功能，也不是从代码反推的产品范围。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)「模块与依赖」「Agent Gateway」「任务、状态与恢复」「代码与依赖」章节；[研发与需求变更模式](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/DEVELOPMENT-AND-CHANGE-MODE.md) 变更分级第 3 条。
- 验收映射：下述 AC1–AC8 与问题 1–6 一一对应；每个 AC 可由自动化测试、门禁脚本或结构化日志复现，不以主观评价为通过依据。

## 问题与目标

上述债务使每次协议/能力演进需要人肉同步约 7 处位置，能力静默降级与复制粘贴缺陷已实际发生（`can_audit` 双赋值、AD-TM-21 记录的 schema 遗漏事故同类风险）；宿主锁模型在 AD-AG-09 修订为单次调用内连续推进后，已成为 UI 响应性与 Windows 双平台化的硬阻塞。

目标：在不改变任何外部可观察行为（wire 协议、持久化 schema、依赖方向、能力协商结果）的前提下，完成核心结构重构，使后续协议演进与多平台扩展的边际成本回落到"改一张表 + 一个实现文件"。

## 范围与非目标

范围：`crates/protocol`、`crates/application`（gateway/query/lib）、`crates/adapters`（task_store 文件拆分）、`apps/desktop`（宿主并发模型）、`scripts/check_architecture.py`（前端门禁）、验证脚本目录迁移。

非目标：

- 不新增、不修改、不废弃任何 wire 协议消息、字段或能力语义；`PROTOCOL_VERSION` 数值不变。
- 不修改 SQLite schema、`user_version`、迁移路径或事件/Outbox 同事务语义。
- 不改变 Cargo 依赖方向与模块边界；不引入新外部依赖。
- 不实现 Outbox 消费/保留期清理（属云端 Connector 与存储后续 Story）；不清理 OpenSpec 归档积压（流程事项，另行处理）。
- 不实施 AD-EX-04 Recipe、不改动 Jev 决策语义、不触碰加密（AD-ST-01 延期不变）。
- 不做 Windows 原生验证（按 2026-09-13 用户决定暂缓；本 Story 不据此标 Done）。
- 不修正 Architecture Spine 的"React 桌宠"表述漂移（文档修正随本 Story PR 顺带提交，不属验收范围）。

## 验收条件

- AC1（协议单一来源）：`valid_id` 只保留 protocol crate 一处实现；`UnknownReason` 到 wire 枚举的映射收敛为单一 `From` 实现，gateway 内手写 match 删除；当前协议版本常量下沉至 `yonder-protocol` 并由 application 再导出、CLI 引用，`apps/` 下不再存在 minor 版本字面量；协议生成 `--check` 与 workspace 测试通过。
- AC2（能力协商等价）：Gateway 能力协商改为单张声明式能力表（capability → 最低 minor → 可用性条件）派生 `can_*` 位、两条版本链、capability 列表与逐请求门禁；新增 golden 测试逐档断言 minor 0–31 的 hello 响应与重构前一致（含 reason 文案与能力列表顺序）；`can_audit` 重复赋值消除。
- AC3（请求分发穷举）：Gateway 对 `Request` 全变体的会话门禁与分发使用穷举 `match`，新增变体未处理时编译失败；同一请求的协议校验收敛到单一入口，重复 `validate` 消除且拒绝语义（错误码、消息）不变，由既有协议回归断言保护。
- AC4（端口拆分）：`TaskStore` 按聚合拆为子 trait（核心读取/生命周期、执行 attempt/控制、审计、计划片段），以组合超 trait 保持 gateway 现有 `impl TaskStore` 签名与借用模型不变；SQLite 实现按同边界拆分为多文件；既有 adapter 回归的断言不得修改且全量通过；`supports_*` 收敛为握手期一次能力探测，调用点不再散布 11 处布尔查询。
- AC5（宿主并发模型）：TaskHost 的 Gateway 处理迁出 Tokio worker 长临界区（专用宿主线程 + 有界通道或等效方案），Socket/CLI 路径不再跨执行持锁；并发回归证明 PlanExecute 连续推进期间 `task_query` 与桌宠展示路径不被无限期阻塞；`CuaControlHub` 既有旁路语义与主线程窗口投递（`run_on_main_thread`）保持。
- AC6（前端门禁）：`check_architecture.py` 增加前端单一来源检查——UI 页面不得定义第二套协议/状态字面量来源，必须引用 `crates/protocol/generated/protocol.ts` 的生成产物路径；门禁自测含接受与拒绝样本。
- AC7（工程整洁）：`apps/desktop` 根目录 `check-*` 验证脚本迁至仓库统一脚本/测试目录，所有引用（文档、脚本、CI）同步更新，迁移后既有验证路径仍可执行并通过。
- AC8（护栏）：实施全程 `user_version`、wire 协议、Cargo 依赖方向不变；`cargo test --workspace --locked` 与 `check_architecture.py` 在每个阶段提交点均通过；若实施中发现任何 AC 与护栏冲突，立即停止并按变更分级第 3 条先走 ADR，不得绕过。

## 待决事项

- 宿主并发模型的具体形态（专用线程 actor 与"缩短临界区 + 读快照分离"两种候选）在架构设计中对比定稿，属工程审阅范畴，不构成未决产品问题。
- AC5 的"不被无限期阻塞"以并发回归的确定性断言为准；若 macOS 平台限制导致部分路径无法消除争用，须在该回归中显式记录剩余争用点与理由，不得静默豁免。
