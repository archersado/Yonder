# AG-S7 Codex 连接生命周期 Spike Verification Goal

日期：2026-10-19。状态：PASS（macOS Codex CLI）；Windows Deferred；ChatGPT Desktop Unsupported。

## 终局能力矩阵

| 客户端/平台 | 当前会话绑定 | 生命周期与恢复 | 结论 |
|---|---|---|---|
| macOS Codex CLI 0.162.1+ | 公开 `SessionStart` Hook 显式提供当前 `session_id`；共享 daemon/proxy 恢复同一 thread | Hook/Connector 拥有有界重连；Yonder 重启、Connector/Hook 退出和新会话启动均有正式证据 | Accepted |
| ChatGPT Desktop Codex | 未发现公开的当前会话 endpoint 或生命周期 Hook | 禁止附着私有 stdio、读取内部数据库或注入进程 | Unsupported/Deferred |
| Windows Codex CLI | 未运行对照样本 | 按用户决定暂缓，不以 macOS 结果外推 | Deferred |

## macOS 正式证据

- AG-S8 正式产品 Goal 已证明：受信 Hook 冷启动后无需手填 thread ID、Socket 或 `agent-bridge`；共享 daemon 的 WebSocket-over-proxy 完成 initialize、明确 thread resume、活动 turn steer 与空闲 turn start。
- 保持 Connector 不动重启正式 `Yonda.app` 后，旧 Gateway 连接关闭并在有界窗口内连接新 UDS；输入 ID 去重证明跨重连不重放正文或任务副作用。
- 终止 Hook 宿主后 Connector 与受监管 proxy 均退出；同一明确会话的受信 `SessionStart(resume)` 可重新建立单一 Connector/Proxy。旧版 Codex 会话无法由新版连接器安全接管时，`thread/resume` 失败关闭；从启动起由新 CLI 持有的新会话重新注册后，正式无副作用 replan 输入只进入该会话。
- AG-S13 后续合约补充了单活抢占关闭语义：旧 Connector 收到 `preempted` 后退出而不重连；普通 EOF/Yonder 重启仍有界重连。未确认输入保持 unknown，不自动重放。
- 2026-10-19 在 Codex CLI 0.163.0 上重新运行 `check-codex-connector-lifecycle-macos.py`：共享 Unix App Server、remote resume、缺失 thread/endpoint 失败关闭全部通过。`cargo test --offline --locked -p yonder-cli` 为 21/21 PASS，覆盖当前协议握手、显式 session、重连、去重、抢占退出、回复订阅和附件边界。
- 同日 EX-S2 正式无副作用任务在 Connector 重连后自动把协议 1.53 `goal_verification` 输入投递到当前唯一 Codex 会话，慢脑读取 Gateway 新鲜事实后完成任务，进一步证明任务通道恢复不冒充输入通道，输入通道恢复后才发生回注。

## 淘汰路线与安全结论

- 拒绝按最近会话、工作目录、进程或单一候选猜测 thread。
- 拒绝另起第二规划 Agent、独立 stdio App Server 接管 active writer、读取 Codex 私有数据库/队列或附着 ChatGPT Desktop 私有 stdio。
- endpoint、thread/session ID 与输入正文不进入 Yonder SQLite、事件、Outbox 或日志；连接中断不重放未确认输入或任何任务副作用。
- 任务 Gateway 与输入/replan 通道分别注册和展示；一侧恢复不能证明另一侧可用。

## 结论

AG-S7 的 macOS Codex CLI 技术路线已由后续产品 Story AG-S8 完整实施并验证，AD-AG-10 的 Accepted 子范围成立。Windows 与 ChatGPT Desktop 分别保留 Deferred/Unsupported，不宣称跨平台完成。Spike 可以 Archive；后续产品缺陷继续在 AG-S8/AG-S13 规格中演进，不重新打开本 Spike。
