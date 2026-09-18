# AG-S3 产品需求

## 问题与目标

Agent需要给任务声明当前准备执行的步骤，用户与Agent能够区分声明、实际执行及结果。登记后保留步骤历史，不因取消或重试丢失。

## 范围与非目标

原始需求完整步骤/动作/Observe链路归AG/TM/CU联合推进。首批只接受所属created任务的纯文本步骤声明，支持幂等及当前声明/历史读取；不人工创建步骤、不执行标签、不制造running、attempt或窗口引用。不实现真实动作、停止、接管记录或自动恢复。

## 验收条件

- STEP-01：已握手归属Agent可声明created步骤；状态不变、sequence递增，当前声明和历史可读取。
- STEP-02：重复step_id同标签返回既有事实，异标签拒绝；取消后重试不复活，不增加声明/事件/Outbox。
- STEP-03：未握手、旧版、过期、伪造/越权、LocalUser写入及非法文本全部拒绝且不写库。
- STEP-04：并发、CAS、事件/Outbox故障原子回滚；配额满拒绝新声明，旧记录不清理。
- STEP-05：1.4事件明确标为步骤声明，旧会话无新增字段且序号不缺；读取当前快照与声明一致。
- STEP-06：明文升级前备份、旧任务无伪造声明；危险/未知格式、加密旧库及失败迁移保留数据。

## 需求来源

原始需求：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)「MVP 主干链路」6～9、「Task Space 与权限模型」→STEP-01/02/05；[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)「执行原则」要求声明与执行/验证分开→STEP-01/05。

架构约束：[ARCHITECTURE-SPINE](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)「Agent Gateway」「任务、状态与恢复」→STEP-03/04/05；AD-ST-01及迁移安全→STEP-06。后续用户“任务通过Agent创建、轻量hover menu、不人工创建”→STEP-03及无人工步骤入口。具体字段、协议版本和配额为[AD-AG-04](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-AG-04-AGENT-STEP-DECLARATION.md)技术选择，不称为原产品原文；真实动作/Observe仍待后续实现，不由代码现状删减。
