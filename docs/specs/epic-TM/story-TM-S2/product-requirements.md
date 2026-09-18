# TM-S2 产品需求

## 问题与目标

不同资源任务有界并行；桌面/浏览器单并发、同文件互斥，开始与结束不影响其他任务。

## 范围与非目标

本 Story 仅负责“资源准入与执行生命周期”。核心准入/start/finish 已存在旧 Change；真实执行器、文件身份与派发前检查尚待设计接线。

## 验收条件

- RESULT-01：动作结论与后置 Observe 分类必须绑定当前完整 attempt 身份，并与任务 sequence、事件、Outbox 同事务提交。
- RESULT-02：相同结论重投幂等，不同结论或旧身份拒绝；故障不得留下部分结果。
- RESULT-03：Agent 可通过协议 1.5 增量事件读取结果；旧协议响应不包含新字段。
- RESULT-04：unknown 不自动重试、不释放执行占用；结果记录不包含输入、窗口树、截图或完整 Payload。

- 目标行为有可复现成功样本，失败不得伪报成功。
- 不同资源任务有界并行；桌面/浏览器单并发、同文件互斥，开始与结束不影响其他任务。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。

## 待决事项

核心准入/start/finish 已存在旧 Change；真实执行器、文件身份与派发前检查尚待设计接线。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：Task Space 与权限模型、MVP 主干链路；补充材料 CUA 与 BUA 的 Task Space。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：任务、状态与恢复。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。
- 后续用户变更：支持多任务并行、所有任务统一展示，已记录 AD-OCT-03；仍遵守 CUA/BUA/同文件资源约束。

## 连续步骤边界增量

来源：产品简报「MVP主干链路」Agent逐步执行与状态回报、补充材料「执行原则」每步动作后Observe，以及用户要求接入CUA/BUA能力。

- STEP-NEXT-01：动作结果已知且后置Observe有效时，可结束当前attempt并保持任务running；prepared/unknown或pending控制不得推进。
- STEP-NEXT-02：普通边界、任务sequence、事件与Outbox同事务；不释放任务级资源占用。
- STEP-NEXT-03：边界成功后Agent可声明下一步骤，Supervisor以同一Permit准备新attempt；旧身份和旧sequence拒绝。
- STEP-NEXT-04：控制停止语义保持不变，普通推进不得伪装暂停、取消或接管。

## 执行尝试准备子范围

来源：产品简报「MVP主干链路」启动执行与可审计状态、补充材料「执行原则」每步Observe、架构步骤/恢复，以及用户多任务并行与接管变更；技术字段采用Accepted AD-TM-08。

- ATTEMPT-01：准入后只把已接受的当前step_id绑定到Application生成的attempt/Worker/宿主身份，请求ID和Agent字段不能代替。
- ATTEMPT-02：任务created→running、Start事件、Outbox和prepared尝试同事务；任一失败均无部分成功。
- ATTEMPT-03：同一完整身份重试不新增事件；不同身份、旧序号、无步骤、非created或已有活动尝试明确拒绝。
- ATTEMPT-04：首批只证明prepared，不派发动作、不声明Observe或安全停止；Windows和真实Driver证据仍保留。

## 已观察失败终结增量（2026-09-18）

来源：产品简报任务 `failed` 状态、补充材料每步动作后 Observe、架构 unknown 不得自动重试，以及 DS-S1 真实失败反馈缺口。

- FAIL-01：归属 Agent 只可在最新 attempt 已 Observe 为 `action_succeeded=false` 且已推进 stopped 边界后提交失败终态。
- FAIL-02：unknown、动作成功、prepared/observed 未停止、pending control、旧序号与越权请求拒绝，任务和资源占用不变。
- FAIL-03：`failed`、事件与 Outbox 复用现有 SQLite 事务原子提交，提交成功后才释放 Desktop 资源。
- FAIL-04：失败依据引用既有 attempt/step 事实，不接受自由文本、截图或完整 Payload；读取历史不重播桌宠失败动画。
