---
title: 'CU-S4 agent-desktop 隔离 Driver Spike Harness'
type: 'feature'
ticket: ''
created: '2026-11-05'
status: 'built'
baseline_revision: '3643a0e5f77c584c81eb4849aaf622b7fdd94cce'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran:
  - 'quick'
review_loop_iteration: 5
context:
  - '_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-07-AGENT-DESKTOP-DRIVER-SPIKE.md'
  - 'docs/specs/epic-CU/story-CU-S4/architecture-design.md'
  - 'openspec/changes/cu-s4-agent-desktop-driver-spike/design.md'
  - 'openspec/changes/cu-s4-agent-desktop-driver-spike/sample-catalog.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Yonder 已确定把 `agent-desktop` 仅作为低层 CUA Driver 候选，但仓库还没有隔离 Worker、唯一 JSONL 契约、派发线性化、证据校验和平台 containment 的可执行 Spike。直接接产品会造成第二控制面、无法安全终止在途调用，并可能把不确定副作用误报为取消成功。

**Approach:** 在 `spikes/agent-desktop-driver/` 建立独立 Rust workspace，固定上游 commit `d3dc190afbf34cd2377252398ddd5c30efbd093a`。先实现 Rust 单一来源协议、64 KiB 有界 framing、Supervisor/admission gate、mock backend 与跨平台 containment 接口，再接固定上游 core 的 Windows/macOS Worker；所有原生运行仍受 OpenSpec 双平台证据门禁。

## Boundaries & Constraints

**Always:** 只使用 inherited stdio JSONL；完整帧 flush 是唯一派发线性化点；flush 后的终止结果为 `unknown` 且零重试；输入分类、身份、fence、attempt/Worker 必须一致；协议与证据 schema 从 Rust 类型生成；固定 action/predicate/oracle；Worker 残留进入全局 quarantine；日志和证据不含输入正文、截图、完整窗口树或 Payload。

**Never:** 不修改 `apps/desktop`、`crates/application`、`crates/adapters`、`crates/protocol`、Gateway、SQLite、Task/Outbox；不接真实产品任务；不开放 HTTP/TCP、MCP、CDP、raw 坐标、无目标按键或自由参数；不以单平台结果替换 trycua；不发送合成 key-up/button-up 冒充清理。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 握手 | 固定 revision、contract/sample/action hashes | Worker 回显相同身份和 capability hash | 任一不匹配时禁止派发 |
| 正常动作 | 当前 fence/generation、唯一 action/ref | flush 后执行；Host 强制 Observe；证据满足 sample profile | Observe 不通过即 `unknown`，零重试 |
| 派发前取消 | admission gate 内 cancel 先于 flush | `not_dispatched`，Worker 未收到动作 | 保持可继续只读观察 |
| 派发后终止 | 帧已 flush，Worker 被 kill/断连/超时 | `unknown`，containment 和只读 cleanup | 残留或无法证明时 quarantine |
| 非法帧 | >64 KiB、partial EOF、invalid UTF-8/JSON、乱序 | 关闭 Worker，不分配无界内存 | flush 前 `not_dispatched`；flush 后 `unknown` |
| Observer 异常 | 未就绪、丢失、重启、generation 回退 | fail-closed，禁止新派发 | 仅只读恢复或宿主重启解除 |

</frozen-after-approval>

## Code Map

- `spikes/agent-desktop-driver/` -- 新增隔离 workspace；不得加入产品 Cargo workspace。
- `crates/adapters/src/cua.rs` -- 只读参考现有 Worker 身份、超时、断连与 Drop 清理模式；不修改。
- `apps/desktop/src/local_agent_stdio.rs`、`spikes/desktop-foundation/src/lib.rs` -- 只读参考解析前 64 KiB 限长。
- `spikes/command-executor/windows-probe.rs` -- 复用 Windows Job Object `KILL_ON_JOB_CLOSE` 探针模式。
- `openspec/changes/cu-s4-agent-desktop-driver-spike/` -- bridge/action/sample/evidence 的权威验收输入；生成器必须 `--check`。
- 上游 `agent-desktop` `crates/core/src/commands/{snapshot,execute_by_ref}.rs`、`response/output.rs` -- 固定 core API、qualified ref 与 disposition 来源。

## Tasks & Acceptance

