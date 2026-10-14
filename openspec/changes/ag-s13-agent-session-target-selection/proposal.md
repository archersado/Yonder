# Proposal：AG-S13 Agent 输入会话目标选择

关联 [AG-S13](../../../docs/specs/epic-AG/story-AG-S13/README.md) 与 Accepted AD-AG-11 增量。Architecture Impact：conforming；扩展 `AgentInputHub` 既有运行时所有权和 Desktop 内部 UI 命令，不改变 Gateway 协议、SQLite、Outbox 或任务状态。

## Why

多个在线 AgentSession 时，Yonder 正确拒绝猜测输入目标，却没有可完成选择的产品入口，导致圈选、语音和 VI-S3 回复验证被永久阻断。

## What Changes

- Hub 暴露不含 session/thread ID 的在线会话摘要与显式选择用例。
- 单会话自动选择；多会话按自动/显式状态稳定处理连接和断连。
- Agent 管理增加在线输入会话区块、当前目标和稳定错误反馈。
- 覆盖 Hub、Desktop/UI 与 macOS 正式多会话验证。

## Non-goals

不建立会话历史、不持久化选择、不修改 Agent 协议、不自动关闭其他会话、不改变注册授权。
