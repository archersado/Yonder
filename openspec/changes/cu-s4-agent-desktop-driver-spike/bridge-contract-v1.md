# agent-desktop-spike-bridge/v1

本文件冻结 Spike bridge 的语义契约；实现时 Rust 类型是消息模型唯一来源，并生成 JSON Schema。首次原生运行前，build manifest 必须记录本文件、生成 schema 与 Worker 制品的 SHA-256。不得另写 TypeScript 或平台专用消息模型。

## 传输与顺序

- 单个受监管 Worker，stdin/stdout UTF-8 JSONL，一行一个完整对象；每帧含结尾换行最多 65,536 bytes。stderr 只能输出无正文的安全分类。
- 同一 Worker 同时最多一个未完成请求；每个请求都必须回显 `request_id`、`attempt_id`、`worker_instance_id` 和 `fence`。
- Host 先完成 `hello`，再按 `snapshot → dispatch → dispatch_result → observe → observe_result` 的唯一顺序执行一步；步骤开始前和完整 Observe 后才允许 `shutdown`。在途动作没有 cancel 消息；需要停止时由 Supervisor 终止 Worker并按 `unknown` 处理。重复、跳步或乱序消息使 Worker 进入 terminal 并退出。
- Host 或 Worker 必须增量读取到换行后才解析。遇到 invalid UTF-8、malformed JSON、未知字段、未知 kind、超过 65,536 bytes 的帧、EOF 前的 partial frame、schema/version/hash/身份不匹配、重复或乱序响应时，Host 或 Worker 必须关闭 Worker。若动作帧尚未 flush，结果为 `not_dispatched`；若已经 flush，结果为 `unknown` 并执行 containment/cleanup，不得自动重试。

## Host → Worker

所有请求共有：

- `schema_version = "agent-desktop-spike-bridge/v1"`
- `kind = "hello" | "snapshot" | "dispatch" | "observe" | "shutdown"`
- `request_id`、`attempt_id`、`worker_instance_id`：非空、有界字符串
- `fence`、`input_generation`：非负 `u64`
- `deadline_mono_ms`：宿主单调时钟 deadline

各 kind 额外字段：

- `hello`：`source_revision` 固定为 `d3dc190afbf34cd2377252398ddd5c30efbd093a`，并携带 `bridge_contract_sha256`、`sample_definitions_sha256`、`action_allowlist_sha256`。
- `snapshot`：携带 `sample_id`、冻结 `target_fixture_id` 与 `target_identity`。identity 只含平台、Host 只读 oracle 给出的进程 ID、原生窗口 ID、进程启动令牌 SHA-256、原生窗口令牌 SHA-256、显示拓扑 generation 和冻结 `oracle_id`；不得按标题或前台窗口隐式选择。窗口令牌必须区分同一进程内窗口对象代际，单独哈希 HWND 与进程启动令牌不合格。Worker 回应唯一 `snapshot_id`；每个固定 Spike 样本必须解析出恰好一个 opaque `qualified_ref`，零个或多个都按 ambiguous fail-closed，禁止默认选择首项。
- `dispatch`：`sample_id`、`action_id`、`snapshot_id`、`qualified_ref`。`sample_id ↔ action_id` 必须与 `sample-catalog.md` 和 `action-allowlist-v1.json` 的唯一绑定一致；不一致在 flush 前拒绝。`action_id` 不承载自由文本、raw 坐标、PID、窗口号或任意参数。
- `observe`：`sample_id`、`predicate_id`、`oracle_id`、`prior_snapshot_id` 与完全相同的 `target_identity`。它必须绑定当前 pending dispatch 的 sample/snapshot/action/ref；predicate/oracle 只引用冻结定义，Worker 不决定最终通过结论。
- `shutdown`：只允许在没有在途 `dispatch` 时使用，表示正常步骤间排空；不承担在途取消。

## Worker → Host

所有响应共有：

- `schema_version`、`kind`、`request_id`、`attempt_id`、`worker_instance_id`、`fence`
- `status = "ok" | "rejected" | "unknown"`
- `error_class = "none" | "schema" | "identity" | "deadline" | "permission" | "stale" | "ambiguous" | "unsupported" | "driver" | "observe"`

各 kind 额外字段：

- `hello_result`：固定 `source_revision`、`bridge_contract_sha256`、`sample_definitions_sha256`、`action_allowlist_sha256`、`capabilities_sha256`；任一不匹配时 Host 禁止派发。
- `snapshot_result`：`snapshot_id`、有界的 `qualified_refs`、`target_identity_verdict`；`status = "ok"` 时目标身份必须匹配、`snapshot_id` 非空且恰好一个 opaque ref。Backend 明确拒绝时，Worker 回传错误分类、空 `snapshot_id` 和空 `qualified_refs`，随后终止会话；Host 不得进入派发。不得返回窗口树或正文。
- `dispatch_result`：`delivery_disposition = "not_delivered" | "delivered_verified" | "delivered_unverified" | "unknown"`、`driver_status` 和可选 `next_snapshot_hint`；不得返回输入正文、通知、剪贴板、截图或完整窗口树。
- `observe_result`：`snapshot_id`、`target_identity_verdict = "match" | "mismatch" | "ambiguous" | "stale" | "unavailable"`、`predicate_verdict = "satisfied" | "unsatisfied" | "unstable" | "unavailable"`、`element_count`。Host 仍须用独立只读 oracle 复核。
- `shutdown_result`：只表示正常排空与 Worker 可退出；不得用作在途动作已取消或副作用已回滚的证据。

## Rust 单一来源门禁

实现必须在 Spike bridge crate 中用 Rust `enum/struct` 定义以上封闭字段、65,536 bytes 帧上限和错误分类，并生成 `bridge-schema-v1.json`。CI/验证比较生成 schema 的 SHA-256 与 build manifest；本文件只定义验收语义，不作为第二份可执行模型。
