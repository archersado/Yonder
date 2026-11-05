# agent-desktop Driver Spike 架构对抗审阅

## 结论

**Verdict：BLOCK（修复 1 个 Critical 与 5 个 High 后可进入 Spike 实施）。** 当前边界方向正确，尤其是 Driver 低层化、Worker 隔离、`unknown` 和后继 Apply 门禁；但现有规则仍允许两套都逐字遵守 AD 的实现，在派发线性化、输入来源判定、Observe 结果、清理后准入、构建面和最终 ADR 结论上产生互不兼容的行为。

审阅对象：`agent-desktop-driver-spike.md`、AD-CU-07、CU-S4 三份设计与 `cu-s4-agent-desktop-driver-spike` OpenSpec。本文只审阅，不修改方案文件。

## 对抗实现

为验证规则是否真正收敛实现，构造两套表面合规方案：

- **实现 A（宽松边界）**：Harness 在等待上游 interaction lease 前检查 fencing/generation；只要 OS 事件带 injected 标志就视为 Driver 输入；动作返回后以“同一 qualified ref 仍存在”作为 Observe 成功；Worker kill 后查询进程与按键状态，失败样本结束但释放 Yonder task lease；使用上游 C ABI、动态 CRT 与平台各自默认构建参数。
- **实现 B（保守边界）**：Harness 与取消共享同一派发临界区，并在首字节/FFI 调用前重检 fencing/generation；只抑制能与本次 Worker/attempt 证明关联的合成输入，未知来源按用户输入处理；Observe 使用预冻结谓词和原生 target identity；Worker kill 后进入全局桌面 quarantine，直到无副作用探针和全新实例 Observe 均通过；使用 JSON bridge/最小 fork、静态 CRT 和固定 toolchain/features。

两套方案都能声称满足当前 AD-1～7，却会对相同竞态、事件、证据和结论给出不同结果。这说明下列项仍是架构漏洞，而不是实现细节。

## Critical

### C1 — “任一平台硬失败即整体淘汰”与“允许单平台采用”互相冲突

**证据：** Spine AD-7 规定 Windows/macOS 任一硬淘汰条件命中即候选 `Rejected`；OpenSpec“任一硬门槛失败”也要求候选路线淘汰。与此同时 Spine `Deferred`、AD-CU-07“结论出口”和 OpenSpec `design.md` 允许“仅单平台且收益足以承担双栈成本”。

**不兼容结果：** Windows 全部通过、macOS 因平台可用性或签名责任链失败时，验证器 A 必须整体 Rejected；验证器 B 可以给出“仅 Windows”结论。两者都能引用现有条款。

**必须修复：** 冻结结论状态机和失败作用域。明确哪些安全硬失败淘汰整个引擎（例如误投递、抢占失效、unknown 重试、残留输入状态），哪些仅使某平台不合格且仍允许单平台候选（如纯平台覆盖/分发差异，如果产品允许）。若坚持“任一平台硬失败整体淘汰”，删除所有单平台出口；若保留单平台出口，新增 `engine-fatal` 与 `platform-disqualifying` 的封闭枚举和判定表，ADS-10 与 OpenSpec 同步。

## High

### H1 — 派发/取消没有唯一线性化点，存在检查后派发竞态

**证据：** AD-3 只写“每次派发前复核”，AD-4 写“先冻结后续派发”，OpenSpec 使用“线性化派发点”但未定义该点、锁/序列所有者或 Worker 接受边界。

**不兼容结果：** 实现 A 在等待上游 interaction lease 前完成复核；等待期间用户输入递增 generation，A 仍可进入 Driver。实现 B 在共享临界区、首字节/FFI 调用前复核并拒绝。两者都能把自己的检查称为“派发前”。

**必须修复：** 新增可执行 AD：取消/用户抢占与派发必须共享同一个 Application-owned admission gate；定义派发线性化点（建议为 Supervisor 向已绑定 `worker_instance_id + attempt_id` 的 Worker 交付动作前的最后一步），并冻结严格顺序：observer ready → task fence/lease → capture generation → snapshot/target verify → compare control/fence/generation → 唯一交付。交付之后一律按 in-flight；交付之前一律 not-dispatched。明确上游 interaction lease 等待发生在哪一侧，以及等待后是否必须重检。

### H2 — `input_generation` 与来源分类没有跨平台契约，fail-closed 规则缺失

