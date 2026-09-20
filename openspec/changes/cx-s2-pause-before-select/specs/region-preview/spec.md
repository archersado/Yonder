# Region Preview Delta

## ADDED Requirements

### Requirement: 圈选前暂停桌面任务

系统 MUST 在CUA持有桌面租约时先以可信本地用户身份暂停租约所属任务，并只在步骤边界停止事实提交后显示圈选层。

#### Scenario: 已观察步骤安全暂停

- **WHEN** 用户从小龙或托盘启动圈选，当前桌面任务的动作已返回且Observe有效
- **THEN** 系统提交`pause`、将任务原子转为paused并释放桌面租约
- **AND** 随后自动显示圈选层，不定位窗口、不启动Recording、不派发新CUA动作

#### Scenario: 停止结果不能确认

- **WHEN** 当前attempt为prepared、unknown，或身份、序号、存储提交失败
- **THEN** 系统保持桌面租约与任务事实，不显示圈选层
- **AND** 返回稳定的结果待核实反馈，不轮询数据库、不自动重试

#### Scenario: 当前没有桌面租约

- **WHEN** 用户启动圈选且没有任务持有桌面租约
- **THEN** 系统不创建控制记录，直接进入既有圈选流程
