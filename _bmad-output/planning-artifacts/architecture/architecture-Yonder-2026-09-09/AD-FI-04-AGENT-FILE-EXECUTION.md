# AD-FI-04 受授权引用的 Agent 文件执行

状态：Accepted（macOS 首批受控执行；Windows 延期）  
日期：2026-09-26  
Architecture Impact：architecture-change（协议新增文件执行方法；复用 TM-S7 启动事务，不新增持久化或第二执行栈）

## 问题

AD-FI-02 与 AD-FI-03 已让本机用户为任务签发不可伪造的临时文件引用，并让归属 Agent 读取安全摘要，但尚没有受监管的执行入口。让 Agent 请求携带路径、授权根、`confirmed` 或对授权事实的复制，会绕过已建立的身份、锁和确认边界；直接由 Tauri 或 Adapter 处理 Agent 请求也会绕过 TM-S7 的任务启动、事件和 Outbox 语义。

## 决策

协议 1.28 新增 `file.execute` 能力和 `task.file.execute` 方法。请求只能携带 `task_id`、期望序号、`grant_id`、与授权一致的操作，以及有界 Base64 写入正文；不接受路径、授权根、文件身份、哈希、确认字段或任意命令。读取、创建、替换、回收站分别严格对应 `read`、`create-new`、`replace`、`trash` 授权。读取结果可含有界 Base64 正文与内容哈希；写入/回收站结果只返回摘要，所有路径及完整正文不得进入日志、事件、Outbox、UI 或错误消息。

首批传输上限为协议请求的 64 KiB：写入正文与读取返回正文均以 48 KiB 原始字节为硬上限，超限明确失败，不分片、不截断、不暗中改用另一条传输通道。FI Runtime 的 16 MiB 本地上限不等同于 Agent Gateway 传输承诺；大文件与附件引用路线须由后续独立 ADR 定案。

Gateway 在认证会话、登记状态、任务归属、协议版本、`file.execute` 能力与期望序号均通过后，调用 Application 用例。用例先按 TM-S7 在同一事务中登记 `created→running`、步骤、attempt、事件和 Outbox，随后解析授权；一次性授权在解析时原子消费。受控 File Port 仍在读取/提交/回收站时复核身份、哈希、目标竞争、租约与宿主锁。副作用完成后仅形成可观察的摘要结果；unknown、超时、崩溃或断连不自动重试，由归属 Agent 经同一 Gateway Observe 后决定下一步。

`trash` 仍要求 AD-FI-03 的本机原生二次确认，授权引用不是风险确认的替代物。创建/替换只接受 Agent 提交的有界字节，不开放 XML、shell、目录枚举、永久删除、批量操作或路径猜测。TaskHost 是唯一持有 File Port 与 Registry 的组合根；Application 不依赖具体 Adapter，Adapter 不访问任务/会话/授权状态。

## 后果与验证

- 任务状态、事件和 Outbox 继续以 SQLite 同一事务为事实源；授权不持久化，也不写入任务事件。
- 读取授权可在期限内重复使用；创建、替换与回收站在首次解析后不可重放，即使后续副作用结果为 unknown。
- Agent 禁用、撤权、任务终态、到期、会话断连或平台不可用均失败关闭；Windows 保持 unavailable。
- 验证必须覆盖协议无路径约束、Base64 和体积门禁、Agent/任务/用途隔离、启动事务、一次性消费、TOCTOU/锁冲突、unknown 不重试、撤权及 macOS TaskHost/Gateway 结构化证据。Windows 证据按用户决定延期且不得计为通过。
