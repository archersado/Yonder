# CUA Driver 对照增量规格

## ADDED Requirements

### Requirement: 无 Agent 对照

Harness SHALL 直接调用候选 Driver/SDK，不得加载 Agent、模型或 Planner。

#### Scenario: 执行固定动作

- **GIVEN** 两候选运行在相同设备与应用状态
- **WHEN** Harness 提交相同语义动作与预期条件
- **THEN** 保存每个候选的原始结果、耗时与验证结论

### Requirement: 故障语义验证

Harness SHALL 验证取消、超时、进程崩溃和结果未知状态。

#### Scenario: 动作中 Driver 退出

- **WHEN** Driver 在副作用动作完成确认前退出
- **THEN** Harness 将结果记录为 unknown
- **AND** 不得自动重复动作

### Requirement: 唯一选型

Spike SHALL 依据预先声明的淘汰门槛输出唯一结论。

#### Scenario: 候选未通过首版 Windows 核心用例

- **WHEN** 候选无法在 Windows 安全完成首版核心用例
- **THEN** 淘汰该候选
- **AND** 不得以双栈进入产品实现

### Requirement: 平台范围延期

E0-S2 SHALL 不以 macOS 和 Microsoft Office 阻塞首版 Windows CUA Driver 决策。

#### Scenario: 首版范围验证完成

- **GIVEN** WPS 代表办公套件场景
- **WHEN** Windows 的普通权限输入、文件资源管理器、故障语义与崩溃恢复均已验证
- **THEN** 允许完成 E0-S2
- **AND** macOS 对等验证进入后续 Epic
