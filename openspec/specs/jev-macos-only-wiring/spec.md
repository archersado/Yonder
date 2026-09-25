# Jev macOS-only 接线规格

## Purpose

定义仅在 macOS 生效的 Jev 有界候选决策、系统 Keychain 凭据边界、固定远端端点与单次超时规则；该规格不授权任务动作派发、任务状态/事件/Outbox 写入或 Windows 注册。

## Requirements

### Requirement: macOS-only 有界候选决策

系统 MUST 只在 macOS 上提供 Jev 候选决策 Port，且每次请求 MUST 绑定任务、步骤与能力，并携带 2..10 个唯一候选。

#### Scenario: 请求缺少唯一 handback

- WHEN 候选少于 2 个、超过 10 个、存在重复或缺少唯一 `handback`
- THEN 系统拒绝调用 Jev
- AND 不修改任务状态、事件或 Outbox

#### Scenario: 低置信或候选不可派发

- WHEN Jev 置信度低于 0.75、返回未知候选或选择不可派发候选
- THEN 系统返回交回结果
- AND 不自动重试或直接派发动作

### Requirement: macOS 凭据与远端调用

系统 MUST 在 macOS 上通过系统凭据入口读取 Jev API Key，服务名为 `Yonder`、账号名为 `jev`；独立设置面板 MUST 允许用户以密码框一次性写入该凭据。API Key MUST NOT 进入 Jev 配置、SQLite、任务数据、日志、进程参数、环境变量或 UI 回显。

Jev 远端配置 MUST 使用 SDK base URL `https://api.typesafe.ai`，由固定 SDK 调用 `/v1/systemone`；其他远端地址或把 `/v1/systemone` 直接写入 base URL 的配置 MUST 被拒绝。

#### Scenario: 用户在设置页更新凭据

- **WHEN** 用户提交非空 API Key
- **THEN** 系统仅以原生 Keychain API 替换该凭据
- **AND** 后续读取接口只返回已配置状态，不返回密钥

#### Scenario: 用户保存空密码框

- **WHEN** API Key 密码框为空
- **THEN** 系统保留既有 Keychain 凭据
- **AND** 仅保存非敏感 Jev 配置

#### Scenario: 任务存储不可用时凭据入口仍可用

- **WHEN** `TaskHost` 或 SQLite 尚未就绪
- **THEN** 设置面板仍可查询和写入 macOS Keychain 凭据
- **AND** 非敏感 Jev 配置错误须单独反馈

#### Scenario: 凭据缺失

- WHEN 系统凭据入口没有可用 API Key
- THEN 系统返回凭据不可用错误
- AND 不调用远端模型

#### Scenario: 远端决策失败

- WHEN 远端返回错误、超时或无效响应
- THEN 系统返回可区分错误
- AND 不自动重试、不写任务状态、事件或 Outbox

#### Scenario: macOS 单次调用超时

- WHEN SDK 或 Worker 在 3000ms 内未完成
- THEN 系统终止本次 Worker 并返回超时
- AND 不自动或隐式发起第二次远端调用

### Requirement: Windows 后补

本 Change MUST NOT 在 Windows 上编译或注册 Jev 决策路径。

#### Scenario: Windows 构建

- WHEN 系统在 Windows 构建
- THEN Jev 决策 Adapter 不进入产物
- AND 系统不得宣称 Windows 接线或双平台验证完成
