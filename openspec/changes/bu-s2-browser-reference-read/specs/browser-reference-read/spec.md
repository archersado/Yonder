# Browser引用只读详情 Delta Spec

## ADDED Requirements

### Requirement：授权读取Browser引用

系统 SHALL 只向可读取目标任务的身份返回AD-BU-01已提交引用；引用缺失与任务不存在必须稳定区分。

#### Scenario：任务存在且已有引用

- **WHEN** 授权调用方请求`task.browser.get`
- **THEN** 返回现有引用、所有权、托管页数、完成状态和更新时间序号
- **AND** 不启动ego-lite、不改变控制权或任务状态

#### Scenario：用户打开活动Browser Task Space

- **WHEN** 用户在任务详情点击“打开 ego-lite”
- **THEN** 系统按任务当前引用调用ego-lite `handOff()`进入对应Task Space
- **AND** 动作完成Observe并更新任务事件与引用所有权

#### Scenario：引用已结束或外部空间不存在

- **WHEN** 用户尝试打开已结束、缺失或不可用的Browser引用
- **THEN** 系统保留任务详情并显示明确失败
- **AND** 不创建替代Task Space

#### Scenario：引用缺失或读取失败

- **WHEN** 任务没有Browser引用或局部读取失败
- **THEN** Task Space保留任务基础详情并显示明确状态
- **AND** 不显示打开按钮，不以CUA、Command或新Task Space兜底