**Execution:**
- [x] `spikes/agent-desktop-driver/Cargo.toml`、`rust-toolchain.toml`、`Cargo.lock` -- 建立 Rust 1.89/edition 2024 独立 workspace，按 target 固定上游 core/windows/macos rev 与静态 CRT 策略。
- [x] `src/protocol.rs`、`src/bin/generate-schema.rs`、`schemas/` -- 定义 hello/snapshot/dispatch/observe/shutdown、evidence 与封闭枚举，从 Rust 生成并同时校验 Spike/OpenSpec 唯一 schema；握手冻结会话，dispatch 保存 sample/snapshot/action/ref，Observe 必须与该 pending dispatch 完整绑定，且证据 schema 包含稳定观察与分发责任链所需字段。
- [x] `src/framing.rs`、`src/supervisor.rs` -- 用宿主共享 admission gate 在同一临界区线性化 control/fence/generation 最终比较与有 deadline 的 flush；实现 `ready→snapshot→dispatched→response→observed/terminal` 顺序机、在途超时 kill/unknown、零重试和宿主级 quarantine；无效/重复/乱序输入进入 terminal 并要求关闭 Worker。
- [ ] `src/containment/` -- Windows 必须 suspended spawn、加入 Job 后再 resume，失败时终止并 wait；macOS 受监管进程组必须以完整进程树只读探针证明无逃逸 helper；实现无副作用 cleanup verdict，其他平台只允许 mock。
- [ ] `src/bin/worker.rs`、`src/backend/` -- 先在 AD/bridge contract 增加首次 snapshot/result、qualified refs 与冻结 target identity 输入，再接固定上游 core 的 snapshot/ref action/Observe；显式关闭 trace/screenshots，使用私有绝对 `AGENT_DESKTOP_HOME`，任何未验证路径 fail-closed。
- [x] `src/bin/harness.rs`、`tests/`、`build-manifest.json` -- 可执行组合覆盖原子抢占竞态、顺序错误、跨消息绑定、阻塞 backend/管道读取的 deadline kill、独立只读 oracle 的两次稳定观察、cleanup/quarantine；cleanup 未完成完整只读探针前不得返回 Released；evidence verifier 绑定 boundary/generation/input/稳定次数及分发责任链，manifest 校验实际 tree/dirty、计划与已验证 target、CRT、build/verification commands 和 artifact provenance（当前仅验证 Windows x86_64 契约构建，无可分发原生制品，明确为 `contract_only_native_backend_unsupported`）。
- [x] `README.md`、OpenSpec `tasks.md` -- 记录运行命令、隐私边界、当前已验证/未验证平台，不把契约测试称作双平台原生通过。

**Acceptance Criteria:**
- Given 独立 workspace，when 构建和测试，then 不修改或链接任何产品运行时路径，固定上游 revision 可从 lockfile/build manifest 追溯。
- Given 任一请求/响应，when schema、hash、身份、fence 或 generation 不匹配，then 动作不会被接受为已知成功。
- Given 64 KiB±1、partial EOF、invalid UTF-8/JSON，when framing 解析，then 在分配无界缓冲前拒绝，并按 flush 边界输出 `not_dispatched` 或 `unknown`。
- Given flush 前后取消竞态，when Supervisor 裁决，then 每次只有一个线性化结果，且 flush 后永不报告确定取消或自动重试。
- Given sample evidence，when 运行生成 schema 的 verifier，then 合法 fixture 通过，Observe失败/unknown错误通过/action错配 fixture 被拒绝。
- Given Worker cleanup 无法证明，when 尝试下一次派发，then global quarantine 阻止动作，且无合成补偿输入。

## Implementation Notes

