# Region Preview Delta

## ADDED Requirements

### Requirement: 来源应用仅在本轮确认卡显示

系统 MUST 在用户显式启动圈选时读取一次有界来源应用显示名称，并只在当前确认卡显示。

#### Scenario: 显示可信来源

- **WHEN** 平台在覆盖层出现前返回有效前台应用名称
- **THEN** 确认卡显示“来源：应用名”
- **AND** 重新圈选继续显示同一来源

#### Scenario: 来源不可用

- **WHEN** 平台未返回名称或名称不满足有界校验
- **THEN** 确认卡显示“来源：当前桌面”

#### Scenario: 来源数据边界

- **WHEN** 圈选会话关闭、取消、提交或超时
- **THEN** PreviewSession清除来源名称
- **AND** 来源名称不进入Agent正文、协议、日志、SQLite、事件或Outbox
