# Jev Config Interface Delta

## ADDED Requirements

### Requirement: 最小配置模型

系统 MUST 提供仅含非敏感字段的 Jev 配置模型，并 MUST 在保存前校验字段和范围。

#### Scenario: 保存有效配置

- WHEN 用户提交有效配置
- THEN 系统校验并保存单行配置
- AND 返回当前配置

#### Scenario: 拒绝无效配置

- WHEN 用户提交非法 URL、超出范围的数字或未支持能力
- THEN 系统拒绝保存
- AND 给出字段级错误

### Requirement: 本机配置界面

系统 MUST 提供独立的 Jev 配置窗口，仅通过本机命令读写 Jev 配置，不得直接访问 Adapter、配置文件或任务状态，且不得嵌入 Task Space。

#### Scenario: 读取配置

- WHEN 用户从系统托盘菜单打开 Jev 设置窗口
- THEN 界面显示当前保存配置
- AND 显示置信阈值与敏感操作闸的只读安全说明

#### Scenario: 保存配置

- WHEN 用户提交配置
- THEN 界面调用本机保存命令
- AND 显示保存成功、配置无效或模型未启用状态

### Requirement: 执行门禁不变

本配置 MUST NOT 授权 Jev 执行；执行接线仍 MUST 受 AD-EX-02 双平台验证门禁约束。

#### Scenario: 配置保存后不执行

- WHEN 用户保存有效配置
- THEN 系统不调用 Jev、不创建任务、不修改任务状态
- AND 不写事件或 Outbox
