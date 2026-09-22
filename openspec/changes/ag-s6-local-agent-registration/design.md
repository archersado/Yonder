# 设计

本地 Agent 注册与撤权只作为 MVP 之后能力，不并入 AG-S2 当前实现。

注册信息持久化在 SQLite，与任务状态同库。撤权写操作与任务状态更新保持同一事务边界；撤权后该 Agent 不能新建任务、提交步骤、控制任务或接收输入，但已有任务保持可见，用户仍可查看和终止。

本设计不新增传输认证机制，不改变现有 Gateway 协议，也不引入本地凭据存储。实施前仍需独立 Story、OpenSpec 与 Verification Goal，不能把本提案当作已完成实现。

## Application/Gateway 接入

Application 应新增独立的 `AgentRegistry` Port，不能把撤权状态散落在 `GatewaySession` 或 Adapter 中。Gateway 在会话建立和每个写操作前先读取当前 Agent 状态；撤权后返回稳定的权限拒绝，不自动重试副作用。

SQLite 侧新增 `agent_registry` 表，与任务状态同库同事务。撤权写操作必须与任务状态更新在同一事务边界内完成，不能出现“已撤权但仍能新建任务”的中间态。

Desktop Agent 管理界面只消费 Application 返回的注册与撤权结果，不直接访问 SQLite 或 GatewaySession。
