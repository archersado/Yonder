# 设计

`task.file.execute` 参数包含任务、期望序号、授权标识、操作与可选 Base64 正文。严格反序列化拒绝额外字段；Base64 采用规范字母表和 padding，解码后不超过 48 KiB。操作与授权用途必须一一匹配。

Gateway 的验证顺序为：握手 1.28 与能力、Agent 已登记、会话身份、任务归属、任务 CAS；随后调用组合根持有的 `FileAuthorizationRegistry + FilePort` 用例。用例先用 TM-S7 声明/准备执行 attempt，成功后才解析授权和触碰 File Port。读取复用授权；创建、替换、回收站授权解析时消费，副作用 unknown 后不重试。

读取在 Port 读到 48 KiB 以下快照后返回 Base64 正文和 SHA-256；创建与替换执行既有原子写并只返回大小、内容哈希与受控摘要；回收站只返回成功摘要。路径、授权根、身份、正文不进入持久任务状态、事件、Outbox、日志或桌面 UI。

执行结束不自动将任务标记完成；Agent 仍通过既有 Observe/完成/失败协议推进。File Port 的身份/哈希/父目录/锁检查是最终准入，任何不确定结果映射为 `unknown` 语义。
