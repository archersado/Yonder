# gateway-capability-routing Specification

## Purpose
约束 Gateway 从单一声明式来源派生能力协商结果，并以穷举请求分类保持版本门禁可审计、可由编译器检查且与既有 wire 行为等价。

## Requirements

### Requirement: 能力协商必须由单一声明式来源派生

Gateway MUST 从同一能力规格与协商快照派生会话能力、hello 协议版本和有序能力列表，不得维护互相独立的版本链。

#### Scenario: 旧 minor 客户端握手

- **WHEN** 客户端以协议 major 1、minor 0 至当前版本中的任一版本握手
- **THEN** 返回的协议版本、能力顺序、能力版本、availability 与 reason 必须与重构前 golden 完全一致

### Requirement: 请求能力门禁必须穷举

Gateway MUST 对每个协议 Request 变体显式声明其协商能力门禁，使新增变体未分类时在编译期失败。

#### Scenario: 未协商能力的请求

- **WHEN** 已握手会话提交当前版本不支持的写入或执行请求
- **THEN** Gateway 返回该请求既有的 `-32010` 错误码和原文案
- **AND** 不调用 Store、Runtime 或 Driver

### Requirement: 重构不得改变外部事实

本 Change MUST 保持 wire 类型、SQLite schema/user_version、事件/Outbox、依赖方向和 UI 行为不变。

#### Scenario: 等价性门禁

- **WHEN** Change 进入验证
- **THEN** workspace 测试、协议生成检查和架构检查全部通过
- **AND** 不产生迁移或协议生成物差异
