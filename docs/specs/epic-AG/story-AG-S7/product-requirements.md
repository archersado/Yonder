# AG-S7 产品需求

## 问题与目标

AG-S5 已证明显式共享 App Server 模式可以把 Yonder 交回输入投递到同一 Codex thread，但当前研发接线要求用户分别启动 App Server、remote TUI 和 `agent-bridge`，并手工传入 thread ID 与 Socket。Yonder 或 Codex 任一侧重启后，桥会退出且不会自动恢复；普通 MCP 恢复只能说明任务通道可用，不能说明慢脑回注可用。

目标产品体验是：用户正常启动 Yonder 与受支持的 Codex 客户端后，当前会话明确绑定，连接器自动受监管运行并在安全边界重连；用户不接触 thread ID、Socket、环境变量或终端守护命令。

## 范围与非目标

本 Story 只验证当前会话绑定、连接器生命周期、重启恢复、客户端能力边界与用户可见状态，并输出终局 ADR 和后续产品 Story。它不交付守护进程、安装项或新的生产连接协议。

### 非目标

- 不在本 Spike 实现守护进程、安装项或新协议。
- 不为 ChatGPT Desktop 使用私有 API、进程注入、内部数据库或未公开队列。
- 不让 Yonder 启动 Codex、替代当前 thread 或创建第二规划 Agent。
- 不把单次手工命令成功当作可发布生命周期证据。

## 原始需求、用户变更与架构约束

- 原始需求：产品简报“Yonder 是 Agent Tool”“Agent Skill + 本地主机接入”，要求 Agent 接入是产品能力而非研发脚本。
- 既有规格：AG-S5 AG5-01、03、08～11，要求明确会话、accepted、确定性路由及任务/输入连接态分离。
- 后续用户变更（2026-10-10）：“以后每次启动都这么麻烦么”“这个是不是 Codex 造成的而不是 Yonder”“把这个记成一个 spec 待开发”。
- 架构约束：Yonder 不负责慢脑规划，不启动第二 Agent，不从最近会话、进程或内部数据库猜测目标；本地接入仍走私有 UDS/Named Pipe，协议由 Rust 单一生成。

## 验收条件

以下条件是本 Spike 的验收条件：

- AG7-01：建立 Codex CLI shared App Server、ChatGPT Desktop Codex 与 Windows/macOS 的能力矩阵，区分官方支持、实验支持和不可用，不把 CLI 结论外推到 Desktop。
- AG7-02：至少验证一种由当前 Codex 会话主动提供 `session_id/thread_id + endpoint` 的可信绑定方式；不得使用“最近会话”、工作目录或单一候选猜测。
- AG7-03：验证 Yonder GUI 重启、Gateway/MCP 重启、连接器崩溃与 Codex 重启四类场景；连接器采用有界退避恢复，旧会话失效后不得投递到新会话。
- AG7-04：验证任务 Gateway 与输入/replan 通道分别呈现连接状态；任一通道恢复不得误报另一通道可用。
- AG7-05：验证断线中的 `agent.input` 结果为 unknown/失败关闭，不自动重试正文，不重放任务副作用，不持久化正文、Socket 或敏感会话载荷。
- AG7-06：验证生命周期托管不启动第二 Agent、不读取 Codex 内部数据库、不修改 Codex 私有队列，也不要求 Yonder Desktop 推断 Agent 厂商。
- AG7-07：输出 Accepted/Rejected/Deferred 路线、进程所有者、重连状态机、安装启动方式、失败矩阵和后续产品 Story 的完整验收映射。

## 目标产品验收（供后续实施 Story 使用）

- 用户只启动 Yonder 与 Codex；受支持客户端的当前会话自动注册，无需终端命令或环境变量。
- Yonder 或连接器重启后自动恢复到同一明确会话；会话切换时旧绑定立即失效，新会话必须主动注册。
- 无受支持接口的客户端明确显示“当前 Codex 客户端不支持慢脑回注”，但任务 Gateway 仍可独立使用。

