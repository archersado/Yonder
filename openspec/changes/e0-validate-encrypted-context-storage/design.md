当前归属 Story：ST-S1；规划：`docs/specs/epic-ST/story-ST-S1/README.md`。旧编号保留历史追溯。

# 设计

Rust Harness 固定使用 `rusqlite 0.40.1` 的 `bundled-sqlcipher-vendored-openssl` 特性，避免依赖用户机器预装 SQLCipher/OpenSSL。数据库连接后立即设置 key，再读取 `cipher_version`；同一加密库内使用 SQLite 原生 FTS5 `trigram` 验证中英文检索，并验证状态/事件/Outbox 原子事务。`trigram` 的 MATCH 查询至少需要 3 个 Unicode 字符；MVP 接受该限制，不引入中文分词库。

附件使用 RustCrypto `aes-gcm 0.11.1` 的 AES-256-GCM。Spike 使用固定测试向量保证可复现；正式实现必须从系统随机源生成每附件唯一 96 位 nonce，并以版本化文件头保存算法、nonce 与密文，不保存主密钥。

Windows 探针直接调用 `CredWriteW`、`CredReadW`、`CredDeleteW`，只输出往返与删除结果。主密钥只进入当前登录用户的 Credential Manager，不落日志。macOS Keychain 按当前范围延期。

不建立产品仓储层、迁移系统或密钥轮换协议；这些属于选型通过后的产品 Story。

2026-09-11 补充：macOS Keychain 探针直接调用 Security.framework SecItem API，临时 UUID 条目与系统随机 32 字节密钥原生验证通过；仅输出状态。它不是产品 Adapter，完整 macOS 路线保持待验收。
