# TM-S4 产品需求

## 问题与目标

用户明确归还后重新 Observe，外部 Agent 决定下一步，不重放已确认动作。

## 范围与非目标

本 Story 仅负责“显式继续与恢复”。暂停接管闭环未验证，不可实现自动恢复。

## 验收条件

- 目标行为有可复现成功样本，失败不得伪报成功。
- 用户明确归还后重新 Observe，外部 Agent 决定下一步，不重放已确认动作。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。

## 待决事项

暂停接管闭环、可信用户输入关联与可确认的 Agent 当前会话投递尚未完成，不可实现自动恢复。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：Task Space 与权限模型、MVP 主干链路；补充材料 CUA 与 BUA 的 Task Space。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：任务、状态与恢复。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。

## 人工接管记录与交回（2026-09-14用户变更）

来源：本次用户明确执行中支持人工接管并记录用户行为，作为交回Agent的Observe依据；架构Recording与桌宠/任务恢复约束，Accepted AD-TM-03。仅Agent创建任务，人工接管不创建新任务。

验收映射：TAKE-01显式接管阻止Agent新动作，停止未确认不称已移交；TAKE-02本次手动接管开启可见、可停止的记录，只录user输入，密码/安全界面/排除应用不采；TAKE-03原始时间线不可变并关联原task_id；TAKE-04交回保存证据并执行新鲜Observe，向归属Agent交付轨迹/证据引用及当前状态；TAKE-05交接失败/配额缺口明确反馈、保持暂停，不自动续跑；TAKE-06Agent重新Observe并显式恢复，重新准入后才executing。

TM-S3管接管及停止确认，RC-S1管用户记录/不可变证据，TM-S5管任务时间线引用，TM-S4管交回及显式恢复，AG管归属Agent交接协议，DS-S2展示已有任务接管/记录中/交回。自动输入干预只暂停，不无提示开启Recording；显式接管作为用户手动开始。已有每设备Recording时明确冲突，不覆盖。

UI流程：任务面板“接管”→“正在停止Agent控制/记录中”→停止确认后人工控制；小龙按既有暂停表现并保留独立录制提示。用户可以停止记录，任务仍保持人工接管；交回时如果停止后有未记录操作标注证据缺口，最终Observe仍必须新采集。“交回Agent”不等于立即继续，不用点击桌宠自动恢复任务。

协议/持久化/Driver停止与授权/隐私字段未定稿，相关Story保持设计阶段，不提前实现采集或向外发送用户记录。Windows暂缓、真实双平台证据保留。

## 多任务输入关联（2026-09-18 设计建议）

来源：产品简报“遇到歧义时暂停并请求用户确认”、用户要求用户输入随连接 Agent 的通道投递、AD-VI-02；关联方式是为满足多任务安全而提出的架构设计建议，尚非已实施能力。

- RESUME-INPUT-01：同一 Agent 有多个等待任务时，用户必须从具体任务详情或该任务明确请求入口选择回复对象；裸语音、最近任务或当前桌面均不能推断关联。
- RESUME-INPUT-02：输入开始前冻结任务、归属 Agent、当前会话和任务序号；投递期间任一项变化，关联失效且任务保持原状态。
- RESUME-INPUT-03：会话输入可被 Agent 接收不等于恢复。Agent 必须在新鲜 Observe 后显式请求恢复，Yonder 不重放旧动作或替 Agent 规划。
- RESUME-INPUT-04：任务记录仅保存无正文的关联与投递结果；输入正文、截图和完整 Agent Payload 不写任务库、事件、Outbox 或日志。
- RESUME-INPUT-05：RC-S1 来源、AG-S5 当前会话确认或 Recording 证据任一门禁不满足时，任务卡片明确显示不可交回，不自动退化为“最近任务”。
