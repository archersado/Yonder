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

#### Scenario: 导出无文本图像控件

- **WHEN** transcript 包含只能由截图视觉消歧的 AX image 节点
- **THEN** 协议 1.46 Observation 以 `image` 返回受最新引用保护的临时句柄，无文本节点只使用通用角色标签，协议 1.45 及以下不返回该句柄

#### Scenario: 图像只是控件装饰或节点已禁用

- **WHEN** AX image 是可操作按钮等节点的后代，或候选节点包含 disabled 状态
- **THEN** Worker 不把装饰性 image 或 disabled 节点另行导出为可操作句柄，也不根据相邻节点猜测代理索引

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

### Requirement: 视觉坐标绑定原始截图 Observation

系统 MUST 只接受绑定同次 Observation 的原始截图像素坐标，不得把渲染坐标或猜测的显示缩放直接传给 Driver。

#### Scenario: 返回原始截图尺寸

- **WHEN** Observation 携带截图
- **THEN** 同时返回从实际图片字节读取的原始像素宽高与 `observation_ref`

#### Scenario: 最新引用与坐标有效

- **WHEN** 坐标候选携带同任务最新 `observation_ref`，且原始像素坐标位于截图边界内、动作前截图尺寸未变化
- **THEN** Worker 只向同一绑定应用派发一次 Sky 坐标动作，并在动作后重新 Observe

#### Scenario: 引用、边界或截图几何失效

- **WHEN** 引用过期或跨任务、坐标越界，或动作前截图尺寸与引用记录不一致
- **THEN** Worker 不派发动作，返回新鲜 Observation 交回慢脑，不缩放或重放旧坐标

#### Scenario: 同尺寸界面语义已变化

- **WHEN** 动作前截图宽高未变化，但 transcript 已不同于产生 `observation_ref` 的 Observation
- **THEN** Worker 将旧坐标视为失效，不派发动作，并以新鲜 Observation 交回慢脑

#### Scenario: macOS 调用边界使用可信 backing scale

- **WHEN** 最新 Observation 的原始截图像素坐标通过全部引用与边界校验，且 Adapter 从当前可信窗口所在显示器取得有效 backing scale
- **THEN** Worker 只在 Sky 调用边界将原图像素转换为窗口逻辑点，Agent、UI 与计划参数均不提供倍率

#### Scenario: backing scale 不可用或已变化

- **WHEN** 当前窗口/显示器无法解析可信倍率，或倍率不同于产生最新 Observation 时的倍率
- **THEN** Worker 不派发坐标动作，返回新鲜 Observation 交回慢脑，且不得猜测倍率或重放旧坐标
