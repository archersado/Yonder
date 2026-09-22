# region-question-submit Specification

## Purpose
定义圈选确认卡向当前已认证AgentSession提交单张临时截图与问题的能力协商、有界传输、结果反馈和数据清理边界。

## Requirements

### Requirement: 附件能力协商

系统 MUST 只向显式声明截图输入能力的已认证Agent会话提交圈选附件。

#### Scenario: Agent不支持附件

- **WHEN** 用户在确认卡发送，而绑定会话没有声明附件输入能力
- **THEN** 系统显示稳定的不支持错误，不发送文字或附件，也不切换到其他Agent

### Requirement: 同会话有界附件提交

系统 MUST 在同一AgentSession内完成单附件暂存、校验和输入引用。

#### Scenario: Agent接受圈选问题

- **WHEN** 用户确认发送非空问题与有效截图
- **THEN** 每个传输帧不超过64 KiB，附件不超过4 MiB，完整校验通过后`agent.input`才引用附件
- **AND** Agent明确返回accepted后确认卡显示成功并清理文字、截图和附件状态

#### Scenario: 提交失败或结果未知

- **WHEN** 附件或输入被拒绝、超时、断连或返回unknown
- **THEN** 系统不自动重试、不切换Agent、不创建任务，并清理本次附件
- **AND** 确认卡显示稳定结果；再次发送截图必须重新圈选

### Requirement: 任务与数据边界

系统 MUST 保持Agent专属任务创建和临时数据边界。

#### Scenario: 圈选问题已送达

- **WHEN** Agent接受`agent.input`
- **THEN** Yonder不调用`task.create`，只有Agent后续真实创建任务时才显示任务与忙碌状态
- **AND** 问题正文、截图、base64、摘要值和本机路径不进入SQLite、任务事件、Outbox、普通日志或上下文索引

### Requirement: 无截图文字提交

系统 MUST 允许圈选提问入口在没有有效截图时向当前已认证AgentSession提交非空文字，且不得构造空附件或要求附件能力。

#### Scenario: 用户未形成有效区域

- **WHEN** 用户直接点击或选择范围不足最小尺寸
- **THEN** 系统显示无截图确认卡并允许编辑、发送、重新圈选或取消
- **AND** 发送只产生一条`agent.input`，不产生附件帧

#### Scenario: 截图权限不可用

- **WHEN** 用户完成有效选择但系统不能取得截图权限
- **THEN** 确认卡明确显示本次不包含截图，仍允许发送非空文字
- **AND** 系统不伪装截图成功、不自动请求权限、不持久化问题正文

#### Scenario: 无附件能力的Agent接受文字

- **WHEN** 当前AgentSession声明`user_input`但没有声明`user_input_attachment`
- **THEN** 无截图文字提交仍交给该会话，accepted、rejected或unknown沿用既有稳定结果与清理语义
