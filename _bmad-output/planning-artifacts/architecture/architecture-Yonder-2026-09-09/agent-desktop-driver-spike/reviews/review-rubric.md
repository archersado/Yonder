# CU-S4 agent-desktop Driver Spike 架构基底审阅

审阅日期：2026-11-05
审阅范围：`agent-desktop-driver-spike.md`、AD-CU-07、CU-S4 Story 三份设计、`cu-s4-agent-desktop-driver-spike` OpenSpec。
审阅方式：按 bmad-architecture Reviewer Gate 的 good-spine checklist；只读审阅，不修改被审文档。

## Verdict

**有条件通过，当前不应从 `design-review` 推进到 `ready`。** 核心边界与 ADS-01～08 的大部分控制语义一致，且没有削弱父架构的 Application 状态所有权、逐步 Observe、用户输入冻结和 `unknown` 不重试约束；但“单平台结论”与“任一平台失败即整体淘汰”互相冲突，ADS-09 缺少正向 OpenSpec 场景，且 Driver 入口/传输仍保留多选，允许两平台各自作出不兼容实现。解决下列 High 后可进入 Spike 实施。

## 高优先级发现

### H1 — 单平台结论出口与硬门槛互相矛盾

- **证据：** AD-CU-07 与 architecture spine Deferred 都允许“仅单平台采用且收益足以承担双栈成本”；但产品需求 ADS-10 写“所有 hard gate 均通过后才能提出产品 Apply”，OpenSpec 的“任一硬门槛失败”场景又规定任一平台失败即“候选路线必须淘汰”。
- **风险：** Windows 失败、macOS 通过时，一个实现者会得出“整体 Rejected”，另一个会进入“macOS-only/Windows-only”后继 Apply；两者都能声称遵循文档。
- **建议修复：** 二选一并贯穿 ADR、Story、spine、OpenSpec：
  1. 仅接受双平台共享引擎，任一平台硬失败则整体淘汰；或
  2. 明确把 hard gate 分为“平台内不可豁免”与“跨平台共享门槛”，允许某平台失败后只淘汰该平台，并为单平台路线增加明确收益阈值、双栈维护成本门槛与独立 ADR。不能继续同时保留两种语义。

### H2 — ADS-09 没有完整 Delta Spec 正向契约

- **证据：** 产品需求 ADS-09 要求固定源码可重现自建、干净 Windows 无未声明 CRT、SBOM/许可清单和平台签名责任链；OpenSpec 只在“任一硬门槛失败”一句中提到未声明运行库/签名，未定义成功场景，也未覆盖可重现构建、SBOM、第三方许可清单、Windows Authenticode 和 macOS 签名/公证各自应产出的证据。
- **风险：** tasks 勾选后仍无法按 spec 判断 ADS-09 是否完成，独立 Verification Goal 缺少可执行的逐项依据。
- **建议修复：** 新增“可重现构建与平台分发责任链”Requirement，至少包含固定源码/依赖锁/双平台制品哈希、干净 Windows 启动、SBOM+license manifest、Windows 自签测试制品验证、macOS 签名/公证测试责任链五组场景，并明确 Spike 只证明能力，不使用生产证书。

### H3 — 入口与传输多选会让两平台 Spike 不可比较

- **证据：** AD-1/AD-CU-07/Story/OpenSpec 均允许“C ABI 或最小桥”以及“继承 stdio 或既有私有 IPC”；同时 architecture-design 又要求完整 delivery/retry disposition，发现 C ABI 丢失时“需最小 fork/桥后重新验证”。
- **风险：** Windows 可选 C ABI+stdio，macOS 可选 CLI/自定义桥+另一私有 IPC，最终差异可能来自包装方案而非平台能力；两个实现单元仍能完全按文档构建出不可比较的结果。
- **建议修复：** 在实现前冻结唯一分阶段路线，例如“共同 Worker JSONL-over-inherited-stdio；优先同一组 C ABI 调用；若 status-only 丢 disposition，则两个平台一起切到同一最小 fork API 并重跑全部相关样本”。把选择记录为 tasks 的前置门禁，不能由平台实现者各自决定。

### H4 — 架构 spine 把尚未接受的新决定全部标成 `[ADOPTED]`

- **证据：** AD-1～AD-7 全标 `[ADOPTED]`，而 AD-CU-07 状态仍为 Proposed、Story 为 design-review；用户只明确了“当 Driver 使用”，没有逐项接受 C ABI/Worker 形式、五日样本、零容忍门槛或单/双平台结论。
- **风险：** 下游会把仍待审或待 Spike 证明的方案误读为既有现实/用户定案，绕过 Proposed ADR 门禁。
- **建议修复：** 仅对从父架构继承的既定约束使用 `[ADOPTED]`，本地 Proposed 规则不加该标签，或明确标注它们是“Spike 执行约束，不是产品采用决定”。Reviewer Gate 完成后才能把 Story 状态推进到 ready-for-spike。

## 中优先级发现

### M1 — 输入来源“可靠区分”缺少可复核判据

ADS-04 的淘汰语义正确，但未冻结事件数量、交错顺序、允许的误报/漏报、后台/前台/修饰键/鼠标按钮组合，也没有定义 generation 线性化点。建议把真实用户与合成输入样本矩阵、零漏报要求、误报处理及派发前后时间顺序写入 OpenSpec；否则独立验证只能主观判断“可靠”。

### M2 — `preempted` 与 `unknown` 的可观察状态存在竞态歧义

