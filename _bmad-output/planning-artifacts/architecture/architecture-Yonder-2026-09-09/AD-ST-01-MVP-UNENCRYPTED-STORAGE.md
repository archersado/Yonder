# AD-ST-01 MVP 暂不加密本地存储

状态：Accepted（2026-09-14，用户明确变更）。Architecture Impact：architecture-change（存储加密与凭据接线延期）。

## 来源与决定

用户明确要求“目前先不做加密，记下加密的需求MVP版本之后再处理”。MVP使用未加密SQLite保存任务当前状态、事件、Outbox与索引；内容附件暂不实施文件级加密。数据库读取不依赖密钥或Credential Store。SQLCipher、附件认证加密、系统凭据及子密钥接线保留为MVP之后需求，不删除原始需求与既有Spike证据。

本决定覆盖AD-E0-06、AD-OCT-01、AD-DS-01及架构主干中MVP必须加密/无Key Provider不得接线的约束；历史Accepted证据仍代表当时验证，不作为当前MVP门禁。SQLite当前状态表仍唯一事实源，状态/事件/Outbox同事务、序号、恢复、授权、日志脱敏、隐私排除、路径规范化与本地配额不变。

## 实施与格式

先更新Story/OpenSpec，再修改存储Adapter。复用现有rusqlite与表结构；未加密打开路径须显式命名并按当前MVP决定使用，不通过试密钥或自动回退判断格式。不修改现有加密库，不把打不开的旧库当空库创建或覆盖。当前无自动迁移；后续迁移另建Change。

## MVP之后待办

ST-S2承接系统Credential Store、SQLCipher数据库/FTS、附件AES-GCM与用途隔离子密钥。恢复实施前补齐三份设计和独立Proposal，设计明文到加密迁移：备份、显式格式、校验、临时写入/原子切换、失败保留原库与恢复验证。Windows/macOS分别提供证据，完成后才能切换产品存储。禁止将固定测试密钥用于产品。