- 2026-11-05：固定 revision 的 `agent-desktop-core` 与 `agent-desktop-windows` 已从 Cargo git dependency 编译，公开 `PlatformAdapter` 接口可从 bridge crate 引用，不存在必须改用 CLI/C ABI 的 API 可见性阻断。
- AD/bridge contract 已补首次 snapshot/result、qualified refs 与冻结 target identity。固定上游 `execute_by_ref` 的 pointer hit-test `Unknown` 仍会 fail-open，且公开结果/API 不暴露可供 wrapper 在副作用前拒绝的 verdict/hook；因此无法满足 AD-CU-07 的 fail-closed 条件。Native backend 保持 `unsupported/not_delivered`，不得用第二通道、隐式定位、CLI 或 C ABI 绕过。
- Windows Job Object 与 macOS process group 已实现进程 containment 接口；interaction lease、修饰键/按钮、临时资源及新实例 Observe 的完整只读 cleanup probes 尚需双平台原生环境，任务保持未勾选。
- Quick Review re-derivation 后，Worker 在 hello 时冻结 attempt/instance/fence/generation，Supervisor 使用共享宿主 quarantine、OS 单调时钟 deadline，并把 write/flush 失败保守裁决为 `unknown`。只有固定 schema、正确 kind/status/error/disposition 的 dispatch response 加完整 Observe 才可进入 `Observed`。
- Windows 现在以 `CREATE_SUSPENDED` 启动，加入 `KILL_ON_JOB_CLOSE` Job 后才恢复主线程；assign/resume 失败均终止并 wait。macOS helper 是否可能脱离 process group 保留为明确 deferred，不据此宣称双平台 containment 完成。
- Evidence/OpenSpec schema 由同一 Rust `Evidence` 类型同步生成；verifier 计算并核对 manifest、contract、fixture artifact 及具体 predicate/oracle 的实际 SHA-256，并绑定样本错误类别与控制状态。
- 第二轮实现把准入最终比较与 flush 放入共享门锁临界区，Worker/Supervisor 形成严格 terminal 顺序机并绑定 Snapshot→Dispatch→Observe；Host 以独立 oracle 在 250–500ms 内双采样。Harness 实际阻塞 Worker backend 并以有界管道读取 deadline 触发 Job kill；仅有进程退出证明的 cleanup 一律为 `Unproven` 并触发 quarantine。Evidence 增加稳定观察、完整 cleanup probes 与 dependency/SBOM/license/signature/notarization/provenance 字段，DIST-01 缺任一责任链即不能通过。
- 第三轮将 Supervisor 的裸 `Write` 参数替换为强制 deadline+kill 的 `DispatchTransport`，共用 `deadline_write` runtime；无 session、写前失败、flush 后取消和所有错序均进入 terminal。Worker 同时冻结并比较 snapshot sample、target oracle 与 Observe oracle。所有 kill 路径在返回前携带不可由外部伪造的 `CleanupEvidence` 回灌 Supervisor/quarantine，WorkerSession 漏记 Drop 也会 quarantine。Harness 的 Host oracle 读取 Worker mock 动作写出的独立 fixture 状态并真实等待 250ms；DIST verifier 对四个显式输入文件计算实际哈希，缺文件或漂移均拒绝。
- 第四轮把在途 cancel 改为 transport 内建 terminate+cleanup 后才返回，正常 dispatch response、Observe 与 shutdown response 全部经请求 deadline 读取，超时/EOF/解析失败先 kill 并 quarantine。Worker 与 Supervisor 都要求 Observe identity=`Match` 且 predicate=`Satisfied`；fixture TargetIdentity 的 PID/window/hash/topology/oracle 全字段派生并比较。mock marker 仅输出 `contract_only_observe_protocol`，明确不是实际动作证据。正常 shutdown 在 wait 后仍执行 containment probe 并原子记录保守 `Unproven`，Harness 各场景使用独立 HostSafetyState 避免预期 quarantine 串扰。
- 第五轮让 hello 与 snapshot 响应也复用请求 deadline 的 read-or-kill-cleanup 路径；shutdown ack 严格校验 kind/schema/identity/status/error，并用同一请求 deadline 同时约束 ack 与 `try_wait`，任何异常均先 kill+cleanup+quarantine。Evidence 新增不可省略的 `evidence_scope=contract_only|native`，全部 schema fixture 固定为 contract-only；verifier 对 native scope 拒绝复用 contract fixture artifact 或 provenance。

## Plan Change Log

