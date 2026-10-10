# Codex CLI Managed Connector Delta

## ADDED Requirements

### Requirement: Hook 显式绑定当前会话

macOS Codex CLI 连接器 MUST 只使用受支持 Hook 提供的当前 session ID 建立绑定。

#### Scenario: 启动或恢复会话

- **WHEN** Codex `SessionStart` 以 `startup` 或 `resume` 提交合法事件
- **THEN** 连接器使用事件中的 `session_id` 恢复同一 thread
- **AND** 不要求用户填写 thread ID 或 Socket
- **AND** 不猜测最近会话或读取 Codex 私有状态

#### Scenario: 非法或重复生命周期

- **WHEN** Hook 事件缺字段、标识无效或来源不是 `startup|resume`
- **THEN** 连接器失败关闭
- **AND** 不选择其他 session

### Requirement: 公开 App Server 传输

连接器 MUST 使用 Codex 公开共享 daemon/proxy 与 App Server RPC，不自建或发现私有 endpoint。

#### Scenario: 投递输入

- **WHEN** Gateway 向已绑定 session 发送用户输入或 replan 输入
- **THEN** idle thread 使用 `turn/start`，active thread 使用 `turn/steer`
- **AND** 正文只通过内存/stdio/UDS 帧传输，不进入命令行参数或日志

### Requirement: 有界恢复且不重放

连接器 MUST 在 Yonder 重启后有界恢复同一显式 session，并阻止输入重放。

#### Scenario: Yonder UDS 断开

- **WHEN** 已注册连接断开
- **THEN** 连接器以有上限的退避和固定恢复窗口重新发送 hello
- **AND** 已确认输入 ID 继续去重
- **AND** 未确认输入不会重新提交给 Codex

#### Scenario: 恢复窗口耗尽

- **WHEN** Gateway 在恢复窗口内始终不可达
- **THEN** 连接器退出并报告未连接
- **AND** 不无限重试或误报已连接

### Requirement: 不支持面清晰

产品 MUST 不把 macOS Codex CLI 能力外推到 ChatGPT Desktop 或 Windows。

#### Scenario: 不支持客户端或平台

- **WHEN** 用户尝试从 ChatGPT Desktop 或 Windows 使用该连接器
- **THEN** 状态为 Unsupported 或 Deferred
- **AND** 不使用私有 stdio、进程注入或 macOS 结论替代验证
