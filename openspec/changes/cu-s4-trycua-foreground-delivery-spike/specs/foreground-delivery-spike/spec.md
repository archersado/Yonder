# Foreground Delivery Spike Delta

## ADDED Requirements

### Requirement: 固定候选与统一样本

Spike MUST 固定比较trycua 0.25.0与0.30.4，并对同一隔离fixture使用相同目标和Observe判据。

#### Scenario: 候选Schema核对

- **WHEN** 运行Spike准备阶段
- **THEN** 证据记录精确窗口target、坐标形式、delivery mode和动作效果字段是否存在，不使用浮动版本

### Requirement: 前台投递必须由后置事实确认

Spike MUST 以精确窗口前台click建立焦点，随后输入固定非敏感标记，并由独立原生读取与SDK Observe共同确认。

#### Scenario: 请求接受但后置事实不成立

- **WHEN** Driver返回成功或接受，但焦点或控件值未匹配
- **THEN** 样本失败并停止，不重试点击或输入

### Requirement: 恢复与错误目标 fail-closed

Spike MUST 在所有终止路径验证原前台恢复，并拒绝错误、关闭或过期的目标。

#### Scenario: 恢复失败

- **WHEN** 动作结束后原前台未在有界时间恢复
- **THEN** 淘汰“单次前台并恢复”路线，不以输入成功覆盖恢复失败

### Requirement: Spike与产品隔离

Spike MUST 不修改正式依赖、Gateway、协议、SQLite或UI。

#### Scenario: Spike通过

- **WHEN** macOS统一样本全部通过
- **THEN** 只允许更新ADR结论并建立独立Apply Change，不直接声明产品链路通过
