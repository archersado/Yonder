# Computer Observation Delta Specification

## ADDED Requirements

### Requirement: 窗口目标必须元素优先且只在歧义时返回视觉证据

系统 MUST 在窗口级CUA动作前读取同一可信窗口的新鲜AX/UIA元素。目标可唯一解析时 MUST 直接使用元素且不得为定位额外截图；目标缺失或不唯一时 MUST 在副作用尚未派发的前提下，至多采集一次同窗口临时截图并交回归属慢脑。

#### Scenario: 唯一元素可解析

- **WHEN** 当前可信窗口存在唯一可操作目标元素
- **THEN** Worker使用该元素派发已授权动作，并以同一窗口Observe结果验证，不触发视觉定位截图

#### Scenario: 元素缺失或不唯一

- **WHEN** 当前可信窗口没有目标元素或存在多个无法消歧的目标元素
- **THEN** Worker不派发原动作，返回已知拒绝和一次同窗口临时Observation，计划停止并交回慢脑

### Requirement: 临时Observation不得成为持久化或成功事实

系统 MUST 只在协议1.34计划执行响应中向归属慢脑返回临时Observation。截图 MUST NOT 写入计划、SQLite、事件、Outbox、日志或Jev请求；`unknown`动作即使带有效Observation也不得被提升为成功或自动重试。

#### Scenario: 副作用结果未知但后置Observe有效

- **WHEN** Driver无法确认动作效果但同次后置窗口Observe成功
- **THEN** attempt保持`unknown`并交回，响应可携带临时Observation供核实，系统不重放该动作

#### Scenario: 旧协议客户端

- **WHEN** 调用方协商协议1.31～1.33
- **THEN** 仍获得原有计划交回结果，不接收1.34新增Observation字段

### Requirement: 后台窗口Observe不得退回当前桌面

窗口级动作及其前后Observe MUST 绑定同一任务已验证并刷新后的应用PID/window；只有显式desktop scope动作可以观察全桌面。

#### Scenario: 目标应用不在前台

- **WHEN** `launch_app`取得可信应用身份而用户继续操作其他前台应用
- **THEN** 后续元素解析、动作和视觉证据仍来自目标应用窗口，且不隐式执行`bring_to_front`
