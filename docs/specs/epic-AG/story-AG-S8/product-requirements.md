# AG-S8 产品需求

## 问题与目标

当前 Codex 慢脑桥要求研发人员分别启动 App Server、remote TUI 和 `agent-bridge`，并复制 thread ID 与 Socket。Yonder 重启还会令桥退出，普通用户无法把它当作稳定产品能力。

## 需求来源与验收映射

- 原始需求：AG-S5 的 Agent 用户输入/replan 必须进入已绑定的同一会话，来源为产品主干的 Agent Gateway 与 2026-09-17 用户变更。
- 后续用户变更：2026-10-10 用户要求把“每次启动都需手工重绑”记录为规格并实施；随后明确接受“先交付 macOS Codex CLI、ChatGPT Desktop 不支持、Windows 挂起”。
- 架构约束：Accepted AD-AG-10、AD-AG-07、AD-VI-02；不得猜测会话、读取 Codex 私有状态、启动第二 Agent或建立第二条 Yonder Agent 通道。
- Spike 证据：AG-S7 证明 Codex CLI 0.162.1 公开 Hook 提供 `session_id`；后续真实探针与官方文档证明逐行 JSON 客户端应使用 `codex app-server --listen stdio://`，而非原始控制 Socket proxy。ChatGPT Desktop 未提供外部可附着的公开 endpoint。

## 范围与非目标

1. 提供可由 Codex `SessionStart(startup|resume)` 后台 Hook 调用的 Yonder CLI 入口。
2. 从 Hook stdin 读取并验证当前 `session_id`，不要求用户填写 thread ID 或 Socket。
3. 启动公开的 stdio App Server 子进程并恢复同一 thread，向既有 Yonder Gateway 注册 UserInput/ReplanInput。
4. Yonder 未启动或重启时进行有界退避重连；不重放未确认输入。
5. 提供可安装、可审阅的 Codex Hook 配置与明确诊断，不记录输入正文或完整会话标识。

### 非目标

- 不启动新的 Codex 规划 Agent；不附着 ChatGPT Desktop 私有 App Server。
- 不自动修改用户 Codex 全局配置或绕过 Hook 信任。
- 不实现 Windows；不新增 Yonder wire 协议、数据库表或任务状态。

## 验收条件

- AC1：给定合法 SessionStart Hook 输入，无需 thread/socket 环境变量即可注册同一 session。
- AC2：Yonder 在连接器运行期间重启，连接器在有界退避内重新注册；桥进程无需人工重启。
- AC3：重复输入 ID 不会向 Codex 重放；已接收但未确认的输入在传输失败后也不自动重放。
- AC4：非法 Hook、非 startup/resume、缺少 Codex CLI、App Server 启动/握手失败均失败关闭并给出无正文诊断。
- AC5：ChatGPT Desktop 与 Windows 保持 Unsupported/Deferred；产品文档不误报支持。
