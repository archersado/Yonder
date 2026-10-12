# Agent Reply Card Interaction Delta

## MODIFIED Requirements

### Requirement: Agent 回复以受限 Markdown 展示

Yonder MUST 在当前回复卡内把累计 Agent 回复渲染为受限 Markdown；同类型连续或松散列表 MUST 保持单一语义列表并显示明确项目标记，同时保持 AG-S9 的关联、序号、大小和内存边界。

#### Scenario: 流式 Markdown 跨帧补齐

- **WHEN** Markdown 标记分布在多个严格递增 delta 中
- **THEN** Yonder 对累计正文重新渲染
- **AND** 未闭合阶段安全显示，补齐后形成对应语义节点

#### Scenario: 常用格式

- **WHEN** 回复包含标题、段落、列表、引用、代码块或行内强调
- **THEN** 回复卡使用对应可访问语义和可读样式展示
- **AND** 用户仍可选择和复制文本

#### Scenario: 松散列表

- **WHEN** Agent 回复的同类型列表项目之间包含空行
- **THEN** Yonder 将这些项目展示在同一个 `ul` 或 `ol` 中
- **AND** 圆点或十进制编号清晰可见

## ADDED Requirements

### Requirement: 圈选确认与回复卡可拖动

Yonder MUST 允许用户从圈选卡标题区拖动当前无边框窗口，且不得破坏卡片内容交互。

#### Scenario: 从标题区拖动

- **WHEN** 用户以主键从卡片标题区开始拖动
- **THEN** 系统使用原生窗口拖动移动当前卡片
- **AND** 回复、输入和 Agent 运行状态保持不变

#### Scenario: 内容区交互

- **WHEN** 用户在正文、代码、输入、图片或按钮区域按下、选择、滚动或点击
- **THEN** 系统不启动窗口拖动
- **AND** 原内容交互正常完成

#### Scenario: 重新打开

- **WHEN** 用户关闭并再次圈选
- **THEN** 卡片按新选区重新定位
- **AND** 不恢复上一次拖动位置
