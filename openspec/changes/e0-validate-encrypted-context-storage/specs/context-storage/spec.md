# 加密上下文存储规范增量

## ADDED Requirements

### Requirement: 加密数据库

系统 SHALL 使用 SQLCipher 保存元数据、当前状态、事件、Outbox 与 FTS5 索引。

#### Scenario: 错误密钥

- **WHEN** 使用错误密钥打开已有数据库
- **THEN** 读取 schema 或业务数据失败
- **AND** 不返回部分数据

#### Scenario: 原子状态变更

- **WHEN** 应用提交任务状态、下一事件和 Outbox 消息
- **THEN** 三者在同一事务全部提交
- **AND** 任一步失败时全部回滚

### Requirement: 附件认证加密

系统 SHALL 使用文件级认证加密保存大内容附件。

#### Scenario: 密文被篡改

- **WHEN** 附件密文或认证标签发生任意改变
- **THEN** 解密失败
- **AND** 不向调用方返回明文

### Requirement: 系统凭据存储

系统 SHALL 只在当前用户的系统 Credential Store 中保存主密钥。

#### Scenario: 删除设备密钥

- **WHEN** 用户删除设备数据
- **THEN** 系统删除对应 Credential
- **AND** 后续读取返回不存在

### Requirement: macOS 原生 Keychain 补充探针

系统 SHALL 使用原生 Security.framework 验证临时凭据写入、读取与精确清理。

关联 E0-S6、AD-E0-06 补充验证；Architecture Impact：conforming。期限为 2026-09-11 本次增量；仅验证待定技术路线，不启动产品凭据 Adapter。

#### Scenario: 临时凭据往返与精确清理
使用系统随机源创建 32 字节测试密钥和 UUID 隔离条目；SecItemAdd 成功、同名添加返回 errSecDuplicateItem、读取字节一致、删除成功且随后读取严格返回 errSecItemNotFound。所有查询限定临时 service/account，禁止宽泛枚举或删除。清理失败输出条目标识便于人工处理；输出不得包含密钥。Windows 以既有 Credential Manager 探针为统一样本对照。
