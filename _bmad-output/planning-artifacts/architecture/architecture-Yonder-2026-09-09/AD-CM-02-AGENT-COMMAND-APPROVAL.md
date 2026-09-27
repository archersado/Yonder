# AD-CM-02 Agent 结构化命令的本机批准

状态：Accepted（macOS 首批；Windows 延期）  
日期：2026-09-27  
关联：CM-S1、AD-CM-01、AD-TM-13

## 决策问题

AD-CM-01 只接受内部 Command Runtime，不能让 Agent、CLI/MCP 或 Jev 直接请求命令，因为结构化参数仍可能造成删除、安装、提权、支付或发送等副作用。单个 `confirmed: true`、Agent 声明、可重放的批准码或只按任务绑定的确认都会被替换参数、重放或跨 Agent 使用绕过。

## 决策

首批 macOS Agent Command 采用“提议—本机批准—一次执行”三段式内存契约，所有 Agent 发起的结构化命令均先获得本机用户明确批准；不试图以启发式判定低风险命令。

1. 归属 Agent 用 `task.command.propose` 提交合法的 `program + args + cwd + env + timeout`。Gateway 验证会话、登记、归属、任务 `created` 状态和请求边界后，TaskHost 将完整提议仅存入有界内存 `CommandApprovalRegistry`，生成不透明 `command_id`。提议不启动任务、不创建 attempt、不写 SQLite、事件、Outbox 或日志。
2. Task Space 只能通过本机组合根读取同一 `command_id` 的本地预览，展示完整可执行文件、字面参数、工作目录、显式环境键和值、超时和“所有 Agent 命令均须批准”的风险说明。该预览不经 Gateway、CLI/MCP、日志或任务事实返回。
3. 只有当前本机用户在 Task Space 的显式“批准执行”动作能使提议成为可执行批准；拒绝、关闭、任务终态、Agent 撤权/断连、到期、重启或容量逐出均使其失效。批准绑定 task、owner Agent、完整命令 SHA-256 摘要、当前任务 sequence 与单一 `command_id`，最长 5 分钟且仅消费一次。
4. 归属 Agent 用 `task.command.execute` 只引用 `task_id`、`expected_sequence` 和 `command_id`。Gateway 不接受 `program`、参数、路径、环境、Shell 字符串或确认布尔值。Application 先由 TM-S7 `start_execution` 同事务提交 `created→running`、步骤、attempt、事件和 Outbox，随后原子消费批准并调用 AD-CM-01 Runtime。副作用结果按 observed/unknown 写既有 attempt 事实；unknown 不自动重试。

批准不是持久化审计确认，也不替代文件授权或删除的原生二次确认。Shell、提权、安装、删除、支付、发送、批量操作、Windows Runtime、Jev 候选与云端 AgentSession 不在本 ADR 范围。批准 UI 不应暴露完整命令到桌宠、任务事件或远端 Agent。

## 后果与验证

- Registry 只在 desktop TaskHost 内存中保存，最多 32 个待定/已批准条目，每项完整 UTF-8 负载最大 48 KiB；满额时拒绝新提议，不驱逐已批准条目。重启后全部失效，不能恢复或重放。
- 提议与执行均需 Rust 协议唯一来源的版本协商；Windows 不注册能力。
- Gateway 只能向当前认证且仍登记的 owner Agent 返回无正文状态和不透明 ID；跨任务/Agent、过期、拒绝、取消、序号变化、摘要不符、重复消费均失败关闭。
- 验证覆盖：无路径/无确认字段的协议、批准前零副作用、预览与批准只能本机进入、参数替换/重放/跨归属拒绝、启动事务、输出边界、unknown 不重试、撤权/断连/重启清空及 macOS 原生批准/拒绝证据。Windows 证据按用户决定延期，不能据此标记完整 Story 完成。
