# codex-cli-managed-connector Specification

## Purpose
定义 macOS Codex CLI 受信 SessionStart Hook 托管连接器的生命周期、同会话输入与附件交付、Yonder 重启恢复及不重放边界，使用户无需手填 thread ID、Socket 或单独运行桥接进程。

## Requirements

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

连接器 MUST 使用 Codex 公开共享 daemon，并在 `app-server proxy` 原始双向字节流上建立 WebSocket App Server RPC；不得自建/发现私有 endpoint、把 proxy 当作 JSONL，或另起与当前 active writer 冲突的 App Server。

#### Scenario: 已加载会话建立回复订阅

- **WHEN** Hook 提供的 thread 已在共享 daemon 中加载
- **THEN** 连接器仍通过同一 proxy 调用 `thread/resume(excludeTurns=true)`
- **AND** 取得同 ID 回执后才声明输入与回复能力
- **AND** 当前客户端可收到该 thread 的有界回复通知
- **AND** 不另起 App Server、不读取历史 transcript

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

### Requirement: Codex 会话附件输入

macOS Codex CLI 连接器 MUST 在同一已绑定 AgentSession 上声明并实现 `user_input_attachment`，且只把完整校验通过的单张圈选图片与文字投递到同一 Codex thread。

#### Scenario: 圈选附件成功

- **WHEN** 当前 session 收到合法 begin、连续 chunk、finish 和引用相同附件的 `agent.input`
- **THEN** 连接器复核 session、MIME、大小、顺序、SHA-256 与 deadline
- **AND** 使用同一个 `turn/start` 或 `turn/steer` 输入投递文字和内存图片
- **AND** Codex 确认接收后才返回 accepted，并立即释放附件

#### Scenario: 附件无效或投递未知

- **WHEN** 附件超限、乱序、哈希错误、跨 session、过期、重复声明、Codex 拒绝或任一连接断开
- **THEN** 连接器拒绝或返回 unknown 并清空本次附件
- **AND** 不落盘、不记录正文/图片/摘要、不自动重试
- **AND** 不降级成只发送文字

### Requirement: 不支持面清晰

产品 MUST 不把 macOS Codex CLI 能力外推到 ChatGPT Desktop 或 Windows。

#### Scenario: 不支持客户端或平台

- **WHEN** 用户尝试从 ChatGPT Desktop 或 Windows 使用该连接器
- **THEN** 状态为 Unsupported 或 Deferred
- **AND** 不使用私有 stdio、进程注入或 macOS 结论替代验证