**证据：** AD-4 只要求“递增 generation 并输出安全分类”，没有规定作用域、单调性、observer 丢事件/重启、未知来源、注入标志可信度和自发事件关联方式。

**不兼容结果：** 实现 A 忽略所有 OS 标记 injected 的事件；实现 B 只忽略与当前 attempt 可证明关联的事件，并把其余 injected/unknown 当用户输入。第三方自动化、上游库内部事件或 observer 短暂失联会使两者产生相反抢占结果。

**必须修复：** 定义最小封闭分类（例如 `driver_current_attempt | user | unknown | observer_lost`）及其安全映射：除可证明属于当前 attempt 的 Driver 事件外，其余均递增 generation 并冻结派发；observer 未 ready、丢失、重启或 generation 回退同样 fail closed。固定 generation 的 OS-session 作用域、宽度/回绕和 Worker 重启继承规则，并要求 Windows/macOS 分别证明“当前 attempt 合成事件”的关联机制，而不是只靠时间窗口。

### H3 — 双平台证据只有字段名提示，没有唯一结果模式与 Observe oracle

**证据：** AD-7 要求相同“样本 ID 和判定字段”，但 Worker 私有消息字段被 Deferred；`architecture-design.md` 只列安全枚举、generation、耗时等自然语言字段。“相同目标身份”“预期条件”“已 Observe 验证”没有稳定 predicate/identity 定义。

**不兼容结果：** 实现 A 把 ref 仍存在视为 Observe 成功；实现 B 要求冻结的业务谓词和原生身份均通过。两平台还可以输出同名但语义不同的 `unknown`、`cleanup_ok` 或时间值，使独立验证无法机械比较。

**必须修复：** 在 Spike 层冻结版本化 evidence schema（不等于产品协议），至少包含 `schema_version`、源码/制品哈希、platform/arch、sample/predicate ID、attempt/worker/fence、generation before/after、派发边界、delivery disposition、Observe identity/predicate verdict、cleanup verdict 和最终 outcome；使用封闭枚举和单调时钟。每个样本预先定义不含敏感正文的 Observe predicate 与 target identity 判定，不能由平台实现自行解释。

### H4 — Worker kill 后的 containment、清理证明与 quarantine 出口未收敛

**证据：** AD-5 要求证明进程树、lease、按键/鼠标和临时资源释放，但未规定如何 containment detached helper、清理探针能否发送补偿输入、失败后 task/global lease 是否释放，以及何时解除“不能启动下一 Worker”。

**不兼容结果：** 实现 A 查询状态后释放 task lease；实现 B 保持全局桌面 quarantine。另一个实现还可能发送 key-up/button-up 来“清理”，该补偿本身又是未观察副作用，但当前规则没有禁止。

**必须修复：** 定义 Worker containment 契约（不得逃逸受监管进程集合；平台机制或等价证明）、无副作用清理探针和失败后的唯一状态。建议任何残留/不可证明释放都进入 global desktop quarantine，禁止新的 CUA 派发，直到人工/宿主重启或明确的只读恢复程序通过；禁止用合成输入补偿来伪造清理成功。区分“样本失败”“任务 lease 释放”和“全局桌面重新准入”。

### H5 — “同一 revision”不足以保证两平台验证同一个 Driver 集成面

**证据：** Stack 允许 `v1.1.0` 或唯一修复 commit；AD-1 又允许“C ABI/最小桥”，`architecture-design.md` 允许 C ABI 丢 disposition 时最小 fork/桥。未固定 Cargo.lock、Rust toolchain、features、target triple、CRT 链接、入口/符号集合、fork diff 或 bridge protocol。

**不兼容结果：** 实现 A 使用原始 C ABI 与动态 CRT，丢失 disposition；实现 B 使用 patched JSON bridge/静态 CRT。两者可共享上游 revision，却实际验证不同 API、构建图和结果语义；Windows/macOS 也可各选不同入口，双平台结果不可比较。

**必须修复：** 开跑前冻结完整 build manifest：上游 commit、fork patch hash（无 patch 也显式为空）、Cargo.lock、Rust toolchain、target triple、features、CRT/链接策略、入口与允许符号/命令集合、bridge schema/version、依赖/SBOM 哈希。Windows/macOS 必须使用同一语义入口和同一 patch set；若因平台必须不同，差异需进入样本和 ADR，不得仍称“同一 revision 已证明共享引擎”。

## Medium

### M1 — Harness 自身写成“当前仓库 revision”，不可复现

