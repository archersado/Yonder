# Proposal：AG-S13 Agent 输入会话单活抢占

关联 [AG-S13](../../../docs/specs/epic-AG/story-AG-S13/README.md) 与 Accepted AD-AG-11 的 2026-10-15 增量。Architecture Impact：conforming；收敛 `AgentInputHub` 既有运行时所有权，不改变 Gateway 协议、SQLite、Outbox 或任务状态。

## Why

允许多个输入会话并要求用户选择目标，暴露了 Codex thread 内部复杂度，也使正常输入多出不可理解的阻断。主人明确本地输入连接应为单活、后连抢占前连。

## What Changes

- Hub 从多会话与显式目标收敛为单一当前 AgentSession。
- 新连接原子替换并关闭旧连接；旧 unregister 不得清除新连接。
- 删除 Agent 管理会话列表、选择命令及圈选卡选择入口。
- 覆盖抢占、回复中断、唯一投递和 macOS 正式语音回复验证。

## Non-goals

不建立会话历史或人工切换，不持久化路由，不修改 Agent wire 协议，不改变任务 Gateway 的连接与归属语义。
