# Gateway查询增量

关联AG-S1 AC1–5，Architecture Impact：conforming。

## ADDED Requirements

### Requirement: 正式宿主保持连接门禁

可信会话查询 MUST 经GatewaySession处理并从TaskHost唯一SQLite读取；不能借用LocalUser权限。

#### Scenario: 未握手
- WHEN 新会话请求task.list
- THEN 返回-32002，不返回任务。

#### Scenario: 归属隔离
- GIVEN 已恢复真实任务库中两个Agent任务
- WHEN 两个已握手会话分别读取
- THEN 每个仅收到所属任务，越权get返回-32004。

#### Scenario: 重连
- WHEN 同一可信身份建立新会话
- THEN 不继承已握手标记。

#### Scenario: 编码
- WHEN 成功或失败响应返回宿主
- THEN 复用Rust协议encode，不手写第二套JSON响应。
