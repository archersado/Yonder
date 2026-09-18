当前归属 Story：ST-S1；规划：`docs/specs/epic-ST/story-ST-S1/README.md`。旧编号保留历史追溯。

# 独立 Verification Goal：E0-S6 macOS Keychain

日期：2026-09-11。关联 E0-S6 补充批次、AD-E0-06、context-storage 的临时凭据往返与精确清理场景。Architecture Impact：conforming。

状态：Keychain 原生探针通过；macOS 完整存储路线尚未验收，本批不 Archive、不扩展 ADR 已接受的平台范围。

## 环境与实测

macOS 本机，系统 Swift + Security.framework；执行命令：

```sh
swift -module-cache-path /private/tmp/yonder-swift-module-cache spikes/encrypted-context-storage/keychain.swift
```

进程退出码 0，运行工具计时 0.382 秒。结构化证据：`spikes/encrypted-context-storage/evidence/keychain-macos-20260911.json`。写入、字节往返、重复写入拒绝、删除及删除后不可读均为 true。初次 Swift 编译因默认位置参数调用错误失败，在任何凭据操作前终止；修正调用顺序后原生运行通过。

探针使用 SecRandomCopyBytes 生成 32 字节密钥，服务名含新 UUID；每次查询限定 generic password、service、account 和非同步属性。不枚举已有凭据，不记录密钥、密钥摘要或命令参数中的秘密。成功运行已清理自己的临时条目；异常路径也尝试精确清理，失败才记录服务名与 OSStatus。

Windows 对照采用已记录的 credential-manager.ps1 32 字节往返与删除样本，本轮未重跑 Windows；新增重复项检查只有 macOS 证据。不将平台参数模拟视为原生证据。

## 限制

尚未验证锁屏/锁定 Keychain、用户拒绝授权、跨进程重启读取、签名应用 ACL 或分发行为；当前验证只覆盖本进程原生 API。Swift/CF 可复制数据，本探针不能保证所有内存副本清零；不得直接作为产品主密钥 Adapter 使用。

未实现 KDF、产品子密钥、凭据迁移、Agent 认证或 Gateway 宿主接线；未打开用户数据库。OCT-S1 真实身份认证待办保持未完成。
