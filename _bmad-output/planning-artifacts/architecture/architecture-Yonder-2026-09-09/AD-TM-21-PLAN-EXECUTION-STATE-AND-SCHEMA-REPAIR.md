# AD-TM-21 计划执行状态与缺失审计表安全修复

状态：Accepted（2026-09-28，用户确认计划执行时 Yonder 必须同步显示执行中）。关联 TM-S8、EX-S2、TM-S5。Architecture Impact：architecture-change（任务状态与持久化启动修复）；不改变状态所有者、协议版本、数据加密边界或 Agent Gateway。

## 背景

真实 Codex 慢脑经产品 MCP 提交多候选计划后，`task.plan.execute` 会先展示 CUA 控制条。若 Jev 在第一个 Driver 动作前交回，既有实现只写入 `next_intent`，任务仍为 `created`，造成浮窗“正在控制”与 Task Space 非执行中相互矛盾。正式用户数据库还暴露出另一类历史中断状态：`user_version=20`，但 schema 18 的三张审计业务表全部缺失；任务写入可继续，而 `task.get/task.events` 因读取审计投影失败。

## 决定

- `task.plan.execute` 被接受后，同一任务已进入产品执行闭环。若 Jev 在首个动作前交回，交回事务必须把 `created` 原子转为 `running`，并与新 sequence、状态事件、Outbox、`next_intent` 投影一起提交；已经 `running` 时保持 `running→running`。任务只有终态、显式暂停/接管、中断或等待用户时才离开执行中。
- 控制条和 Task Space 只读取同一任务事实；不得用浮窗本地布尔值伪造执行状态。交回表示慢脑接续同一运行任务，不创建第二任务或第二状态机。
- 启动时检查 schema 18 的 `task_artifact_manifests`、`task_artifact_manifest_items`、`task_user_confirmations`。三表全部存在时不变；三表全部缺失时，可执行仅建空表的确定性修复，因为不存在可被覆盖的该组业务记录；只缺部分表时拒绝启动并保留数据库，禁止猜测或补造审计数据。
- 修复使用原始表约束，不改现有任务、事件、Outbox、附件、确认或 `user_version`。不得自动修改已有加密库。

## 验证

覆盖：首动作前 Jev 交回的 `created→running` 原子事务、既有 running 交回、事件/Outbox 连续性、三表全缺安全修复、三表部分缺失拒绝、正常 v20 零变更。macOS 再以正式 `Yonda.app`、Codex MCP 和真实多候选 Jev 片段验证 Task Space、`task.get`、`task.events` 一致；Windows 继续暂缓。
