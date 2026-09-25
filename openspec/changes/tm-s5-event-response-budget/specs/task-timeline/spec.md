## ADDED Requirements

### Requirement: 任务事件响应按编码字节有界

`task.events` SHALL 同时限制已投影单事件与完整 JSON RPC 响应的 UTF-8 字节数，并保留可按序继续读取的完整事件前缀。

#### Scenario: 多条事件到达整响应预算
- **WHEN** 下一条完整事件会使响应超过 256 KiB
- **THEN** 本页仅返回已容纳的前缀，下一次可从最后实际返回的序号继续读取该事件

#### Scenario: JSON 转义后超限
- **WHEN** 事件序列化后的字节数超过 8 KiB
- **THEN** 查询明确失败，不截断正文、不跳过该事件

#### Scenario: 首项无法容纳
- **WHEN** 首个事件不能放入完整成功响应预算
- **THEN** 查询明确失败，不返回空页并暗示历史结束

#### Scenario: 没有后续事件
- **WHEN** `after_sequence` 后无已提交事件
- **THEN** 返回正常空事件页，保留调用方原游标