Stack 中 `Yonder Spike harness | 当前仓库 revision` 会随执行时点变化。应在首次运行前记录确切 Git commit、dirty diff 哈希和构建命令；dirty 状态无法重现时不得作为最终证据。

### M2 — deadline、耗时和观察稳定窗口未冻结

文档要求记录耗时和 deadline，但没有规定单调时钟、超时值、Observe 稳定窗口或重试次数。两个实现可通过选择不同时间预算得出相反结论。应在样本清单冻结每类动作的 deadline、稳定窗口和“零自动重试”计数方法。

### M3 — “签名责任链可建立”的通过证据不明确

一套实现可把本地测试签名/adhoc codesign 视为可建立，另一套要求可验证的 Yonder 发行主体、Windows Authenticode 和 macOS Developer ID/notarization。应明确 Spike 需要真实平台验证、测试证书验证，还是只需构建流程证明；三者不能混为同一 ADS-09 通过结果。

### M4 — 多显示器/DPI 的“相同样本”缺少拓扑等价定义

Windows 96/192 DPI 与 macOS Retina/缩放不是天然同一语义。应冻结逻辑坐标/物理像素、主副屏、负坐标、跨屏和缩放变化的拓扑元数据；否则平台可各挑容易通过的显示样本。

## 已经收敛、无需扩大范围的部分

- 候选只作为低层 Driver，任务状态与恢复仍由 Yonder 持有；
- C ABI/桥不得进 Desktop 主进程，不开放 HTTP/TCP，不接 Gateway/SQLite；
- raw 坐标、无目标按键、CDP、上游 MCP 与默认敏感 trace/screenshots 被明确禁用；
- in-flight kill 不能报告确定取消，交付不明不得自动重试；
- Spike 通过不等于产品 Apply，通过后仍需新 Story/OpenSpec。

这些边界应保留；本审阅建议只补齐它们之间的线性化、数据形状、恢复准入与结论状态机。

## 2026-11-05 修订后复核

**Verdict：BLOCK（C1、H1、H2、H4 已关闭；H3、H5 仍有两个可机械触发的残留阻断）。**

### 已关闭

- **C1 PASS：** AD-CU-07、Spine AD-9、Story 与 OpenSpec 已统一为“双平台共享或整体淘汰”，删除单平台长期双栈出口。
- **H1 PASS：** 派发、抢占、取消共用 Application-owned admission gate；完整 JSONL 帧 `write + flush` 被定义为唯一线性化点，前后分别是 `not_dispatched` / `in_flight`，上游 lease 等待顺序也已冻结。
- **H2 PASS：** 已冻结宿主会话级 `u64 generation`、四类来源、observer fail-closed 和 Worker 重启不重置规则；INPUT-01～03 提供对应固定样本。
- **H4 PASS：** 已规定 Windows Job Object、macOS 受监管进程组、禁止脱离 helper、只读清理、禁止合成补偿输入、global quarantine 与唯一重新准入出口。

### 残留阻断

#### R1（H3）— Evidence schema 尚未编码跨字段安全不变量，sample catalog 也未给出实际 `predicate_id`

`evidence-schema-v1.json` 已经固定字段和枚举，这是实质进展；但当前 schema 仍允许以下记录通过 JSON Schema 校验：

- `observe_identity = mismatch`
- `observe_predicate = unsatisfied`
- `cleanup_verdict = residual`
- `global_quarantine = true`
- `outcome = pass`

同样，`sample-catalog.md` 的列名写着 “Predicate / identity oracle”，但每行只有自然语言说明，没有规范要求的实际 `predicate_id` 值，也没有可散列的 predicate/oracle 定义。因此 Windows 实现可把 `IDENTITY-01` 的 predicate 命名/解释成一套语义，macOS 实现使用另一套语义，仍能声称遵循目录。

**关闭条件：** 在 schema 中用 `allOf`/`if-then` 或配套确定性 verifier 固定至少这些关系：`outcome=pass` 必须 `observe_identity=match`、`observe_predicate=satisfied`、`cleanup_verdict in [not_required,released]`、`global_quarantine=false`、结果/dispatch/delivery 组合一致；`unknown`、`not_dispatched`、`unsupported`、`environment_unavailable` 也要有封闭组合。Sample catalog 每行增加稳定 `predicate_id` 与 `oracle_id`，并把具体定义放入版本化、可散列的 fixture/oracle 清单。

#### R2（H5）— `agent-desktop-spike-bridge/v1` 目前只有名称，没有可冻结/散列的数据形状