visual-interaction-design 同时定义 `preempted`（检测到用户输入）和“在途 Worker 被终止时必须先显示 `unknown`”。若用户输入发生在动作已派发后，调用方可能显示两个不同终态。建议明确：`preempted` 是冻结控制事实；若存在在途动作，其动作结果仍为 `unknown`，样本最终判定由两条正交字段表达，不用单一状态覆盖。

### M3 — 版本尚未真正固定，不能开始实现

Stack 写 `v1.1.0；若 CRT 阻断则……唯一修复 commit`，tasks 的“冻结唯一源码 revision”仍未完成。这对 design-review 阶段可接受，但在变更为 ready/implementing 前必须落成唯一 SHA、依赖锁和哈希；否则两个平台可能运行不同源码。建议把 Story README 当前状态区明确写为前置阻塞项。

### M4 — ADR 的研究链接路径错误

`AD-CU-07-AGENT-DESKTOP-DRIVER-SPIKE.md` 位于 `_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/`，其中 `../../research-vercel-labs-agent-desktop-yonder-cua/...` 会解析到 `_bmad-output/planning-artifacts/research-...`；实际报告位于 `_bmad-output/research-...`，应再向上一级。此问题影响审计可追溯性，但不改变架构语义。

## Checklist 结果

| 检查项 | 结果 | 说明 |
|---|---|---|
| 修复 epic→story 的真实分歧点 | 部分通过 | 控制平面、目标身份、抢占、取消、Observe 已固定；入口/传输仍多选。 |
| AD Rule 可执行且对应 Prevents | 部分通过 | AD-1～7 基本可执行；AD-7 与单平台出口冲突。 |
| Deferred 不放任下层漂移 | 部分通过 | 产品 Apply 延后合理；单平台双栈结论缺少一致判定。 |
| 技术版本已现实核验并固定 | 部分通过 | v1.1.0 有研究证据，但最终 revision 尚未冻结。 |
| 与 brownfield/父架构一致 | 通过 | 未削弱 Application 状态所有权、Gateway、SQLite、Observe 与 unknown 边界；trycua 仍为当前基线。 |
| 覆盖 Story ADS-01～10 | 部分通过 | ADS-01～08、10 大体覆盖；ADS-09 Delta Spec 不完整。 |
| 运行/部署/运维维度 | 部分通过 | Worker、清理、干净机、签名、证据边界已出现；可重现构建/SBOM/签名正向验收未成契约。 |
| 无第二状态所有者/协议 | 通过 | 明确禁止上游模型进入 Domain/Gateway/持久化。 |

## 建议 Gate 结论

先修复 H1～H4，再完成最终 revision 冻结并补充 ADS-09 与输入抢占判据。随后重新运行 spine lint 和 OpenSpec 校验；通过后可把 CU-S4 从 `design-review` 推进为 `ready`，但仍只授权 Spike，不授权产品 Apply。

## 2026-11-05 修复复核

**结论：PASS。** 上轮 H1～H4 与 M1～M4 均已在当前架构、Story 与 OpenSpec 中关闭；本审阅镜头没有残留阻断项。此结论只表示可通过本轮架构 Reviewer Gate，仍只授权 CU-S4 Spike，不授权产品 Apply。

| Finding | 复核结果 | 当前证据 |
|---|---|---|
| H1 单平台出口冲突 | 关闭 | AD-CU-07、spine AD-9、Story ADS-10、OpenSpec 与 tasks 统一为“仅双平台共享或整体淘汰”，任一平台硬失败不保留单平台出口。 |
| H2 ADS-09 正向契约缺失 | 关闭 | Delta Spec 新增“可重现构建与平台分发责任链”，覆盖固定 build manifest、干净 Windows、SBOM/许可、Authenticode 与 macOS Developer ID/hardened runtime/notarization/stapling/Gatekeeper。 |
| H3 入口/传输多选 | 关闭 | 唯一路线冻结为最小 Rust bridge + `agent-desktop-spike-bridge/v1` JSONL over inherited stdio；明确禁止 status-only C ABI、CLI 和其他 IPC，两平台不得各选入口。 |
| H4 Proposed 规则误标 ADOPTED | 关闭 | spine AD-1～AD-9 已全部移除 `[ADOPTED]`；继承约束只保留在 Inherited Invariants。 |
| M1 输入可靠性判据 | 关闭 | 封闭输入分类、宿主会话 `u64 generation`、observer fail-closed 与 sample catalog 的 INPUT-01～03 已冻结；真实/合成各 10 轮要求零漏分、零误分。 |
| M2 preempted/unknown 竞态 | 关闭 | visual-interaction-design 明确 `preempted` 为控制事实、`unknown` 为动作结果，两者正交；另以 `global_quarantine` 表达清理门禁。 |
| M3 revision 未固定 | 关闭 | 上游固定为 `d3dc190afbf34cd2377252398ddd5c30efbd093a`，`git ls-remote` 复核该 SHA 对应 `refs/tags/v1.1.0`；禁止因 CRT 漂移源码。首次原生运行前另冻结 harness/dirty diff/lock/toolchain/features/target/链接策略与制品哈希。 |
| M4 ADR 研究链接错误 | 关闭 | ADR 链接改为 `../../../research-vercel-labs-agent-desktop-yonder-cua/...`，当前可解析到实际报告。 |

补充确认：派发、抢占、取消现共享唯一 admission gate 与 stdio flush 线性化点；Worker containment、只读清理与 global desktop quarantine 已成为明确规则；证据 schema、target identity oracle 与 predicate 也已跨平台冻结，未发现新的父架构冲突或第二状态所有者。
