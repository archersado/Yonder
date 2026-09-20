# Region Question Submit Delta

## ADDED Requirements

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
