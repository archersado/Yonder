# Region Question Submit Recovery Delta

## ADDED Requirements

### Requirement: 未接受的圈选输入可恢复

Yonder MUST 只在 Agent 明确接受圈选输入后清除提交临时数据。

#### Scenario: Agent 拒绝输入

- **WHEN** 当前圈选输入返回 rejected
- **THEN** 图片、问题和来源保持在当前确认卡
- **AND** 用户可修正或再次发送

#### Scenario: 送达状态未知

- **WHEN** 当前圈选输入返回 unknown
- **THEN** 图片、问题和来源保持用于诊断
- **AND** Yonder 禁止自动或手动重复发送本次输入，直到用户关闭或重新圈选

### Requirement: 附件上限在投递前可诊断

Yonder MUST 在发送附件帧和建立回复关联前按协议附件上限校验解码后的截图。

#### Scenario: 截图超过附件上限

- **WHEN** 解码后的截图超过协议允许的 4 MiB
- **THEN** 输入不进入 Agent 投递
- **AND** 确认卡保留内容并提示用户缩小选区后重新圈选

### Requirement: 失败反馈不得销毁草稿

圈选卡 MUST 对发送前错误和明确拒绝给出具体反馈，且不得以错误处理清空问题或预览。

#### Scenario: Agent 通道不可用

- **WHEN** 没有 Agent、目标未选择、附件能力不支持或通道异常
- **THEN** 卡片显示对应原因并保留本次内容
- **AND** 只有用户明确关闭、重新圈选或后续 accepted 才清除临时数据
