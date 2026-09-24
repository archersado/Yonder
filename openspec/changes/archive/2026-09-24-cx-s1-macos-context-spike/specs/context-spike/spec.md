# macOS Context Spike Delta

## ADDED Requirements

### Requirement: 事件驱动原生元数据验证

macOS Spike MUST 使用系统事件验证当前应用与窗口元数据能力，并且不得保存标题或控件正文。

#### Scenario: 应用或焦点窗口变化

- **WHEN** 验证者显式启动样本并切换前台应用或窗口
- **THEN** Spike只记录应用标识/PID有效性、窗口属性可用性和事件计数
- **AND** 停止后移除Workspace通知与AXObserver资源

#### Scenario: Accessibility未授权

- **WHEN** 当前进程没有Accessibility权限
- **THEN** Spike返回`capability_unavailable`
- **AND** 不轮询窗口、不截图、不自动请求扩大权限

### Requirement: 有界Native Messaging验证

macOS Native Host MUST 校验本机字节序长度头、有界JSON和精确扩展来源，并保持stdout协议隔离。

#### Scenario: 分片和连续消息

- **WHEN** Host接收UTF-8分片输入或连续有效帧
- **THEN** 每个完整帧只产生一个对应协议响应
- **AND** 响应和诊断均不包含URL、标题或完整Payload

#### Scenario: 非法或超限消息

- **WHEN** 长度超过1 MiB、JSON非法或消息不完整
- **THEN** Host稳定拒绝并退出
- **AND** 不分配无界内存、不继续处理后续输入

### Requirement: 浏览隐私与产品门禁

扩展 MUST 不申请History权限并在发送前拒绝隐私窗口；Spike通过不得直接开放产品采集。

#### Scenario: 隐私窗口事件

- **WHEN** 扩展观察到`tab.incognito=true`
- **THEN** 不向Native Host发送该事件

#### Scenario: 单浏览器子范围通过

- **WHEN** Google Chrome实连通过而Microsoft Edge未安装或未验证
- **THEN** 证据只标记Chrome子范围通过
- **AND** 产品Context Port、Recording、SQLite、索引与同步保持disabled
