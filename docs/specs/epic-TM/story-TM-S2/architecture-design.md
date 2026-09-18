# TM-S2 架构设计

## 边界与依赖

复用 AD-OCT-06；先准入后状态提交再派发，确认停止后终态提交再释放；占用不是任务事实源。

## 状态与契约

schema8 在现有 task_attempts 增加 `observed/unknown` 结果阶段、result_sequence、动作成功分类和 unknown 原因。Application 从事实源读取当前 attempt 后提交 Driver 结果；Adapter 在单事务内 CAS running 任务、追加 `running→running` 事件/Outbox并更新 attempt。协议 1.5 仅为事件增加可选 attempt_result，旧版本剥离该字段。

与 TM-S1/S5 的联合设计要求：宿主为每次明确的新执行生成并持久化 attempt_id，绑定已登记 step_id 与执行器后才派发；客户端重投递不创建新 attempt。单任务当前设计仅一个未确认停止的动作尝试，跨任务仍按资源并行。资源准入成功不是永久执行授权，派发前仍检查控制状态、当前权限和 deadline。可信结果归原 attempt，不能借迟到结果覆盖新步骤；观察子操作字段和持久化契约仍待 ADR 定稿。

SQLCipher 保持任务当前事实源；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

## 失败与验证

核心准入/start/finish 已存在旧 Change；真实执行器、文件身份与派发前检查尚待设计接线。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本文件为规划迁移，运行时无变化；具体实施按关联 ADR 和 Proposal 声明影响，draft/design-review 不授权实施。

## 连续步骤边界设计

Accepted AD-TM-08授权schema11把observed attempt原子推进为普通stopped，并以running→running事件保留审计。running步骤声明仅在最新attempt普通停止且无pending控制时开放；prepare next复用同一任务Permit，不重新竞争资源。Application/Adapter仍沿既有Port依赖，不新增协议或第二状态源。

## 派发与控制排序点联审

执行身份/派发/停止契约统一见[AD-TM-08](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-08-EXECUTION-IDENTITY-AND-STOP.md)（Accepted，当前仅授权尝试准备）。准入后必须持久化当前尝试，派发与冻结共享排序点，不能只检查一次权限后延迟无条件交付。旧attempt/Worker回调不完成新步骤或释放新租约；未知副作用不重投递，停止结果提交前保留占用。现有start/finish核心不等同真实执行契约。

## 执行尝试准备定稿

Accepted AD-TM-08首批不改外部协议。Application新增`ExecutionAttempt`与TaskStore的prepare/get Port；Adapter schema7在IMMEDIATE事务中读取当前任务与最新步骤，提交created→running、事件、Outbox和prepared尝试。active部分唯一索引提供数据库级单任务尝试约束。Application生成的三个身份均使用既有受限ID规则；Agent步骤声明只提供step_id。

Admission新增对应start入口：先取得资源Permit，再调用prepare Port；失败只释放本次从未派发的Permit。成功返回的Permit仍不得丢弃或提前释放。真实Driver派发、回调、停止与定位均不属于本Change。

## 已观察失败终结定稿（2026-09-18）

按 Accepted AD-TM-11，协议 1.18 增加 `task.fail`。Gateway 只调用 Application；Application 复核归属、CAS、running、Desktop Permit、无 pending control，以及最新 stopped attempt 的既有结果为 `Observed { action_succeeded:false }`。随后复用 `Action::Fail` 和当前 Store `commit` 原子写任务、事件、Outbox，成功后释放 Permit。SQLite schema 不变；不新增错误正文或第二结果模型。desktop 统一终态响应判定将成功 `task.fail` 映射为一次性 failed 动画。

共享终结门禁同时要求 `task.complete` 的最新结论为 `Observed { action_succeeded:true }`，避免同一失败事实被提交成 completed；请求与响应终态不一致时桌宠不展示动画。
