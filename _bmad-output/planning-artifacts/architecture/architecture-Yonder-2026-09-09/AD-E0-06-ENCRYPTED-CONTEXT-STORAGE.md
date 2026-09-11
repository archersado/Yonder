# AD-E0-06 加密上下文存储

- 状态：Accepted（Windows 范围）
- Story：E0-S6
- OpenSpec：`e0-validate-encrypted-context-storage`
- 日期：2026-09-11

## 决定

首版本地上下文存储使用 Rust 进程内 `rusqlite 0.40.1`，启用 `bundled-sqlcipher-vendored-openssl`；FTS5 使用原生 `trigram` tokenizer。大内容附件使用 RustCrypto `aes-gcm 0.11.1` 的 AES-256-GCM。设备主密钥仅保存于 Windows Credential Manager，数据库与附件用途的子密钥在内存中按用途派生。

不使用系统普通 SQLite、数据库 Sidecar、Node Worker、本地向量库或跨设备密钥同步。

## 证据

- Linux 与 Windows MSVC 均验证 SQLCipher `4.14.0 community`、密文文件头和错误密钥 HMAC 拒绝。
- 加密数据库内 FTS5 `trigram` 中英文检索通过。
- Outbox 唯一键冲突发生在状态和事件写入后，事务整体回滚，无部分状态。
- AES-256-GCM 正确密钥往返成功，单字节篡改被认证拒绝。
- Windows Credential Manager 的临时 32 字节凭据完成写入、读取、删除及删除后不可读。
- Windows Release 为 4,881,408 字节，热运行约 575 毫秒；冷构建约 15 分 40 秒。

## 架构围栏

- 每个数据库连接必须先设置 key，再执行任何 schema 或业务查询；启动时必须校验 `cipher_version`。
- 当前状态、事件和 Outbox 必须在同一 SQLCipher 事务中提交；`(task_id, sequence)` 使用唯一约束。
- FTS5 `trigram` 的 MATCH 查询至少 3 个 Unicode 字符；MVP 不为短查询引入中文分词器。
- 附件格式必须版本化；每个附件使用系统随机源生成唯一 96 位 nonce，并将格式版本、算法和附件标识作为 AAD。写入先到临时文件，认证数据完整后原子提交。
- 数据库与附件不得直接复用同一子密钥；子密钥仅驻内存，敏感缓冲区使用后清零。
- Credential 名称可记录，Credential Blob、SQLCipher key、附件 key 和 nonce 生成材料不得写入数据库、附件或日志。
- Windows 构建机必须固定 MSVC Build Tools 与 Perl；最终应用静态包含 SQLCipher/OpenSSL，不要求用户安装构建工具。
- macOS Keychain 与原生构建证据延期，本 ADR 不构成 macOS 技术路线完成。
