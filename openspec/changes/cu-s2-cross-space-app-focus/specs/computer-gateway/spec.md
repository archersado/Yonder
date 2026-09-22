# Computer Gateway Delta Spec

## ADDED Requirements

### Requirement: 应用启动与前置保持SDK动作原义

系统 MUST 保持SDK的`launch_app`与`bring_to_front`动作原义；跨Space前置能力由SDK和显式后续动作决定，Yonder不得改写目标或隐式扩展动作。

#### Scenario: 显式前置最近启动的应用

- **WHEN** 同一任务先成功调用`launch_app`，再显式调用`bring_to_front`
- **THEN** Yonder从前一步SDK返回值注入可信应用与窗口身份
- **AND** Agent不能提交PID或窗口号，`launch_app`不会被隐式改写为动作序列
- **AND** Observe可返回最近启动目标是否有窗口在屏幕上且未被SDK明确标为非当前Space的有界事实

#### Scenario: 跨Space前置被SDK拒绝

- **WHEN** SDK不能把目标窗口前置到当前Space
- **THEN** Yonder返回动作失败且不伪报用户可见
- **AND** Agent可显式组合SDK公布的Dock键盘动作并在每步后Observe
