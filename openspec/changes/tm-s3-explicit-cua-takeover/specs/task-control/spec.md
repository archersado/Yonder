# task-control Delta Specification

## MODIFIED Requirements

### Requirement: CUA 人工接管必须来自显式本机入口

系统在真实 CUA 执行区间必须显示与圈选工具条同屏、同顶部居中位置和同视觉样式的控制条；普通 HID 输入不得自动暂停。只有控制条“接管电脑”按钮可以登记本轮接管，已派发动作完成并 Observe 后才可提交暂停与定位。

#### Scenario: 普通输入不停止

- **WHEN** CUA 动作执行期间出现物理鼠标或键盘输入
- **THEN** Worker 继续执行且任务不产生新的 `unknown/user-input`

#### Scenario: 显式点击接管

- **WHEN** 用户在当前 CUA 控制卡点击“接管电脑”
- **THEN** 系统冻结后续动作、等待当前动作 Observe，并以最新任务 sequence 提交既有 takeover 控制

#### Scenario: 控制卡位置

- **WHEN** CUA 在多显示器环境开始执行
- **THEN** 控制条使用圈选交互相同的 pet 当前显示器与 work area，在该工作区顶部 16 逻辑像素处横向居中显示且不遮挡条外区域
