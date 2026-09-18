# 历史记录（只读追溯，不作为当前规划）

# E0-S6 加密上下文存储验证

## Story

作为 Yonder 开发团队，我们需要验证 SQLCipher、FTS5、附件认证加密与 Windows Credential Manager 的组合，确定个人上下文能否安全持久化。

## 验收条件

1. SQLite 文件使用 SQLCipher，`cipher_version` 非空，磁盘文件头不得出现 SQLite 明文标识，错误密钥必须 fail closed。
2. FTS5 在加密数据库内可创建、写入和检索中文及英文内容。
3. 当前状态、追加事件和 Outbox 在同一事务内提交；故障回滚后不得出现部分写入，每任务 `sequence` 唯一且递增。
4. 附件使用带认证的文件级加密；正确密钥可恢复，密文或标签篡改必须拒绝，不得返回部分明文。
5. 32 字节主密钥可写入、读取并从 Windows Credential Manager 删除；数据库和附件不得存储主密钥。
6. 记录构建时间、Release 体积、基本读写耗时、许可证与 Windows 分发风险。
7. Spike 不引入仓储抽象、迁移框架、向量库或跨设备密钥同步。

OpenSpec：`openspec/changes/e0-validate-encrypted-context-storage/`

## macOS 补充批次：2026-09-11

为 OCT-S1 凭据接线补齐前置证据，本次时间盒验证 Security.framework 临时 32 字节密钥往返、重复写入拒绝、删除后不可读。失败即保留 macOS 路线待定，不开始依赖它的产品实现。Windows 范围的既有结论不变。
