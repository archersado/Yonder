# EX-S2 架构设计

## 边界与依赖

AG-S1 的慢脑接入边界已由 Accepted [AD-AG-07](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-07-SLOW-BRAIN-GATEWAY-INGRESS.md) 固定；以下计划字段与事务仍须本 Story 联审后才能实施。

本子范围的配置界面已由 Accepted [AD-EX-03](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-EX-03-JEV-CONFIG-INTERFACE.md) 定案；它只交付非敏感配置的读取、校验和保存，不授权 Jev 执行。

当前实施基线保持 AD-EX-01 的“每步 Jev 决策”。[AD-EX-04](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-EX-04-BOUNDED-ACTION-RECIPE.md) 保持 Proposed，仅作为后续对照观察路线；在 Accepted 前不进入 EX-S2 实施、协议、迁移或运行时代码，且须计划片段契约修订后才能另行创建对照 OpenSpec。

2026-09-24 用户变更：先实施 macOS-only 远端 Jev 决策接线，Windows 后补。Application 拥有有界候选请求、置信阈值和交回策略；macOS Adapter 从系统凭据入口读取 API Key，经固定 `@typesafe-ai/sdk@0.6.0` 调用 `https://api.typesafe.ai/v1/systemone`，模型固定 `jev-latest`。API Key 不进入 Jev 配置、SQLite、任务数据、日志或 UI；Windows 路径不编译、不注册、不验证。

## 跨 Story 联审前置

本 Story 的设计不重排既有 Epic：AG-S1 的传输与身份、AG-S3 的归属 Agent 步骤声明、TM-S2/TM-S7 的任务事实、CU/BU 的模型无关 Driver 与 Bridge、DO/FI/CM 的文件锁与命令围栏都先保持现状。EX-S2 定稿计划片段、版本/CAS、内部快脑步骤来源、事件/Outbox 交回和可区分错误矩阵。AD-AG-09 已授权 macOS CUA 的受限单槽位消费基线；TM-S7 的统一启动门禁继续适用，其他 Driver、Recipe 与 Windows 不进入本次实施。

慢脑从现有 Agent Gateway 提交首次计划与 replan；本地仍经 MCP stdio/CLI 与 Local Socket，云端仍经单一出站 WSS，两者汇入同一 Application 用例。计划至少绑定 `task_id`、归属 `agent_id`、`request_id`、计划版本、允许能力/资源范围、目标完成判据、截止时间及决策/动作预算。字段以 EX-S1 和 TM-S7 联审为准，最终仅在 Rust 协议类型定义并派生 Schema/TS。Yonder Application 复用统一执行入口，定义一次「从当前 Observe 枚举候选→决策→授权/准入→派发→Observe→提交结果」用例及一个快脑决策 Port。Jev 只由 Adapters 实现此 Port；Adapter 不调用另一 Adapter。快脑需要 replan 时，将有界 Observe/失败依据经既有事件与 Outbox 交给归属 Agent，新计划仍须由 Gateway 进入；不直连慢脑或创建第二会话。

与 AG-S1 的联审候选保持同一契约基线：Gateway 先校验可信会话、身份一致、任务归属、能力、`expected_sequence`、计划版本和 `deadline`，再交给 EX-S2 的计划用例。`request_id` 用于幂等；重复请求在未启动前返回原结果，启动后返回进行中或明确冲突。EX-S2 不新增第二套身份、连接或状态源。

当前 AG-S3 `task.step.declare` 只接受归属 Agent，CU-S2 `computer.step` 也以 Agent 调用推进。快脑连续执行需由 Application 在已验证计划内使用独立可信内部来源写下一步骤/attempt，事件同时保留计划版本、归属 Agent 和实际决策来源；不能复用 Gateway 的 Agent 身份来伪造请求。新增来源及其幂等、事务和冻结排序先与 AG-S3、TM-S2、TM-S7 联审并更新对应 ADR/OpenSpec。片段完成只产生待验证结果/交回事件；现有 `task.complete`/`task.fail` 仍由归属 Agent 经 Gateway 提交。

Jev 配置由 Application 拥有并通过既有桌面命令提交；独立 Jev 设置窗口只渲染和收集，不直接读写 Adapter、配置文件或模型端点。非敏感配置包含启用状态、AD-EX-02 定稿的服务形态与端点、全局步数/时间/token 上限、CUA/BUA/Document/Command 能力上限；实际片段取计划授权值与全局上限的较小值。配置在片段开始时形成快照；片段运行中修改不改变已冻结片段，但关闭 Jev 等同用户控制，必须立即冻结下一次决策并交回慢脑。无效配置拒绝保存/启用并给出字段级错误。API Key、证书与刷新令牌不属于本配置模型；API Key 仅允许设置面板以非空密码框提交给 macOS Keychain Adapter，读取接口只回传布尔状态。UI 与日志不得展示或持久化完整端点查询串、密钥和模型请求正文。Task Space 不承载配置交互。

