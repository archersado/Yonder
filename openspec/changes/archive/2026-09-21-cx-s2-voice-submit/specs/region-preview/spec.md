# Region Preview Delta

## ADDED Requirements

### Requirement: 圈选语音直接提交

系统 MUST 只在用户从当前圈选确认卡显式开始语音后采集，并将最终非空转写作为当前selection请求直接提交。

#### Scenario: 携带当前截图提交

- **WHEN** 确认卡持有临时截图，用户点击麦克风并产生最终非空转写
- **THEN** 部分转写只显示在问题框，最终转写自动复用当前附件提交
- **AND** 系统只产生一条selection输入，不另行产生voice输入或要求再次点击发送

#### Scenario: 无截图语音提交

- **WHEN** 当前确认卡因小选区或权限缺失没有截图，用户完成语音转写
- **THEN** 系统以selection来源自动提交一条无附件输入

#### Scenario: 圈选会话结束时清场

- **WHEN** 用户关闭、取消、重新圈选，或圈选会话超时
- **THEN** 系统立即取消本轮region语音采集并丢弃未完成转写
- **AND** 后续最终事件不得提交给普通voice或旧PreviewSession

#### Scenario: 稳定提交结果

- **WHEN** 最终转写提交返回accepted、rejected或unknown
- **THEN** 确认卡复用既有selection提交反馈与附件清理语义
- **AND** unknown不得自动重试
