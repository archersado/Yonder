# BUA Bridge 延期增量规格

## ADDED Requirements

### Requirement: Windows 首版能力不可用

Windows 首版 SHALL 不宣称或隐式实现 BUA。

#### Scenario: Agent 请求 BUA

- **WHEN** Agent 在没有 ego-lite Runtime 的设备上请求 BUA
- **THEN** Yonder 返回 `capability_unavailable`
- **AND** 不得自动改走 CUA 或其他浏览器自动化引擎

### Requirement: 保持外部 Task Space 边界

未来恢复 BUA 时，Yonder SHALL 直接使用 ego-lite Task Space。

#### Scenario: 恢复平台支持

- **WHEN** 目标平台存在可验证的 ego-lite Runtime
- **THEN** BUA Bridge 保存 `external_task_ref` 并映射状态
- **AND** Yonder 不复制 Task Space
