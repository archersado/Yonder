# Jev 配置界面增量规格

## MODIFIED Requirements

### Requirement: 最小配置模型

系统 MUST 将受限模型标识作为非敏感 Jev 配置字段，并 MUST 允许经校验的本地 HTTP/HTTPS base URL进入既有 Jev Adapter。远端模式仍 MUST 只允许既有 TypeSafe HTTPS 端点。

#### Scenario: 保存有效配置

- WHEN 用户提交有效配置
- THEN 系统校验并保存单行配置
- AND 返回当前配置

#### Scenario: 拒绝无效配置

- WHEN 用户提交非法 URL、模型标识、超出范围的数字或未支持能力
- THEN 系统拒绝保存
- AND 给出字段级错误

#### Scenario: 本地模型用于两类决策

- WHEN 用户保存本地端点和模型标识并启动新的计划片段
- THEN 候选选择和动作后步骤核验使用同一配置快照中的模型标识
- AND Worker 不得硬编码、推断或覆盖该值

#### Scenario: 拒绝非法模型标识

- WHEN 模型为空、超过 128 字节或包含受限字符集之外的字符
- THEN Application 拒绝保存
- AND 设置窗口显示模型字段错误

#### Scenario: 升级已有配置

- WHEN 已持久化 JSON 不包含模型字段
- THEN 系统使用 `jev-latest` 兼容读取
- AND 不修改任务、事件或 Outbox

#### Scenario: 配置本地模型

- WHEN 用户选择本地服务并提交合法 base URL 与模型标识
- THEN 系统保存二者并在候选选择和步骤核验中使用同一配置快照
- AND Adapter 或 Worker 不得把模型覆盖为固定值

#### Scenario: 兼容既有配置

- WHEN SQLite 中的既有 Jev 配置尚未包含模型字段
- THEN 系统以 `jev-latest` 安全读取该配置
- AND 用户可在设置面板改为服务目录实际发布的模型
