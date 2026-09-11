# Verification Goal：E0-S6

复核真实 SQLCipher 版本、密文文件头、错误密钥拒绝、加密库内 FTS5、三表事务原子性、附件认证加密篡改拒绝及 Windows Credential Manager 删除后不可读。若仅使用普通 SQLite、主密钥落数据库/附件/日志、错误密钥可读、篡改返回明文或事务出现部分提交，Goal 失败。

当前状态：Windows 范围通过。Windows MSVC 与 Linux 对照均为真实 SQLCipher `4.14.0 community`；密文文件头、错误密钥、FTS5、三表事务回滚、AES-GCM 篡改拒绝及 Credential Manager 删除后不可读均有可复现证据。macOS Keychain 按已确认范围延期。
