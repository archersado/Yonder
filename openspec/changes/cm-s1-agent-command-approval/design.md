# 设计

协议以 Rust 为唯一来源新增 `task.command.propose` 与 `task.command.execute`。提议携带有界结构化命令，仅返回 `command_id` 与 `awaiting_user`；执行携带任务、CAS sequence 和 `command_id`，没有路径、参数、环境、Shell 或确认字段。

TaskHost 持有 `CommandApprovalRegistry`，上限 32 项、最长五分钟。Registry 保存完整提议但不写 SQLite；本机 UI 获取预览、批准或拒绝。批准绑定任务、归属 Agent、当前 sequence 与 canonical command SHA-256；执行时原子消费。任何身份、sequence、摘要、终态、撤权、断连、过期或重启变化均拒绝。

Application 执行用例先调用 TM-S7 `start_execution`，再消费批准并调用 `CommandPort`；确定结果映射 observed，无法确认进程组停止映射 unknown，均不自动重试。响应、事件、Outbox 和日志只带安全结果摘要，完整命令只存在本机内存预览。
