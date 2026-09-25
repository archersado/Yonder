## ADDED Requirements

### Requirement: 已提交 Observe 事实按事件序号可检查

`task.events` SHALL 在协议 1.21 为已有可信 Observe 事件提供当时的步骤、验证结果和获准摘要，不从任务当前快照重建历史，也不改变旧协议投影。

#### Scenario: 多次 Observe 保留各自事实
- **WHEN** 同一任务先后提交两次 Observe，第二次更新当前详情
- **THEN** 历史页分别返回两条观察的原步骤、结论和摘要；第一条不被第二条覆盖

#### Scenario: 未知观察
- **WHEN** 一次已提交 Observe 的结论为 unknown
- **THEN** 时间线明确显示未知，不把动作成功或任务完成推断为匹配

#### Scenario: 旧版会话
- **WHEN** 客户端协商协议 1.20 或更低版本读取相同事件
- **THEN** 响应不包含新增的 `observation` 字段，原有事件字段与顺序不变

#### Scenario: 畸形历史与无权限读取
- **WHEN** 已授权任务的观察 payload 畸形，或调用方当前无读取权限
- **THEN** 前者明确读取失败、不伪造观察；后者先拒绝授权，不泄露历史内容