macOS-only 接线不新增任务状态。Application 校验配置、能力范围和 2..10 个有界候选，且必须包含唯一 `handback` 候选；Jev 只能返回已提交候选 ID 和置信度。低置信、未知候选、不可派发候选或缺少发送确认一律交回，不自动重试。交回与任务序列、事件、Outbox 及受限 `next_intent` 同一事务提交；归属慢脑消费 Outbox 后经既有 Gateway 重新 Observe、规划或执行，Yonder 不作本地兜底。模型失败、凭据缺失、依赖缺失或超时仅返回可区分错误，仍由未来片段执行用例统一映射到既有状态与事件。API Key 从设置页密码框一次性进入 macOS Keychain Adapter；状态查询与写入不依赖 `TaskHost` 或 SQLite，UI 只能查询是否存在，禁止经 `security` CLI 参数、环境变量或临时文件传输。

配置字段定稿为：

| 字段 | 类型与范围 | 说明 |
|---|---|---|
| `enabled` | bool | 是否允许新片段使用 Jev。 |
| `service_mode` | enum：`local` / `remote` | 服务形态，首版只开放 AD-EX-02 定稿的取值。 |
| `endpoint` | URL | SDK base URL；macOS-only 远端固定为 `https://api.typesafe.ai`，SDK 再调用 `/v1/systemone`；不保存查询串、fragment、用户名或密码。 |
| `step_limit` | integer，`1..100` | 每片段最大步数。 |
| `time_limit_ms` | integer，`1000..600000` | 每片段最大执行时长。 |
| `token_limit` | integer，`1..100000` | 每片段最大模型 token 数。 |
| `capabilities` | array，仅可含 `CUA` / `BUA` / `DOCUMENT` / `COMMAND` | 全局允许的执行层上限。 |

非敏感配置持久化为 SQLite `jev_config` 单行 JSON，由 Application 在同一事务内校验并原子写入；不写任务、事件或 Outbox。保存成功仅表示配置已通过结构校验，不代表模型可用，运行时仍须在派发前检查当前配置和外部服务可用性。

## 状态与契约

2026-09-28 用户变更将 CUA 候选从“工具名与参数”扩展为受限语义声明：`action_kind`、不含敏感值的 `target_ref`、`preconditions`、`expected_observe` 与可选 `confirmation_ref`。Rust 协议仍是唯一来源；Jev 请求只获得动作语义、当前 Observe 的有界摘要及预期条件，不获得接收人、正文、截图或 Driver 参数。Application 在选择前复核前置条件，在选择后解析一次性本地确认并交给既有 CUA Driver；确认不存在、过期、取消或 Observe 不符均交回归属 Agent。发送等敏感副作用不得把确认值、接收人或正文写入片段 JSON、任务事件、Outbox、日志或模型请求。

SQLite 当前任务状态仍是事实源；已有 step/attempt、sequence、事件、Outbox 事务链记录最小决策来源（计划版本/候选编号/结果分类）和动作结果，不写模型输入或截图。快脑持有的 Observe/目标引用只在当前执行身份和有效期内使用。派发前复核 `task/step/attempt/host`、计划版本、授权、租约、目标新鲜度及用户控制标记。动作结束强制 Observe；目标达成须独立验证，不能只信 Jev `DONE`。偏离、低置信、无候选、参数不足、预算耗尽转为「待慢脑 replan」的可观察原因，具体状态映射复用 TM，不增第二状态机。

取消、接管、用户输入先冻结快脑新决策，再按既有步骤边界停止/Observe/事务确认。副作用超时、崩溃或断连为 `unknown`，保留占用并交回慢脑；重启 running→interrupted，不能恢复旧候选、旧计划或自动重发。敏感操作继续走显式用户确认，不因高置信豁免。

## 失败与验证

macOS-only 子路线验证 Application 候选/置信策略、Keychain 凭据边界、SDK Worker、组合根和 Windows 不注册。计划片段执行则另行验证 Domain、协议、SQLite/Outbox/Gateway 以及 Windows/macOS 原生取消、接管、权限、网络断连与恢复；它仍以 EX-S1、TM-S7 与完整执行层门禁为前置。