- 2026-11-05：完成独立 workspace、协议/schema、framing/Supervisor、证据 verifier、mock Worker/Harness、基础跨平台 containment 与构建清单；记录原生 bridge 契约缺口，未宣称双平台原生通过。
- 2026-11-05：Quick Review 发现监督与证据契约缺少可执行细节；将 capability/身份/deadline、写入不确定性、宿主级 quarantine、suspended containment、唯一 schema、真实哈希与组合 harness 明确为任务。避免把可达副作用误报为未派发或把无效响应/伪证据判为成功。KEEP：独立 workspace、固定 revision、64 KiB framing、零重试、fail-closed native backend 与不接产品路径。
- 2026-11-05：按 Quick Review 重新推导协议、Supervisor、Windows containment、evidence/schema 与组合 harness；修复所有 non-deferred bad-plan/patch 行，避免 flush 失败误报、会话漂移、无效响应升级成功、deadline 失效、quarantine 绕过、占位哈希与手写第二 schema。保留 KEEP 项；macOS helper 完整进程树审计登记 deferred，native 双平台证据仍未完成。
- 2026-11-05：第二轮 Quick Review 发现单消息校验尚未形成运行时闭环；补充共享原子 admission gate、严格顺序机、pending dispatch 跨消息绑定、阻塞 I/O deadline kill、Host 独立稳定 Observe、cleanup fail-closed、完整 evidence/distribution/provenance 要求。避免合法但乱序消息、阻塞 Worker 或伪造 Worker verdict 绕过监督。KEEP：首轮已修 capability/session/哈希/schema/Windows suspended Job 行为及全部 fail-closed 边界。
- 2026-11-05：第三轮 Quick Review 将剩余问题限定为 8 个运行时闭环：flush 后取消终态、带 kill 的有界写、Worker pending sample/oracle 绑定、所有错序终态、deadline kill 回灌 quarantine、读取实际 fixture 的 Host 双采样、DIST 实际文件哈希、cleanup 强制记账。KEEP：loop 2 的 snapshot 协议、原子 gate、严格主路径、native fail-open 阻断证据与未完成双平台门禁。
- 2026-11-05：第四轮 Quick Review 发现 6 个最后闭环：在途 cancel 必须内建 terminate+cleanup，response/Observe read 必须使用请求 deadline，Worker mismatch/unsatisfied Observe 不得升级成功，mock oracle 不得表述为真实动作证据，fixture identity 必须全字段比较，正常 shutdown 也必须执行 cleanup 记账。KEEP：loop 3 transport/cleanup evidence/DIST 实际文件绑定；mock 输出统一降级为 contract-only，真实效果仍属 native 未完成门禁。
- 2026-11-05：第五轮 Quick Review 只保留三个收口项：hello/snapshot read deadline、shutdown ack+wait deadline、Evidence 中不可省略的 `contract_only/native` 范围。KEEP：所有前序安全闭环；契约 fixture 必须显式 contract-only，不能由平台字段或 observed/pass 冒充原生证据。
- 2026-11-05：完成第三轮 8 项运行时闭环及聚焦测试；派发只接受拥有 deadline/kill 的 transport，cleanup evidence 构造受限且所有非 Released 路径在返回前 quarantine，实际 fixture 双采样与 DIST 文件哈希不再可由常量/格式伪造。继续保留 macOS helper、完整原生 cleanup probes、native fail-open 和双平台分发证据为未勾选门禁。
- 2026-11-05：完成第四轮 6 项闭环及聚焦测试；cancel/read/shutdown 均在返回前完成 termination 与 cleanup 记账，Observe 双端拒绝 mismatch/unsatisfied，mock 输出降级为 contract-only，完整 fixture identity 不再只比较 fixture/oracle 两字段。未改变 native unsupported、macOS helper 与双平台证据门禁。
- 2026-11-05：完成第五轮 3 项收口与聚焦测试；hello/snapshot 不再存在无界 read，shutdown ack/wait 共用 deadline 且严格校验，Evidence scope 与 fixture artifact/provenance 绑定阻止 contract 证据冒充 native。未改变任何原生或双平台未完成门禁。

## Review Triage Log

