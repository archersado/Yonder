# agent-reply-presentation Specification

## Purpose
定义 Yonder 如何在既有 AgentSession 上协商、关联并临时展示圈选输入对应的有界 Agent 回复，同时保持 Gateway 单一入口和正文不持久化边界。

## Requirements

### Requirement: 回复能力显式协商

需要回复回显的输入 MUST 只发送到声明 `user_input_reply` 的同一已认证 AgentSession。

#### Scenario: 圈选输入请求回复

- **WHEN** 当前唯一 Agent session 声明回复能力且用户提交圈选问题
- **THEN** 输入携带流式回复模式和稳定 input ID
- **AND** accepted 只表示 Agent 已接收输入

#### Scenario: Agent 不支持回复

- **WHEN** Agent 支持输入但未声明回复能力
- **THEN** 圈选输入仍可按原契约发送
- **AND** Yonder 明确提示用户到 Agent 客户端查看回答

### Requirement: 同一 Gateway 回传有界回复

Connector MUST 通过同一 AgentSession 把匹配输入的最终 Agent 消息按严格序号回传，不建立旁路。

#### Scenario: 成功回答

- **WHEN** App Server 为关联 turn 发布最终 Agent 消息增量，并以 completed 结束
- **THEN** Connector 发送 started、零到多个 delta 和唯一 completed
- **AND** Yonder 只向匹配 input/session 的当前圈选卡展示

#### Scenario: 非成功终态

- **WHEN** turn failed、interrupted、连接断开或通知不合法
- **THEN** Yonder 显示失败或中断
- **AND** 不把 input accepted 或已有部分正文显示为完成

#### Scenario: 回复超时

- **WHEN** 关联回复在 5 分钟内未到达终态
- **THEN** Connector 回传唯一 `interrupted`
- **AND** Yonder 释放本次临时关联而不补抓历史正文

### Requirement: 回复不持久化

回复正文 MUST 只在当前运行时与圈选卡内有界存在。

#### Scenario: 完成、关闭或重启

- **WHEN** 用户关闭卡片、会话终止或进程重启
- **THEN** Yonder 释放问题、截图和回复正文
- **AND** 不从 SQLite、任务事件、Outbox、日志、索引或 Codex 私有历史恢复

### Requirement: UI 有界且可诊断

圈选卡 MUST 区分发送、等待回复、流式回答、完成、失败、中断和截断。

#### Scenario: 回复达到上限

- **WHEN** 累计可见回复达到 64 KiB
- **THEN** 后续正文不再追加
- **AND** 卡片显示“回答过长，已截断”而不是静默丢失

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
