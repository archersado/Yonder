# 发布签名与分发

## ADDED Requirements

### Requirement: 固定版本发布

Yonder 发布产物 MUST 使用同一固定版本，并包含 desktop、CLI、MCP、IPC 协议、迁移和 Driver manifest。

#### Scenario: 版本一致

发布脚本 MUST 校验提交 SHA、协议版本和 SQLite schema 版本一致；不一致时拒绝发布。

### Requirement: 签名与公证

macOS 产物 MUST 通过 `codesign` 与 `notarytool`；Windows 产物 MUST 通过 code-sign。签名失败 MUST 阻止发布。

#### Scenario: 签名校验

发布脚本 MUST 在本地验证签名链、时间戳和公证结果，并保存结构化证据。

### Requirement: 升级前备份

升级 MUST 在执行迁移前备份 SQLite 主库和必要 sidecar；备份失败 MUST 拒绝升级。

#### Scenario: 迁移失败回退

迁移失败 MUST 保留备份并允许回退旧版本，不得丢失任务数据。