- `supervisor.rs:95-103`：`high / bad_plan`；验证确认 `write_all` 后 `flush` 失败仍返回 `NotDispatched`，实际交付已不确定。
- `supervisor.rs:118-134`：`high / bad_plan`；验证确认未检查响应 kind/schema/status/error/disposition，拒绝响应可经 Observe 变为成功。
- `supervisor.rs:67-103`、`worker.rs:74-174`：`high / bad_plan`；验证确认 hello 后未冻结 attempt/Worker/fence/generation。
- `protocol.rs:255-265`：`medium / bad_plan`；验证确认 `capabilities_sha256` 被通配忽略，违反固定能力握手。
- `protocol.rs`、`supervisor.rs`、`worker.rs`：`high / bad_plan`；验证确认 deadline 仅序列化，从未参与准入或在途裁决。
- `supervisor.rs:47-149`：`high / bad_plan`；验证确认 quarantine 是实例字段，新建 Supervisor 可绕过宿主级隔离。
- `worker.rs:43-48`：`high / patch`；验证确认环境变量缺失会通过，未做到 trace/screenshots 必须显式为 `off`。
- `worker.rs:119-159`：`high / bad_plan`；验证确认 Worker 只查 sample/action，未校验 ref 及 predicate/oracle 冻结绑定。
- `containment/windows.rs:52-60`：`high / bad_plan`；验证确认进程先运行后入 Job，且 assign 失败会遗留未终止 Child。
- `containment/mod.rs:48-80`：`maybe-false / defer`；当前未证明固定上游会派生脱离 process group 的 helper；需 macOS 进程树静态审计与运行探针确定，若可达则为 high。
- `evidence.rs:124-212`：`high / bad_plan`；验证确认只检查哈希格式，合法 fixture 使用占位哈希也会通过。
- `evidence.rs:215-280`：`high / bad_plan`；验证确认样本 predicate/oracle、拒绝错误类别和部分控制状态未绑定。
- `generate-schema.rs:41-52`：`high / bad_plan`；验证确认 OpenSpec evidence schema 是未参与生成校验的第二份手写模型。
- `harness.rs:21-100`：`high / bad_plan`；验证确认可执行 harness 只有 hello/shutdown，未组合 dispatch/Observe/cancel/kill/cleanup/quarantine。
- `supervisor.rs:129-163`：`high / bad_plan`；第二轮确认最终比较与 flush 未被共享 admission gate 原子线性化，输入抢占可落在缝隙中。
- `supervisor.rs`、`worker.rs` 顺序状态：`high / bad_plan`；第二轮确认允许连续 dispatch、直接 Observe、未 Observe shutdown，invalid response 后也未 terminal。
- `supervisor.rs`、`harness.rs` deadline：`high / bad_plan`；第二轮确认同步管道和 backend 阻塞无 timeout/kill，字段比较不能终止在途调用。
- `protocol.rs`、`supervisor.rs`、`worker.rs` pending binding：`high / bad_plan`；第二轮确认 dispatch 与 Observe 各自合法但未绑定为同一步。
- `supervisor.rs` Observe：`high / bad_plan`；第二轮确认 Host 直接信任 Worker verdict，未执行独立 oracle 与 250ms 间隔双采样稳定判定。
- `containment/*` cleanup：`high / bad_plan`；第二轮确认当前 Released 只证明主进程/pgid 退出，完整只读探针缺失时必须改为 Unproven 并 quarantine。
- `evidence.rs` 正常 pass profile：`high / bad_plan`；第二轮确认未绑定 response boundary、generation 单调、input source 与稳定观察次数。
- `evidence.rs` DIST-01：`high / bad_plan`；第二轮确认 schema 无依赖/SBOM/许可/签名/公证字段，测试 artifact 不能证明分发责任链。
- `generate-schema.rs` build manifest：`high / bad_plan`；第二轮确认 tree/dirty/targets/CRT/build command/artifacts 声明未被真实性校验。
- `backend/mod.rs` native：`high / bad_plan`；真实 snapshot/ref/Observe 仍缺桥接输入，当前 fail-closed 正确但冻结 Approach 尚未完成；按既定 qualified-ref 与 Host oracle 约束补全 AD/bridge contract，不得另建通道或隐式选目标。
- `supervisor.rs:321-366`：`high / patch`；第三轮确认 flush 后 cancel 未置 terminal，后续响应仍可升级为 Observed。
- `supervisor.rs:254-318` 写端 deadline：`high / bad_plan`；第三轮确认同步 write/flush 可无限阻塞并占住 gate；必须改为只有具备 deadline+kill 能力的 transport 才可派发。
- `worker.rs` pending sample：`high / patch`；第三轮确认 Worker 未在 SnapshotReady/AwaitObserve 保存并比较 sample，且 Observe target oracle 未与 oracle_id 相等。
- `supervisor.rs` 错序终态：`high / patch`；第三轮确认无 session dispatch 与写前失败仍可继续会话，必须终态并要求关闭 Worker。
- `harness.rs` deadline quarantine：`high / patch`；第三轮确认阻塞 kill 的 Unproven 未回灌对应 Supervisor/HostSafetyState。
- `harness.rs` Host oracle：`high / bad_plan`；第三轮确认固定 true stub 可伪造动作效果；必须读取由 fixture 动作产生的独立状态并用真实 250–500ms 间隔双采样。
- `evidence.rs` DIST 文件哈希：`high / bad_plan`；第三轮确认 SBOM/许可/依赖/provenance 只检查 64 字符格式；必须绑定 verifier 输入的实际文件哈希，缺制品时 DIST pass 不可构造。
- `cleanup` 强制闭环：`high / bad_plan`；第三轮确认调用方可漏调或直接传 Released；termination API 必须产出受类型约束的 cleanup evidence，并在返回前原子更新 Host quarantine，完整 probes 未实现时只能 Unproven。
- `supervisor.rs` flush 后 cancel：`high / bad_plan`；第四轮确认 terminal/Unknown 后仍由调用方另行 kill，公开 API 必须原子 terminate+cleanup 后返回。
- `harness.rs` response/Observe deadline：`high / bad_plan`；第四轮确认只有 dispatch write 和单独阻塞样本有 timeout，正常 read 必须绑定消息 deadline 并在超时后 kill+quarantine。
- `supervisor.rs`、`worker.rs` Observe verdict：`high / patch`；第四轮确认未要求 Worker identity/predicate verdict 成功，mismatch/unsatisfied 仍可能 Observed。
- `harness.rs` mock oracle：`medium / bad_plan`；第四轮确认 Worker 自写 marker 不能证明真实桌面效果；只可验证独立 oracle 契约并输出 `contract_only`，不得称动作已生效。
- `TargetIdentity` fixture：`high / patch`；第四轮确认 mock oracle 未全字段比较且固定 hash 与 fixture 无关；测试 identity 必须派生并比较 PID/window/hash/topology 全字段。
- `WorkerSession::shutdown`：`high / bad_plan`；第四轮确认正常退出未运行 containment/cleanup 记账；所有关闭路径都必须在返回前产出 CleanupEvidence 并更新 Supervisor，完整 probes 缺失即 quarantine。
- `harness.rs` hello/snapshot：`high / patch`；第五轮确认两处仍直接阻塞 read，必须复用请求 deadline 的 read-or-kill-cleanup 路径。
- `harness.rs` shutdown ack/wait：`high / patch`；第五轮确认未校验 kind/identity/status 且 wait 无 deadline，必须严格验证后在期限内 wait，否则 kill+quarantine。
- `Evidence` scope：`high / bad_plan`；第五轮确认 contract fixture 可验证为 Windows observed/pass；schema/verifier 必须要求 `evidence_scope = contract_only | native`，fixture 固定 contract-only，原生证据不能复用 fixture artifact/provenance。

## Design Notes

上游 Rust 命令 API是公开 crate 内部实现细节，首次实现先以固定 git rev 编译验证；若核心命令不可从外部 crate 调用，不得改用 CLI/C ABI 绕过，应记录为候选阻断并保持 mock/contract 层可验证。Linux 只用于契约测试，不提供原生 Driver 结论。

## Verification

**Commands:**
- `cargo fmt --manifest-path spikes/agent-desktop-driver/Cargo.toml -- --check` -- 格式通过。
- `cargo test --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked` -- 协议、framing、Supervisor、evidence 与 containment 契约测试通过。
- `cargo clippy --manifest-path spikes/agent-desktop-driver/Cargo.toml --locked --all-targets -- -D warnings` -- 零 warning。
- `cargo run --manifest-path spikes/agent-desktop-driver/Cargo.toml --bin generate-schema --locked -- --check` -- 生成 schema 与提交版本一致。
- `npx -y @fission-ai/openspec@latest validate cu-s4-agent-desktop-driver-spike --strict --no-interactive` -- OpenSpec 严格校验通过。
- `git diff --check -- spikes/agent-desktop-driver openspec/changes/cu-s4-agent-desktop-driver-spike docs/specs/epic-CU/story-CU-S4` -- 无空白错误。
