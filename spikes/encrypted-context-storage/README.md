# 加密上下文存储 Spike

运行 Rust 验证：

```bash
cargo run --release
```

验证覆盖 SQLCipher 版本与密文文件头、错误密钥、FTS5 `trigram` 中英文检索、状态/事件/Outbox 回滚，以及 AES-256-GCM 往返与篡改拒绝。`trigram` 查询至少 3 个 Unicode 字符；固定 key/nonce 仅为可复现测试向量，禁止进入产品代码。

依据：[SQLCipher API](https://www.zetetic.net/sqlcipher/sqlcipher-api/)、[rusqlite 上游说明](https://github.com/rusqlite/rusqlite)、[RustCrypto AES-GCM](https://docs.rs/aes-gcm/latest/aes_gcm/) 与 [Windows CredReadW](https://learn.microsoft.com/windows/win32/api/wincred/nf-wincred-credreadw)。

Windows Credential Manager 实机验证会创建一个随机命名的临时 Generic Credential，完成内存比较后立即删除，不输出主密钥：

```powershell
.\credential-manager.ps1
```

Windows 原生 MSVC 构建使用已安装的 Build Tools 与 Strawberry Perl，仅为当前进程加载环境：

```powershell
.\run-windows.ps1
```
