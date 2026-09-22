# Region Preview Delta

## ADDED Requirements

### Requirement: 应用切换必须结束可见圈选会话

系统 MUST 在macOS可见圈选层或确认卡失去焦点时结束本轮圈选会话。

#### Scenario: 选择中切换应用

- **WHEN** 圈选层可见且用户切换到另一应用
- **THEN** 系统取消本轮语音、清除临时选区和截图并隐藏圈选窗口
- **AND** 不发送Agent输入或创建任务

#### Scenario: 确认卡中切换应用

- **WHEN** 确认卡可见且用户切换到另一应用
- **THEN** 系统清除问题、来源应用和临时截图并隐藏确认卡

#### Scenario: 主动隐藏用于截图

- **WHEN** 系统为截图主动隐藏圈选窗口
- **THEN** 随后的失焦不得清除正在进行的圈选会话
- **AND** 成功截图后仍可显示确认卡
