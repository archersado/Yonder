# 设计

Application 的 `FileAuthorizationRegistry` 只保存无正文授权记录。签发函数要求可信 `AuthContext::LocalUser`、当前任务快照、合法稳定 `grant_id`、用途和有效期；任务必须非终态。Registry 以 Mutex 保护固定容量映射，签发重复 ID 拒绝，不自动驱逐有效记录。

既有文件通过 `file::read` 获得规范路径、身份和 SHA-256后立即丢弃字节。新建目标通过 File Port `inspect_create_target` 获得规范目标路径与父目录身份；执行时仍调用 `write_atomic(CreateNew)`，父目录或目标竞态由 Adapter 再验证。

解析要求可信 Agent 身份与当前任务 owner 一致，且 grant 的 task/owner/purpose/有效期全部匹配。读取用途返回克隆且保留；新建、替换、回收站在锁内先移除再返回，确保并发请求只有一个取得能力。过期记录在访问时清理。Registry 不记录路径、正文或 Payload。
