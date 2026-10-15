# AD-AG-10 Codex 连接器生命周期所有权

状态：Accepted（macOS Codex CLI 基线）；Deferred（Windows、ChatGPT Desktop）
日期：2026-10-10  
Architecture Impact：architecture-change（本地 Agent 连接器进程所有权、会话绑定与重连生命周期）  
关联：AG-S1、AG-S5、AG-S7、AD-AG-05、AD-AG-07、AD-AG-08

## 决策

AG-S5 已验证 Codex CLI 0.156.1 的共享 App Server + 显式 thread 绑定可以向同一会话投递 `agent.input/replan`。但当前研发接线要求用户手工启动 App Server、remote TUI 与薄桥；Yonder 重启后薄桥退出且无人恢复。ChatGPT Desktop 当前使用宿主私有 stdio App Server，尚无证据证明外部连接器可受支持地附着到当前 thread。

2026-10-10 主人接受先交付 macOS Codex CLI、ChatGPT Desktop 明确不支持、Windows 继续挂起。macOS 产品基线采用 Codex 公开生命周期 Hook 与共享 App Server：

- Codex `SessionStart` command hook 是当前 session 标识的唯一来源；Yonder 不按最近会话、工作目录、进程或本地数据库推断。
- Yonder CLI 连接器幂等启动 `codex app-server daemon`，再通过 `codex app-server proxy` 的原始双向字节流建立 WebSocket App Server 客户端并恢复同一 thread；不创建新的规划 Agent，不依赖自建 Socket。
- Hook 以后台方式运行连接器；Codex 会话结束时由 Hook 生命周期终止连接器。`startup|resume` 才建立绑定，`compact` 不重复启动。
- 连接器拥有 proxy 子进程、Gateway 注册和有界退避重连；Yonder Desktop 仍只消费统一 `AgentSession`，不拥有 Codex 进程或会话。
- Yonder 断开后只重新注册相同 session；已接收但未确认的输入不得重放，任务副作用更不得重放。
- Hook 安装与信任是一次性显式设置。未安装、未信任、Codex 版本不兼容、连接器退出和 Gateway 不可达必须呈现为未连接，不能误报已连接。

## 边界

采用“会话宿主显式发布绑定，厂商连接器监管桥，Yonder 消费统一 AgentSession”的边界：

- Codex 会话宿主通过受支持 Hook 提供当前 thread/session 标识；共享 endpoint 由公开 daemon/proxy 管理，Yonder 不发现或推断 endpoint/session。
- Codex 连接器拥有 App Server 兼容性、start/steer 映射、桥进程监管与有界重连；它不是规划 Agent。
- Yonder Desktop 不启动 Codex，不读取 Codex 内部数据库/队列，不附着私有 stdio；只接受显式注册的统一 AgentSession。
- MCP 任务连接与输入/replan 连接分别注册和展示；任一通道恢复不代表另一通道恢复。
- 重连不重放未确认输入或任何任务副作用；旧 thread 变化后永久作废，等待新会话主动注册。

## 平台结论与限制

- macOS Codex CLI 0.162.1：Accepted。公开 Hook 输入含 `session_id`；daemon、WebSocket-over-proxy、`thread/resume`、active turn steer 与 idle turn start 构成受支持接线。
- ChatGPT Desktop：Unsupported/Deferred。当前没有可供外部 Yonder 连接器附着当前会话的公开 endpoint；不得注入宿主私有 stdio。
- Windows：Deferred。按主人决定不以缺少 Windows 对照阻断 macOS 子范围，但不得宣称跨平台完成。
- AG-S7 保留未完成的 Windows 与跨平台终局验证；AG-S8 只实现并验证本决定接受的 macOS 子范围。

## 拒绝路线

- Yonder 按最近会话、工作目录、进程或单一候选猜测 thread；
- 读取 Codex 内部数据库、私有队列或注入 ChatGPT Desktop 私有 stdio；
- Yonder 启动新的 Codex Agent 代替原会话；
- 让 MCP 单向任务连接冒充双向输入连接；
- 用无限重试隐藏客户端不支持或绑定已经失效。
- 把输入正文放入 `codex queue --message` 命令行参数，造成进程列表或诊断工具泄露正文。

## 2026-10-15 单活抢占退出语义增量

AG-S13 真实验证发现，仅关闭旧 Socket 会被 Codex Connector 按“Yonder 重启”处理并自动重连，导致旧会话与新会话持续互相抢占。接受以下修订：

- Yonder 主动抢占旧 AgentSession 时，先通过既有本地 Agent 连接发送有界 `agent.session.close(reason=preempted)` 帧，再关闭连接。
- Connector 收到 `preempted` 必须终止当前绑定进程，不得进入 Gateway 重连循环；只有连接拒绝、重置、EOF、超时等传输故障继续使用既有有界重连。
- 关闭帧只包含当前 `session_id` 与封闭原因，不包含输入、回复、任务或 Agent Payload，不写入 SQLite、事件、Outbox 或日志。
- 若输入已经发送但尚未收到接受回执时发生抢占，结果仍是 `unknown`，不得自动重放；关闭帧不能伪造未送达事实。
- 把 `codex app-server proxy` 当作逐行 JSON App Server。该命令只转发 daemon 控制 Socket 的原始字节，客户端必须在其上完成 WebSocket 握手与帧协议。
- 为当前仍由共享 daemon 持有 active writer 的会话另起 `app-server --listen stdio://`；第二个 App Server 会因 writer 冲突拒绝 `thread/resume`。
