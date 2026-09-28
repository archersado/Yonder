# EN-S3 架构设计

## 边界与依赖

本 Story 是纯重构：只改变代码组织与并发结构，不改变系统边界、依赖方向、状态所有者、协议语义与持久化。依赖图保持 `desktop→adapters/application、adapters→application/protocol、application→domain/protocol、CLI→protocol`；CI `check_dependencies` 白名单不变。

按变更分级第 3 条，本 Story 触及"协议（单一来源实现位置）与持久化（仅文件组织）"，但其外部可观察行为经 AC1/AC2/AC8 等价性证明约束为不变，故以本 Story 的三份设计与等价性 golden 测试替代新 ADR；若实施中任何等价性前提被打破，立即停止并先补 ADR。

## 状态与契约

### 阶段 1：协议单一来源（AC1，独立可合）

- `valid_id`：保留 `crates/protocol/src/lib.rs:1292` 实现作为唯一来源；`crates/application/src/lib.rs` 的 `validate_id/valid_id` 改为引用 protocol（application 已依赖 protocol，方向合法）。application 公共导出保留 `pub use yonder_protocol::valid_id` 以免调用点扩散。
- `UnknownReason`：application `computer_use::UnknownReason` 与 protocol `AttemptUnknownReason` 合并——application 侧保留内部枚举，转换收敛为单一 `impl From<internal> for wire` 实现（置于 protocol 或 application 其一），`gateway.rs` 尾部与 `attempt_result` 内的散落 match 删除。语义映射表（含 8 个 reason 值）由既有协议回归逐值断言保护。
- 版本常量：`PROTOCOL_VERSION`（当前 1.31）从 `application/gateway.rs` 移入 `yonder-protocol`，application `pub use` 再导出；`apps/yonder-cli/src/main.rs` 握手参数改为引用该常量，删除 `minor: 31` 字面量。CLI 仅依赖 protocol，此下沉方向合法。

### 阶段 2：能力协商等价重构（AC2/AC3，独立可合）

- 新增声明式能力表：每项为 `{ Capability, 最低 minor, 可用性闭包（negotiated/store supports_/runtime available/platform 的组合） }`，位于 `gateway.rs`。`GatewaySession::can_*` 布尔位改为从表按 hello 参数一次性计算；两条版本链、`CapabilityInfo` 列表生成、逐请求门禁（`-32010` 检查）全部由表派生。
- 等价性 golden 测试：以重构前 hello 响应为基准，对 minor 0–31 逐档断言 `{ protocol_version, capabilities 列表（名称/版本/availability/reason） }` 完全一致；同时断言每档下各请求类型的接受/拒绝（错误码与消息）。基准在重构前一次提交固化。
- 请求分发：`handle` 与 `handle_encoded_with_runtimes_and_file_grants` 中的 `if let/matches!` 早返回串改为对 `Request` 全变体的穷举 `match`（按"握手前/握手/查询/执行"四组路由）；协议校验收敛到分发入口单次调用，删除 `gateway.rs:1120`、`:728` 与 `query.rs` 入口的重复 `validate`。拒绝语义不变由既有回归断言。
- 顺手消除 `can_audit` 双赋值与两条版本链的不一致（以 golden 基准为准——基准即重构前行为，链不一致处按重构前实际行为对齐并单独记录差异点，不静默"修正"语义）。

### 阶段 3：TaskStore 端口拆分（AC4，随下次触碰 task_store 的变更一并落地，不阻塞阶段 1/2）

- 按聚合拆子 trait：`TaskCoreStore`（register/get/list/events/commit/presentation）、`TaskExecutionStore`（attempt/control/focus/wait_for_user/step）、`TaskAuditStore`（audit/manifest/confirmation）、`TaskPlanFragmentStore`（plan fragment 四方法）；`pub trait TaskStore: TaskCoreStore + TaskExecutionStore + TaskAuditStore + TaskPlanFragmentStore` 超trait 保持所有现有调用点签名不变（零调用点改动）。
- SQLite 实现按同边界拆文件：`task_store/` 目录下 `core.rs / execution.rs / audit.rs / plan_fragment.rs / schema.rs（initialize 与迁移）/ mapping.rs（name/status/reason 等转换）`；42 个内联测试随所属模块迁移，断言不修改。
- `supports_*` 收敛：握手期一次调用生成 `StoreCapabilities` 结构体，`GatewaySession` 持结构体而非 11 次散布查询；trait 默认方法模式废除，未实现的能力在结构体中显式缺席。这使"实现遗漏"从握手期静默降级变为能力表显式行。
- 拆分全程不改 SQL、schema、事务边界与 Outbox 写入；以既有 42 项 adapter 回归 + `PRAGMA user_version` 断言为护栏。