方案已选定唯一最小 Rust bridge、固定 upstream commit、stdio JSONL、toolchain/target/features/CRT 等 build manifest 字段，也禁止平台改换入口；但仓库中只有 `evidence-schema-v1.json`，没有 bridge v1 的 request/response/handshake schema 或 Rust 单一来源。于是两个实现仍可在以下方面不同而都标记同一个 `agent-desktop-spike-bridge/v1`：

- 动作帧是否含 attempt/Worker/fence/generation/target identity；
- Worker 在何时确认收帧、取得上游 lease 与调用 Driver；
- disposition、Observe 和 error 的必需字段与拒绝未知字段策略；
- JSONL 最大帧、deadline、版本协商和异常/EOF 映射。

Build manifest 虽要求记录 `bridge schema`，当前却没有实际文件/生成物可供哈希，无法证明 Windows/macOS 验证的是同一语义入口。

**关闭条件：** 开跑前新增版本化 bridge v1 契约，优先由 Spike Rust 类型生成 JSON Schema；至少固定 handshake、action request、result/error、必需身份字段、frame 上限、未知字段拒绝、deadline/EOF/invalid JSON 映射，并在 build manifest 中记录该 schema/生成物哈希。两平台合约测试必须读取同一契约。

## 2026-11-05 第二次修订后复核

**Verdict：BLOCK。新增文件已关闭大部分 H3/H5，但各剩 1 个可复现的不一致，当前仍不能把 Story 从 design-review 推到 ready。**

### 已确认通过的补充

- `sample-catalog.md` 现已为每条样本绑定稳定 `predicate_id` / `oracle_id`，`sample-definitions-v1.json` 提供版本化定义与哈希责任链。
- Evidence schema 固定 source revision、definition/contract hashes 与 `retry_count=0`；仓库 fixture 实测为 `observed-pass.json` 接受、`invalid-observe-pass.json` 拒绝。
- `bridge-contract-v1.md` 已冻结单 Worker、JSONL、hello/dispatch/observe/shutdown、身份回显、乱序/未知字段拒绝、在途无 cancel、Rust 类型单一来源和生成 schema 门禁。

### 残留 R3（H3）— Evidence schema 仍未把 sample 与允许的结果组合绑定

当前跨字段条件只约束“若 `action_result=observed`，则 Observe 必须成功”，以及 cleanup/quarantine；它没有约束某个 sample 要以什么 action/result/Observe 组合才可 `outcome=pass`。只读实测从合法 `ELEMENT-01` fixture 派生以下两条，均被 Draft 2020-12 schema **接受**：

1. `ELEMENT-01 + action_result=unknown + delivery=unknown + Observe=unavailable + outcome=pass`；
2. `ELEMENT-01 + action_result=not_dispatched + delivery=not_delivered + Observe=unavailable + outcome=pass`。

但 catalog 明确要求 ELEMENT-01 完成 qualified-ref 动作并稳定 Observe。因此两个平台仍可用不同结果组合把同一样本判为通过。`sample-definitions-v1.json` 目前只有自然语言 definition，没有机器可执行的 expected-result profile；已有非法 fixture 只覆盖 `observed + unavailable`，没有覆盖上述 sample/result 错配。

**关闭条件：** 给每条 sample 绑定封闭 `expected_profile_id`（或等价确定性 verifier），机器校验 `sample_id + action_result + dispatch + delivery + Observe + cleanup + quarantine + outcome`。至少加入上面两条非法 fixture 并证明拒绝。FAULT/CANCEL 这类“预期 unknown/not_dispatched 但样本通过”的 profile 必须与 ELEMENT/APPSET 这类“预期 observed”的 profile 分开，不能用全局 `outcome=pass ⇒ observed` 简化。

### 残留 R4（H5）— Bridge contract 引用不存在的动作 allowlist，且 framing 失败语义未完成

`bridge-contract-v1.md` 规定 `dispatch.action_id` “只能引用 sample definitions 的 allowlist 固定动作”，但当前 `sample-definitions-v1.json` 只有 `predicates` 与 `oracles`，没有 `actions`/allowlist。不同平台因此仍可自行选择同名 `action_id` 的实际 Driver 调用和参数，无法证明使用同一语义入口。

此外，原 H5 关闭条件中的最大 JSONL frame、oversized line、invalid UTF-8/JSON、EOF/partial line 映射尚未定义。现有“未知字段/unknown kind/schema mismatch 为 unknown”不覆盖解析前 framing 失败；一个 Worker 可把 partial EOF 视为 `not_delivered`，另一个视为 `unknown`。

