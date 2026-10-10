# AG-S8 架构设计

## 边界与依赖

本 Story 只扩展 `apps/yonder-cli` 的 Codex 连接器生命周期，复用既有 Agent Gateway 与协议 1.33，不新增持久化。依赖方向保持 `CLI → protocol`；Codex CLI 是连接器监管的外部公开进程接口，不进入 Rust crate 依赖图。

## 生命周期

```text
Codex CLI SessionStart Hook
  └─ stdin: { session_id, source, ... }
      └─ yonder codex-hook
          ├─ codex app-server --listen stdio://（子进程）
          ├─ initialize / initialized（逐行 JSON）
          ├─ thread/resume(session_id, excludeTurns=true)
          └─ 既有 Agent Gateway hello(session_id, UserInput + ReplanInput)
```

Hook 配置只匹配 `startup|resume` 并以后台方式执行。`compact` 不创建重复桥；会话结束后由 Codex Hook 生命周期终止连接器。连接器不保存 thread ID、输入正文或 endpoint。

## 状态与契约

### 所有权

- Codex：当前 session 标识、thread 与 Hook 生命周期。
- Yonder CLI 连接器：stdio App Server 子进程、同一 session 的 Gateway 注册、输入去重、有界退避重连。
- Yonder Desktop：统一 AgentSession 与任务输入路由；不启动 Codex、不读取 Codex 私有状态。

没有新增状态所有者、wire 字段或持久化。Gateway 仍是外部慢脑初始计划和 replan 的唯一入口。

## 失败与验证

- Hook 输入无效或来源不是 startup/resume：失败关闭，不选择其他 session。
- Yonder 不可达/断开：以封顶退避在固定恢复窗口重连相同 session；窗口耗尽后退出，由下一次显式 SessionStart 恢复。
- Codex App Server 断开：当前输入视为未确认且加入去重终态，不自动重投；连接器退出，下一次显式 SessionStart 才可恢复相同 session。
- thread 无法恢复或标识改变：旧连接器退出；新 session 只能由新 Hook 显式注册。
- Hook 输出、stderr 与 Yonder 日志不得包含正文、完整 session ID 或完整 Agent Payload。

### 安全约束

- 仅调用 Codex CLI 公开命令和 App Server v2 RPC；不读取 transcript、rollout 或内部数据库。
- 消息正文只通过受控 stdin/UDS 内存帧传输；禁止使用 `codex queue --message` 参数。
- `apps/yonder-cli` 继续只依赖 `crates/protocol`；实现不改变 Cargo 方向。

### 验证层级

- Rust 单元：Hook 输入、来源、去重、退避边界。
- 合约：伪造 stdio App Server 与 Gateway，验证启动参数、initialize、resume、start/steer、断开重连、不重放。
- macOS 原生：真实 Codex CLI session 启动、Yonder 重启、桥退出、session 切换结构化日志。
- Windows 与 ChatGPT Desktop 不纳入通过条件，但必须保留 Deferred/Unsupported 说明。