### 阶段 4：宿主并发模型（AC5）

候选对比后定稿，首选"专用宿主线程 + 有界通道"：

- `TaskHost` 迁入专用 OS 线程，Socket/Tauri 命令路径通过 `mpsc` 有界通道投递请求并等待回包（`tokio::sync::oneshot` 回执）；宿主线程内保持现有顺序处理语义（AD-AG-09 连续推进天然兼容）。Tauri 主线程窗口投递仍走 `run_on_main_thread`，与宿主线程解耦。
- Socket 路径删除跨执行持锁：`local_agent_socket.rs` 的 `host.lock()` 临界区收缩为"发送请求 + await 回执"；`CuaControlHub` 既有进程内旁路保留。
- 回退候选（若通道化改动面过大）：保持 Mutex 但临界区内禁止执行动作（PlanExecute 改为槽位间释放/重取 + 进程内续约），须证明不破坏 AD-AG-09"单次调用内同步有界连续推进"的响应字节不变性——两种候选均以"响应字节与 Outbox 事实不变"为等价底线。
- 并发回归：宿主线程处理一个含多个槽位的 PlanExecute 期间，另一连接并发 `task.query`/`task.list` 在既定 deadline（协议 10s）内获得响应；该测试为确定性断言（通道化后天然满足；回退候选须以真实调度证明）。同步 CLI 路径（stdio）行为不变。

### 阶段 5：门禁与工程整洁（AC6/AC7，独立可合）

- `check_architecture.py` 增加 `check_frontend_single_source`：扫描 `apps/desktop/ui/*.js`，凡出现状态字面量来源（如独立定义 status/purpose/availability 枚举值集合且用于协议字段映射）须能追溯到 `crates/protocol/generated/protocol.ts` 的引用（import/注释声明 + 路径存在性校验）；`scripts/test_check_architecture.py` 补接受/拒绝样本。
- `apps/desktop` 根目录 `check-*` 脚本（约 60 个）迁至 `scripts/verification/`（或既有 tests 目录惯例），全仓 grep 引用点（README、story 验证记录、脚本内相对路径）批量更新；迁移不改任何脚本逻辑。

## 失败与验证

- 每阶段独立提交、独立可回滚；任一阶段护栏（workspace 测试、协议 `--check`、依赖方向、`user_version` 断言）失败即回退该阶段，不带病合并。
- 等价性以 golden 基准为准：能力协商、错误码、wire 字节三者在重构前后逐字节/逐值一致；任何有意的行为差异（如 `can_audit` 双赋值消除若影响某档响应）必须在 PR 中逐项列出并说明为何不属于"外部可观察行为变化"。
- 宿主并发回归在 macOS 原生环境运行并留存结构化日志；按围栏要求提供 macOS 证据，Windows 暂缓记录在案。
- 验证顺序：阶段 1/2/5 以自动化测试与门禁自测为验收主体；阶段 3 以回归全量通过为验收主体；阶段 4 须额外 macOS 原生证据（并发回归日志 + UI 非阻塞结构化证明）。

## 架构影响

- 对外协议、持久化、依赖方向：none（等价性证明约束）。
- 对内结构：protocol 单一来源强化（AC1）、Gateway 声明式能力表（AC2/AC3）、TaskStore 子 trait 化（AC4）、宿主线程模型（AC5）——均为 conforming 演进，不产生第二状态所有者，不引入 Adapter 互调。
- 与 AD-AG-09 的关系：阶段 4 只改执行所在的线程与同步机制，不改变"单次 Gateway 调用内同步有界连续推进、异常立即交回"的契约；响应字节不变为硬约束。
