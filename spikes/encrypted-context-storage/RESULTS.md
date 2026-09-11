# 验证结果

日期：2026-09-11

## 已通过

- Linux 与 Windows MSVC 均运行 SQLCipher `4.14.0 community`，数据库文件头不是 `SQLite format 3`，错误密钥触发页 HMAC 失败并拒绝读取。
- SQLCipher 内 FTS5 `trigram` 可检索中文 3 字符子串与英文词；MATCH 查询少于 3 个 Unicode 字符不在 MVP 保证范围。
- 成功事务同时写入状态、事件与 Outbox；第二个事务在 Outbox 唯一键冲突后，先前状态和事件写入全部回滚。
- AES-256-GCM 往返成功；修改一个密文字节后认证失败且不返回明文。
- Windows Credential Manager 临时 Generic Credential 写入、32 字节内存往返、删除及删除后不可读全部通过；密钥未输出。

## 成本

- Linux 冷构建：约 2 分 12 秒；Release 单文件 7,994,872 字节；热运行墙钟约 1.48 秒，Harness 内约 467 毫秒。
- Windows MSVC 冷构建：约 15 分 40 秒；Release 单文件 4,881,408 字节；热运行墙钟约 575 毫秒，Harness 内约 490 毫秒。
- Windows 构建期需要 MSVC Build Tools、NMake 与 Perl；vendored OpenSSL/SQLCipher 不要求最终用户安装这些工具。

## 依赖与许可证

- `rusqlite 0.40.1` / `libsqlite3-sys 0.38.2`：MIT；bundled SQLCipher 为 BSD-style。
- SQLCipher Community `4.14.0`：BSD-style。
- `aes-gcm 0.11.1`：MIT OR Apache-2.0。
- vendored OpenSSL `3.6.3`：Apache-2.0。

## 结论

Windows 范围通过。采用 Rust 进程内 SQLCipher+FTS5 `trigram`、AES-256-GCM 文件附件和 Windows Credential Manager；不引入数据库 Sidecar、Node Worker、向量库或跨设备密钥同步。macOS Keychain 按当前范围延期。
