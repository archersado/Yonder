# TaskStore 聚合端口边界

## ADDED Requirements

### Requirement: Store 能力必须一次探测

Gateway MUST 在握手时只获取一次不可变 `StoreCapabilities`，并从该快照派生全部存储相关能力，不得在协商分支中散布独立 `supports_*` 调用。

#### Scenario: Gateway 握手

- **WHEN** Agent 提交合法 hello
- **THEN** Store 能力只探测一次
- **AND** hello 的版本、能力顺序、availability 与 reason 和重构前完全一致

### Requirement: TaskStore 必须按职责拆分

Application MUST 以核心、执行、审计、计划片段四个子 trait 定义持久化端口，并以组合 `TaskStore` 保持现有调用边界。

#### Scenario: 新增 Store 实现

- **WHEN** Adapter 实现完整 `TaskStore`
- **THEN** 编译器要求其满足四个职责子 trait
- **AND** Application 不依赖具体 Adapter

### Requirement: SQLite 分区不得改变持久化行为

SQLite 实现文件重组 MUST 保持 SQL、schema、迁移、事务和错误语义不变。

#### Scenario: 结构重组验证

- **WHEN** 分区后的 Adapter 运行既有回归
- **THEN** 原有断言无需修改且全部通过
- **AND** `PRAGMA user_version` 与迁移测试结果不变
