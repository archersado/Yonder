# CUA Observation Element Handles

## ADDED Requirements

### Requirement: 新鲜 Observation 可返回临时可操作元素句柄

系统 MUST 只从当前 Sky transcript 派生有界的可操作元素句柄，不返回完整 AX 树或静态正文。

#### Scenario: 返回可操作句柄

- **WHEN** 动作后新鲜 transcript 包含按钮、菜单项或文本框
- **THEN** Observation 返回最多 128 个只含索引、封闭角色和短标签的句柄以及不透明 `observation_ref`

#### Scenario: 过滤敏感与静态内容

- **WHEN** transcript 包含消息正文、静态文本或密码输入
- **THEN** 这些内容不得进入句柄、日志、事件、Outbox、SQLite或顶部浮窗

### Requirement: 元素动作绑定同次 Observation

系统 MUST 要求元素索引动作携带产生该索引的最新 `observation_ref`，并在动作前重新Observe确认索引仍可操作。

#### Scenario: 最新引用与索引有效

- **WHEN** 同任务下一片段提交最新引用和仍存在的可操作索引
- **THEN** Worker只派发一次Sky元素动作，并在动作后重新Observe验证效果

#### Scenario: 引用或索引失效

- **WHEN** 引用来自旧Observation、其他任务、Worker重建，或索引已消失
- **THEN** Worker不得派发动作，返回新鲜Observation并交回慢脑

#### Scenario: 动作回执但后置事实不足

- **WHEN** Sky动作调用返回但transcript与视觉事实不能确认变化
- **THEN** 结果保持unknown/handback，且不得自动重试
