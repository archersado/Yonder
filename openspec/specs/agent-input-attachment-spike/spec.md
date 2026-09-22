# agent-input-attachment-spike Specification

## Purpose
定义Agent会话临时附件产品接线前的技术门禁：用无敏感样本验证有界分块、会话隔离、完整性与所有结束路径清理，防止产品协议发送本机路径、持久化截图或残留会话缓冲。

## Requirements

### Requirement: 有界会话附件样本

Spike MUST 保持既有单帧上限，并限制单个临时附件总量。

#### Scenario: 成功传输边界样本

- **WHEN** Spike传输固定生成的无敏感附件字节
- **THEN** 每个帧不得超过64 KiB，附件总量不得超过4 MiB
- **AND** 完整哈希校验通过后才允许当前会话引用

### Requirement: 会话隔离与失败清理

Spike MUST 按会话隔离附件，并在所属会话失败或结束后清理缓冲。

#### Scenario: 所属会话失败

- **WHEN** 第二个或重复附件开始、分块乱序、超限、哈希错误、输入拒绝或结果未知、超时或断连
- **THEN** Spike拒绝附件且所属会话接收缓冲归零
- **AND** 不自动重传、不持久化字节或输出附件正文

#### Scenario: 其他会话引用

- **WHEN** 第二个会话引用第一个会话的附件标识
- **THEN** Spike拒绝引用且不能读取或删除第一个会话的附件
- **AND** 第一个会话断开后其接收缓冲归零

### Requirement: 产品门禁

产品协议 MUST 等待Spike与独立验证通过。

#### Scenario: Spike尚未通过

- **WHEN** Spike和独立Verification Goal尚未通过
- **THEN** 不得修改产品协议、扩大产品帧上限或启用确认卡发送
