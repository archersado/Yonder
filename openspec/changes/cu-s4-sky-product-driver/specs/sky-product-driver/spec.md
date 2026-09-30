# Sky Product Driver Delta

## ADDED Requirements

### Requirement: macOS只使用Sky产品Driver

系统 MUST 在macOS产品组合根只构造固定版本Sky Worker；不得携带或回退trycua。

#### Scenario: Sky依赖不可用

- **WHEN** 固定Sky入口或包身份校验失败
- **THEN** CUA capability为unavailable，且不启动其他CUA Driver

### Requirement: 新鲜应用级AX元素优先

系统 MUST 对同任务绑定的唯一应用在每步前后读取新鲜AX transcript，并优先使用当前`element_index`执行。

#### Scenario: QQ音乐公开唯一搜索框

- **WHEN** 新鲜transcript包含唯一可操作的搜索文本框
- **THEN** Worker使用该次index执行聚焦或赋值，动作后重新Observe且不走坐标兜底

#### Scenario: 元素缺失或多义

- **WHEN** 语义目标无法从新鲜transcript唯一解析
- **THEN** Worker停止动作并返回同应用有界视觉Observation供慢脑重规划，不自动重试

### Requirement: AX transcript不持久化

系统 MUST 仅在Worker内使用AX transcript；不得写入任务状态、事件、Outbox、日志或顶部浮窗。

#### Scenario: 动作完成

- **WHEN** Worker取得动作前后transcript
- **THEN** 对外Observation只包含元素数量、受控截图引用与可见性，不包含transcript正文
