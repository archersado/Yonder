# Agent Reply Markdown Presentation Delta

## ADDED Requirements

### Requirement: Agent 回复以受限 Markdown 展示

Yonder MUST 在当前回复卡内把累计 Agent 回复渲染为受限 Markdown，同时保持 AG-S9 的关联、序号、大小和内存边界。

#### Scenario: 流式 Markdown 跨帧补齐

- **WHEN** Markdown 标记分布在多个严格递增 delta 中
- **THEN** Yonder 对累计正文重新渲染
- **AND** 未闭合阶段安全显示，补齐后形成对应语义节点

#### Scenario: 常用格式

- **WHEN** 回复包含标题、段落、列表、引用、代码块或行内强调
- **THEN** 回复卡使用对应可访问语义和可读样式展示
- **AND** 用户仍可选择和复制文本

### Requirement: Markdown 不得扩大内容执行边界

Markdown 渲染 MUST 不执行 Agent 提供的 HTML、脚本、事件属性、危险 URL 或外部资源。

#### Scenario: 不可信富内容

- **WHEN** 回复包含原始 HTML、脚本、Markdown 图片、`javascript:` 或其他不支持协议
- **THEN** Yonder 只显示安全文本或占位
- **AND** 不创建可执行节点、不导航、不发起资源请求

#### Scenario: 渲染异常

- **WHEN** 受限解析器无法处理累计正文
- **THEN** Yonder 回退显示完整纯文本
- **AND** 回复序号、失败、中断、截断和清理状态不受影响
