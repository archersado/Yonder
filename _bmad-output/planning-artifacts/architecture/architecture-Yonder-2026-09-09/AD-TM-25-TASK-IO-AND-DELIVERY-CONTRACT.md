# AD-TM-25 任务输入输出与交付契约

状态：Accepted（2026-10-16，用户明确要求解决交付物为空并补齐任务分配输入输出契约）。  
Architecture Impact：architecture-change（协议、运行时事件、持久化、能力 Adapter 产物登记）。关联 AG-S2、TM-S5、TM-S9、TM-S10、DS-S2、FI-S1、DO-S1。

## 问题

现有 `task.create` 只有名称和内部任务说明，未声明可展示的输入摘要与预期输出；`task.complete` 只有核验 ID，不能提交有界交付摘要。TM-S5 已有不可变产物清单核心，但发布入口仅供可信能力用例，File/Document 成功回执尚未登记，因此 Task Space 常见“交付物为空”。直接展示完整 Agent 输出会扩大敏感内容和传输量，也不能证明任务完成。

## 决定

1. 协议 1.52 增加任务 I/O 契约：创建时由归属 Agent 提交 `input_summary` 与 `expected_output`，均为纯文本有界摘要；任务说明继续只作内部执行输入，不进入 Task Space。
2. `task.complete` 增加 `result_summary`。它是“Agent 交付摘要”，只有在 AD-TM-24 的最新 achieved 目标核验通过后才能随完成提交；不得替代 Observe、目标核验或用户确认。失败提交不伪造交付摘要。
3. 活动任务的交付摘要随 `ExecutionRuntime` 终态事件进入异步投影；非 Runtime 兼容路径在终态、事件、Outbox 与结果摘要同一 SQLite 事务提交。投影落后不得阻塞 Driver 或完成响应。
4. File `create-new/replace` 与 Document `save-as` 在动作已成功且 Observe 有效后，由 Application 生成任务绑定的 opaque `reference_id`，通过 Runtime 产物事件或兼容事务发布下一版不可变清单。Agent、UI 和 Driver 不得自报路径或伪造引用。Read、Trash、Command、CUA、BUA 默认不产生文件交付物；它们用交付摘要表达结果。
5. Task Space 展示“任务输入摘要 / 预期输出 / Agent 交付摘要 / 产物”，缺失契约明确标识为旧任务未声明。摘要只用 `textContent`，不显示完整 Agent Payload、任务正文、截图、输入、完整命令输出或路径。

## 限额与兼容

- `input_summary`、`expected_output` 各 1..512 UTF-8 字节；`result_summary` 1..1024 UTF-8 字节；允许换行，拒绝其他控制字符。
- 协议 1.51 及以下保持旧请求和响应形状；协商 1.52 的新任务必须提供两项创建契约，完成任务必须提供交付摘要。
- SQLite schema 27 新增 `task_io_contracts` 与 `task_results`；旧任务不回填、不从 description 或历史事件推断。
- 同一幂等键的契约不同视为幂等冲突。结果摘要按终态序号唯一，不允许终态后改写。
- 产物引用仍遵循 TM-S5 每版 4096 条上限；本轮不提供路径解析或“打开产物”，不复制用户文件。

## 验收

- 创建、查询、重试和冲突覆盖完整 I/O 契约；旧协议看不到新字段。
- 最新 achieved 核验后完成，摘要与终态按 Runtime/兼容路径正确投影；缺摘要、旧序号或未核验拒绝。
- File/Document 成功产生非空可信产物清单；失败、unknown、read/trash 不产生假产物。
- Task Space 不等待产物页即可显示三项摘要；大正文/完整输出不进入 DOM。
- 协议生成、SQLite 迁移/回滚、Application/Gateway/Adapter 集成及 macOS 独立 Verification Goal 通过；Windows 继续按既有安排暂缓。
