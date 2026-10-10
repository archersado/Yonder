# AG-S7 架构设计

## 边界与依赖

候选终局边界是“会话宿主提供绑定，连接器托管桥，Yonder 只消费已认证会话”：

1. Codex 会话宿主或其受支持扩展明确提供当前 `thread_id/session_id` 与可连接 endpoint。
2. 厂商连接器负责把该会话绑定转换为 AG-S5 的 `AgentSession + agent.input/replan_input`，并监管自身重连。
3. Yonder Desktop 不启动 Codex、不扫描最近会话、不读取 Codex 数据库；只接受既有 Gateway 身份下的显式会话注册。
4. MCP 任务通道与输入通道各自注册、各自断开、各自展示，不能相互冒充。

该边界属于 Proposed AD-AG-10，在 Spike 通过前不授权实施。

## 候选路线

### A. Codex CLI 共享 App Server 托管

由受监管启动器或 Codex 侧连接器启动/复用 `app-server --listen unix://...`，TUI 通过 `--remote` 使用同一 endpoint；连接器从当前会话获得明确 thread ID，监管薄桥并在 Yonder 重启后有界重连。这是 AG-S5 已验证基线，需要补齐冷启动、崩溃恢复、会话切换与安装生命周期。

### B. ChatGPT Desktop 受支持连接器

仅当 ChatGPT Desktop/Codex 提供公开的插件、SDK、生命周期 hook 或可复用 App Server endpoint 时采用。必须由当前会话主动给出绑定，不能附着私有 stdio、注入进程或解析内部状态。

### C. Yonder 猜测或代启 Agent

拒绝。包括按最近会话、工作目录、唯一进程推断 thread，读取内部数据库/队列，或由 Yonder 启动另一个 Codex Agent。该路线破坏 AG-S5 的确定性路由和 Yonder 非 Planner 边界。

## 状态与契约

### 生命周期状态机

Spike 应验证以下建议状态，不在本 Story 落代码：

- `unsupported`：客户端没有受支持的双向接口；任务 Gateway 可独立在线。
- `disconnected`：有能力但当前无明确会话绑定。
- `connecting`：连接器正在对 endpoint 与 thread 做受支持的加载/身份复核。
- `bound`：Gateway 身份、会话标识和输入能力均已确认。
- `reconnecting`：同一显式绑定的 endpoint 暂时断开，按有界退避恢复；不投递输入。
- `stale`：宿主或 thread 已变化，旧绑定永久失效，必须等待新会话主动注册。

重连不得自动重放 `agent.input` 或任务副作用。正在投递但未确认的输入按 unknown/失败关闭；新连接只接受新的输入 ID。

## 数据与安全

- 正文仍只存在于 AG-S5 有界内存交付中，不进入日志、SQLite、事件或 Outbox。
- endpoint 和 thread ID 默认属于短生命周期连接元数据；除非后续 ADR 明确证明必要性，不由 Yonder 持久化。
- 若后续需要新增 bootstrap/registration 协议，Rust 协议类型仍是 Schema 与 TypeScript 的唯一来源，并复用同一 Agent Gateway，不新增本地 HTTP/TCP。
- 连接器进程可被监管，但监管者不得是一个新的规划 Agent；Desktop 不得因连接器失败阻止任务 Gateway 启动。

## 失败与验证

### 验证矩阵

每条候选路线至少记录：冷启动、Yonder 重启、连接器崩溃、Codex 重启、thread 切换、活动 turn steer、空闲 turn start、MCP 单独恢复、输入 unknown、无正文落盘。macOS 与 Windows 使用同一矩阵；缺少平台证据时只保留 Deferred，不接受跨平台结论。
