# execution-experience-recall Specification

## Purpose
定义已验证、非敏感执行经验的晋升与有界召回规则，使 Yonder 在不改变任务事实、慢脑规划权或运行时轻量边界的前提下，为 handback 后的重新规划提供安全策略提示。

## Requirements

### Requirement: 只保存已验证且非敏感的执行经验

系统 SHALL 只把动作后 Observe 成功且归属慢脑最终核验 achieved 的稳定策略保存为建议性经验。经验 MUST NOT 包含任务正文、用户输入、联系人、消息正文、截图、transcript、坐标、PID/window ID、元素索引、临时句柄或完整 Payload。

#### Scenario: 已核验的安全激活动作晋升

- **WHEN** 非敏感 CUA 激活动作已 Observe 成功，且归属慢脑提交最新目标 achieved
- **THEN** 系统按平台、Driver、应用、动作语义、控件角色与交互方式聚合一条经验
- **AND** 只保存通用前置条件与必须重新验证的成功条件

#### Scenario: unknown 或敏感动作不晋升

- **WHEN** 动作结果 unknown、尚未目标核验，或动作涉及输入、发送、坐标及其他敏感副作用
- **THEN** 系统不写入成功经验
- **AND** 不把动作回执提升为已验证成功

### Requirement: replan 前有界召回

系统 SHALL 在 handback 唤醒归属慢脑前，按当前稳定动作特征召回最多三条经验提示。召回结果 SHALL 经既有 Agent 输入通道交付，并明确要求慢脑先读取新鲜任务事实。

#### Scenario: 命中历史经验

- **WHEN** 当前 handback 存在匹配的已验证经验
- **THEN** replan 输入包含通用策略、适用前置与成功条件
- **AND** 慢脑仍须调用 `task.get` 与 `task.events` 后提交新片段

#### Scenario: 无命中或存储不可用

- **WHEN** 召回为空、存储忙或读取失败
- **THEN** 系统立即投递原 replan 输入
- **AND** 不阻塞 handback、不修改任务事实、不建立第二条 Agent 通道
