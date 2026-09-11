# 加密上下文存储规范增量

## Requirement：加密数据库

系统 SHALL 使用 SQLCipher 保存元数据、当前状态、事件、Outbox 与 FTS5 索引。

### Scenario：错误密钥

- **WHEN** 使用错误密钥打开已有数据库
- **THEN** 读取 schema 或业务数据失败
- **AND** 不返回部分数据

### Scenario：原子状态变更

- **WHEN** 应用提交任务状态、下一事件和 Outbox 消息
- **THEN** 三者在同一事务全部提交
- **AND** 任一步失败时全部回滚

## Requirement：附件认证加密

系统 SHALL 使用文件级认证加密保存大内容附件。

### Scenario：密文被篡改

- **WHEN** 附件密文或认证标签发生任意改变
- **THEN** 解密失败
- **AND** 不向调用方返回明文

## Requirement：系统凭据存储

系统 SHALL 只在当前用户的系统 Credential Store 中保存主密钥。

### Scenario：删除设备密钥

- **WHEN** 用户删除设备数据
- **THEN** 系统删除对应 Credential
- **AND** 后续读取返回不存在
