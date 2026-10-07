# CUA Observation Element Handles

## ADDED Requirements

### Requirement: 新鲜 Observation 可返回临时可操作元素句柄

系统 MUST 把当前 Sky transcript 作为有界临时 Observation 返回归属慢脑，并派生可操作元素句柄；两者均不得持久化。

#### Scenario: 返回可操作句柄

- **WHEN** 动作后新鲜 transcript 包含按钮、菜单项或文本框
- **THEN** Observation 返回最多64 KiB transcript、最多 128 个只含索引/封闭角色/短标签的句柄以及不透明 `observation_ref`

#### Scenario: 导出可选择行

- **WHEN** transcript 中的 AX 行带有 selectable 属性但没有按钮角色
- **THEN** Observation 以 `selectable-row` 返回该行，并只用该行后代中的有界安全文本生成短标签

#### Scenario: 过滤安全输入且禁止持久化

- **WHEN** transcript 包含密码/安全输入或普通界面正文
- **THEN** 安全输入行被过滤，普通 transcript 只在当前 Gateway 响应可见，任何 transcript 与标签都不得进入日志、事件、Outbox、SQLite或顶部浮窗

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

#### Scenario: 坐标调用返回错误

- **WHEN** 坐标点击已提交给 Sky 后返回超时、窗口失效或其他动作错误
- **THEN** Worker 不重放点击，仍执行唯一一次动作后 Observe，并携带新鲜事实返回 unknown/handback
