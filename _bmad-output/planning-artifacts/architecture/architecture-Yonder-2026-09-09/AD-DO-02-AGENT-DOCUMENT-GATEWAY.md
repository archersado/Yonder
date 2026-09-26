# AD-DO-02 Agent 文档 Gateway 与双授权另存

状态：Accepted（macOS 首批；Windows 延期）  
日期：2026-09-26  
Architecture Impact：architecture-change（协议新增 Document 执行方法；复用 FI 授权与 TM-S7，不新增持久化或文件执行栈）

## 决策

`task.document.execute` 仅允许已认证归属 Agent 以同一任务的两个临时文件授权执行：源文件必须为 `read`，输出必须为 `create-new`。请求仅含授权标识、期望序号、`expected_hash` 和唯一文本替换语义；不接受路径、授权根、ZIP/XML、覆盖标志或确认布尔值。输出固定为新建目标，Agent 不能通过该入口覆盖或删除原文件。

Gateway 在认证、归属、版本和 CAS 检查后，使用 TM-S7 启动事务登记 attempt；Application 解析源/输出授权，调用既有 Document Port 与受控 File Port `save_as`。新建输出授权在解析时消费；源读取授权可复用。所有路径、OOXML、正文和完整替换文本不得进入日志、事件、Outbox、Task Space 或错误消息。返回仅包含格式、输出哈希、大小与安全执行摘要。

源哈希必须等于源授权签发时的快照哈希且等于请求的 `expected_hash`；File Port 在提交前仍复核身份、哈希、目标不存在、锁与原子替换。未知结果不重试。覆盖原件继续只允许可信 LocalUser 的现有用例；大文档/大响应不因本 Gateway 打开新的传输通道。

## 验证

验证双授权/跨任务隔离、hash 冲突、唯一匹配、目标存在、宿主锁、一次性输出消费、任务启动事实、无路径响应和 macOS TaskHost/Gateway 结构化日志。Windows 明确延期且不得计为通过。
