# Foreground Text Delta

## ADDED Requirements

### Requirement: 受保护坐标文本参数

协议 MUST 只允许会话目标查询和消息草稿引用的文本动作携带有限窗口坐标，正文仍只能由Application引用解析。

#### Scenario: Agent提交delivery mode或正文

- **WHEN** 计划候选包含`delivery_mode`、`text`、target或坐标外字段
- **THEN** Gateway在持久化和派发前拒绝整个片段

### Requirement: Worker强制动作级前台投递

Worker MUST 为受支持窗口坐标动作注入精确窗口target、受监管session和foreground模式。

#### Scenario: SDK缺少foreground字段

- **WHEN** 当前工具Schema不公开`delivery_mode`
- **THEN** Worker拒绝动作并交回，不降级为后台或桌面全局输入

### Requirement: 确认后置事实才能推进

坐标动作 MUST 在同一精确窗口完成Observe；只有Driver confirmed且Observe有效才能推进计划。

#### Scenario: 不可核实坐标文本

- **WHEN** Driver返回`unverifiable`、`partial`或Observe失败
- **THEN** 当前步骤交回且不得重试、建立焦点凭据或显示成功