**关闭条件：** 在 sample definitions 或独立版本化清单中增加带哈希的 `action_id → 固定 Driver operation + 非敏感参数形状` allowlist；bridge contract 冻结最大 frame/line、UTF-8/JSON 解析、partial EOF 与 oversized frame 的唯一 `unknown`/终止规则，并要求生成的 Rust schema/合约测试覆盖这些负例。

## 2026-11-05 最终候选复核

**Verdict：BLOCK（H3 已关闭；H5 仍有 1 个 action/sample 绑定阻断）。**

### H3 PASS

- Draft 2020-12 实测：`observed-pass.json` 接受；`invalid-observe-pass.json` 与 `invalid-element-unknown-pass.json` 均拒绝。
- 15 个 catalog sample 均有 expected-result profile，且 `predicate_id` / `oracle_id` 全部解析到版本化 definitions。
- `retry_count=0`、cleanup/quarantine、Observed 与各 sample 的 pass 结果组合均已有机器约束。

### H5 framing/握手 PASS

- Bridge 已冻结 65,536-byte UTF-8 JSONL 帧、incremental newline parsing、malformed JSON、unknown 字段/kind、oversize、partial EOF、乱序/重复响应的关闭与 `not_dispatched`/`unknown` 映射。
- Hello 双向绑定 source revision、bridge contract、sample definitions、action allowlist 与 capabilities hash；Rust 类型生成 schema 及 build-manifest hash 门禁明确。

### 残留 R5（H5）— Sample 没有唯一绑定 action，且 INPUT-03 无任何合法 action

`action-allowlist-v1.json` 已定义固定动作及参数边界，但它只给出多对多 `allowed_samples`，catalog 和 evidence expected profile 都没有冻结每个 sample 实际使用的 `action_id`。只读一致性检查得到：

- `INPUT-03` 不在任何 action 的 `allowed_samples` 中，但 evidence schema 又强制每条记录必须有 `action_id`，因此不存在同时满足 schema 与 allowlist 的 INPUT-03 证据。
- 其余多数样本有两个合法动作。例如 `ELEMENT-01` 同时允许 `ACTION-SNAPSHOT` 与 `ACTION-TYPE-FIXTURE-TEXT-QUALIFIED-REF`，`CANCEL-01/02` 同时允许 click 与 type。Windows 可以用 click、macOS 可以用 type，仍使用相同 sample ID 并各自通过 expected profile，无法证明同语义样本。
- `IDENTITY-02` 的 pass profile 要求 `action_result=rejected`，但 allowlist 同时允许只读 `ACTION-SNAPSHOT`；二者没有唯一组合约束。

**关闭条件：** 在 catalog/definitions 中为每条 sample 冻结唯一 `action_id`，或显式拆成带 action 后缀的独立 sample ID；如果一个场景需要 snapshot + action，分别定义准备步骤与唯一受测 action，evidence 的 `action_id` 记录受测动作。为 `INPUT-03` 指定 `ACTION-NONE`（并加入 allowed_samples）或专门的“attempted but blocked”动作定义。Evidence schema/确定性 verifier 必须校验 `sample_id ↔ action_id`，并增加至少一个 action/sample 错配负例。

## 2026-11-05 最终复核结论

**Verdict：PASS。上轮 H3/H5 及 R5 均已关闭，无残留架构阻断。**

只读机械复核结果：

- 15 个 catalog sample 与 allowlist 的 sample 集完全一致；每个 sample 恰好出现一次，`sample_id → action_id` 映射逐项相同。
- Evidence schema 对全部 15 个 sample 机械约束唯一 action；`INPUT-03` 与 `DIST-01` 固定 `ACTION-NONE`。
- Schema fixtures：`observed-pass.json` 接受；Observe 失败、ELEMENT unknown+pass、ELEMENT 错误 action 三个负例全部拒绝。
- 15 个 sample 的 expected-result profile 全覆盖；predicate/oracle ID 均解析到版本化 definitions。
- Bridge 的 65,536-byte framing、UTF-8/JSON/oversize/partial EOF、hash 握手、action allowlist、Rust 单一来源和 build-manifest hash 责任链完整。

本结论只表示设计与 Spike 契约通过 Reviewer Gate；不表示 agent-desktop 技术路线、Windows/macOS 原生样本或产品 Apply 已通过。
