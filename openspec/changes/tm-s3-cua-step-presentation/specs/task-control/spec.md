# task-control Delta Specification

## MODIFIED Requirements

### Requirement: CUA 人工接管必须来自显式本机入口

系统在真实 CUA 执行区间必须显示与圈选工具条同屏、同顶部居中位置和同视觉样式的控制条；普通 HID 输入不得自动暂停。只有控制条“接管电脑”按钮可以登记本轮接管，已派发动作完成并 Observe 后才可提交暂停与定位。控制条必须展示当前真实执行步骤；对已验证的计划片段，还必须展示有界规划步骤及完成/执行/待执行状态。

#### Scenario: 连续计划展示

- **WHEN** 已验证的 CUA 计划片段开始执行
- **THEN** 控制条显示当前真实派发步骤、至多四条计划槽位标签以及可辨的 completed/executing/pending 状态，不显示动作参数、正文或原始观察数据

#### Scenario: 单步或读取失败降级

- **WHEN** CUA 请求没有计划片段或计划投影暂时不可读取
- **THEN** 控制条仍显示当前步骤与诚实降级文案，不阻断 Driver 派发且不虚构后续步骤
