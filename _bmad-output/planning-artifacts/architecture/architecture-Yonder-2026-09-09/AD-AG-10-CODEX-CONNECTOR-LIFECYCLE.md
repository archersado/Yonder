# AD-AG-10 Codex 连接器生命周期所有权

状态：Proposed（等待 AG-S7 Spike）  
日期：2026-10-10  
Architecture Impact：architecture-change（本地 Agent 连接器进程所有权、会话绑定与重连生命周期）  
关联：AG-S1、AG-S5、AG-S7、AD-AG-05、AD-AG-07、AD-AG-08

## 待决问题

AG-S5 已验证 Codex CLI 0.156.1 的共享 App Server + 显式 thread 绑定可以向同一会话投递 `agent.input/replan`。但当前研发接线要求用户手工启动 App Server、remote TUI 与薄桥；Yonder 重启后薄桥退出且无人恢复。ChatGPT Desktop 当前使用宿主私有 stdio App Server，尚无证据证明外部连接器可受支持地附着到当前 thread。

需要决定谁拥有 App Server、thread 绑定、薄桥监管与重连，以及如何在不猜测会话、不启动第二 Agent 的前提下提供自然启动体验。

## 候选决策

建议采用“会话宿主显式发布绑定，厂商连接器监管桥，Yonder 消费统一 AgentSession”的边界：

- Codex 会话宿主或受支持扩展提供当前 thread/session 标识和 endpoint；Yonder 不发现或推断。
- Codex 连接器拥有 App Server 兼容性、start/steer 映射、桥进程监管与有界重连；它不是规划 Agent。
- Yonder Desktop 不启动 Codex，不读取 Codex 内部数据库/队列，不附着私有 stdio；只接受显式注册的统一 AgentSession。
- MCP 任务连接与输入/replan 连接分别注册和展示；任一通道恢复不代表另一通道恢复。
- 重连不重放未确认输入或任何任务副作用；旧 thread 变化后永久作废，等待新会话主动注册。

## 当前限制

本决定保持 Proposed，不授权实施。AG-S7 必须先验证：

1. Codex CLI 冷启动、Yonder 重启、桥崩溃、Codex 重启和 thread 切换；
2. ChatGPT Desktop 是否存在公开支持的当前会话连接器；若无则明确 Unsupported；
3. macOS/Windows 对照及相同失败矩阵；
4. 不读取私有状态、不猜测会话、不启动第二 Agent、不持久化输入正文；
5. 自动恢复不会误报连接或重放 unknown 输入。

## 拒绝路线

- Yonder 按最近会话、工作目录、进程或单一候选猜测 thread；
- 读取 Codex 内部数据库、私有队列或注入 ChatGPT Desktop 私有 stdio；
- Yonder 启动新的 Codex Agent 代替原会话；
- 让 MCP 单向任务连接冒充双向输入连接；
- 用无限重试隐藏客户端不支持或绑定已经失效。
