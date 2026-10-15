# AG-S8 架构设计

## 边界与依赖

本 Story 只扩展 `apps/yonder-cli` 的 Codex 连接器生命周期，复用既有 Agent Gateway 与协议 1.33，不新增持久化。依赖方向保持 `CLI → protocol`；Codex CLI 是连接器监管的外部公开进程接口，不进入 Rust crate 依赖图。

## 生命周期

```text
Codex CLI SessionStart Hook
  └─ stdin: { session_id, source, ... }
      └─ yonder codex-hook
          ├─ codex app-server daemon start（幂等）
          ├─ codex app-server proxy（原始双向字节流）
          ├─ WebSocket Upgrade → initialize / initialized
          ├─ thread/loaded/list （兼容性诊断）
          ├─ thread/resume(session_id, excludeTurns=true) → 复用 thread 并订阅回复通知
          └─ 既有 Agent Gateway hello(session_id, UserInput + UserInputAttachment + ReplanInput)
```

Hook 配置只匹配 `startup|resume` 并以后台方式执行。`compact` 不创建重复桥；会话结束后由 Codex Hook 生命周期终止连接器。连接器不保存 thread ID、输入正文或 endpoint。

Hook 未显式提供 `YONDER_AGENT_ID` 时使用与正式 MCP 配置一致的 `codex-cli`；显式环境变量仍可覆盖。这样任务通道与输入通道进入同一已登记 Agent 身份，不要求用户为 Hook 重复配置环境变量。

## 状态与契约

### 所有权

- Codex：当前 session 标识、共享 App Server daemon、thread 与 Hook 生命周期。
- Yonder CLI 连接器：WebSocket proxy 子进程、同一 session 的 Gateway 注册、输入去重、有界退避重连。
- Yonder Desktop：统一 AgentSession 与任务输入路由；不启动 Codex、不读取 Codex 私有状态。

没有新增状态所有者、wire 字段或持久化。Gateway 仍是外部慢脑初始计划和 replan 的唯一入口。

### 圈选附件

连接器复用协议 1.33 已存在的 `agent.attachment.begin/chunk/finish` 与 `agent.input.attachment_id`。每个 Gateway 连接只允许一个本会话附件处于组装或待消费状态；连接器按序解码分块，复核声明长度、SHA-256、MIME、deadline 与 session，再把文字和内存 data URL 组成同一次 Codex App Server `turn/start` 或 `turn/steer` 输入。Codex 确认接收后才向 Yonder 返回 accepted。

附件字节不得落盘、进入命令行、日志或错误文本。任何拒绝、超时、App Server 断开、Yonder 断开或输入消费后都立即释放附件；输入 ID 去重仍覆盖最终投递，避免同一截图重复进入 Codex。

## 失败与验证

- Hook 输入无效或来源不是 startup/resume：失败关闭，不选择其他 session。
- Yonder 不可达/断开：以封顶退避在固定恢复窗口重连相同 session；窗口耗尽后退出，由下一次显式 SessionStart 恢复。
- Codex proxy 断开：当前输入视为未确认且加入去重终态，不自动重投；连接器退出，下一次显式 SessionStart 才可恢复相同 session。
- `thread/loaded/list` 不代表当前 App Server 客户端已订阅通知；无论 thread 是否已加载，连接器都必须经共享 daemon/proxy 调用 `thread/resume(excludeTurns=true)` 建立订阅。不得因此另起独立 App Server。
- 附件声明、分块、完整性或会话校验失败：拒绝本次附件并清理；不得继续发送纯文字形成伪成功。
- thread 无法恢复或标识改变：旧连接器退出；新 session 只能由新 Hook 显式注册。
- Hook 输出、stderr 与 Yonder 日志不得包含正文、完整 session ID 或完整 Agent Payload。
- App Server 诊断只允许暴露失败 RPC 阶段（初始化、已加载查询、恢复/订阅），不输出请求参数、session ID 或服务端原始错误载荷。

### 安全约束

- 仅调用 Codex CLI 公开命令和 App Server v2 RPC；不读取 transcript、rollout 或内部数据库。
- 消息正文只通过受控 stdin/UDS 内存帧传输；禁止使用 `codex queue --message` 参数。
- `apps/yonder-cli` 继续只依赖 `crates/protocol`；实现不改变 Cargo 方向。

### 验证层级

- Rust 单元：Hook 输入、来源、去重、退避边界。
- 合约：验证 daemon/proxy 启动参数、双向字节适配和 WebSocket App Server RPC；再与 Gateway 验证 resume、start/steer、断开重连、不重放。
- 附件合约：验证 begin/chunk/finish、大小/哈希/顺序/会话/deadline、内存图片输入、消费后清理及拒绝时不投递正文。
- macOS 原生：真实 Codex CLI session 启动、Yonder 重启、桥退出、session 切换结构化日志。
- Windows 与 ChatGPT Desktop 不纳入通过条件，但必须保留 Deferred/Unsupported 说明。
