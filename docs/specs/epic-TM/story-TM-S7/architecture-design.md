# TM-S7 架构设计

## 状态所有权与边界

Task、sequence、events、Outbox 与准入仍由 TM/Application 拥有。所有能力 Adapter 只实现派发和 Observe Port；不得自行将 UI、文件写入或进程启动解释为 `running`。CUA 与 BUA 现有首次 `start_attempt` 行为迁入共享 Application 用例；Document 与 Command 在其各自门禁满足后只能调用该用例。

## 统一启动事务

`start_execution(task_id, expected_sequence, step, attempt, resources)` 仅接受 `created` 或持有相应准入的 `running` 任务：

1. 对 `created`，在同一 SQLite 事务完成状态 `created→running`、步骤声明、prepared attempt、sequence 递增与 Outbox；事务提交前不触发副作用。
2. 对 `running`，只允许在已 Observe 且已推进的边界准备下一 attempt；不得重新 Start 或取得第二套状态。
3. 事务成功后由能力 Adapter 派发；每次派发后必须 Observe 并以同一任务事实写入 observed/unknown。停止、完成与失败复用 TM-S2/TM-S3 既有确认流程。

若 EX-S2 引入计划片段，`start_execution` 只接受由 AG-S1 Gateway 已验证的 `plan_id`/`plan_version`，并在同一启动事务中持久化计划事实；字段与事务落点未定稿前不修改现有用例。

资源由能力声明：Desktop（CUA）、Browser（BUA）、DocumentWrite（同文件互斥，依赖 FI-S1）、CommandProcess（受监管进程树）。资源细节不由本 Story 复制；新增资源种类、协议或持久化前先更新 AD-TM-13。

## 边界与依赖

Application 是唯一启动协调者，SQLite Store 是唯一任务事实源。CUA/BUA Adapter 只经共享 `start_execution` 传递能力资源和副作用 Port，不互调、不自行迁移状态；Document/Command 必须等待 FI-S1/CM-S1 与协议门禁。

## Gateway 与可观察性

Gateway 在执行请求受理后只返回已提交的任务快照；UI 仅消费该快照及事件。Document/Command 尚未具备正式 Gateway 契约时返回 `capability_unavailable`，不得先走通用 shell 或直接文件写入。日志不记录正文、截图、命令完整输出或完整 Agent Payload。

## 状态与契约

`created→running`、步骤声明、prepared attempt、sequence、事件与 Outbox 仍由现有 SQLite 事务承载；新增资源枚举必须从 Rust 唯一模型派生。`running→下一 attempt` 继续校验已 Observe/Stopped 边界、同一任务准入和 CAS sequence。

## 失败与验证

启动事务失败不派发；准入失败、缺少占用、非边界运行、断连和 Observe 失败分别返回可区分错误。Application/SQLite 回归覆盖 created/running 双分支、Outbox 原子性和失败回滚；真实 Document/Command 与双平台证据待门禁。

## 验证

Application/SQLite 集成验证四类能力的首次启动均先产生 `running` 事实，再允许可控副作用；故障注入验证无半提交。CUA/BUA 复用现有 Windows/macOS 原生证据；Document 与 Command 在各自 Spike、FI-S1、协议和双平台证据完成后补验证。独立 Verification Goal 不得在这些前置完成前 Archive。
