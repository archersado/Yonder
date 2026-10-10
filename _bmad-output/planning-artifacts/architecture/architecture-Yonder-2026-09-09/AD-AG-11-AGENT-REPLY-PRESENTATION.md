# AD-AG-11 Agent 回复回传与临时呈现

状态：Accepted（macOS Codex CLI）  
日期：2026-10-10  
Architecture Impact：architecture-change（AgentSession 双向协议、回复运行时所有权与圈选交互生命周期）  
关联：AG-S5、AG-S8、CX-S2、AD-AG-10、AD-CX-02、AD-VI-02

## 背景

圈选图片与问题已能经同一已认证 AgentSession 进入当前 Codex thread，但 Yonder 只等待输入 accepted，随后关闭确认卡。Codex App Server 的正式事件面提供 `item/agentMessage/delta` 与 `turn/completed`；当前连接器却丢弃所有 notification，用户只能切回 Agent 客户端查看答案。

2026-10-10 主人要求 Agent 回答回到 Yonder，并同意统一使用既有 Agent Gateway 实施。

## 决策

- 在既有 AgentSession 上增加协商式 `user_input_reply` 能力；不建立第二个 Socket、HTTP 服务或厂商专用桌面通道。
- `agent.input` 可显式请求临时回复流。输入 accepted 只表示 Agent 宿主接收成功，不表示回答完成。
- Connector 把与该输入关联的最终 Agent 消息增量映射为有界 `agent.reply` 通知，并以 `started → delta* → completed|failed|interrupted` 的单调序列回传。
- Yonder Desktop 的 `AgentInputHub` 拥有当前输入与回复流的临时关联；圈选卡只展示匹配当前 `input_id + session_id` 的新鲜通知。
- 回复正文、截图、问题、Codex item/turn 细节不进入 SQLite、任务事件、Outbox、日志或索引。卡片关闭、完成后离开、超限、超时或断连立即释放运行时内容。
- Connector 只使用 Codex App Server v2 的公开通知。优先展示 `final_answer`；对未提供 phase 的兼容消息允许显示，明确的 commentary/partial_answer 不作为最终答案回显。
- `turn/completed.status=completed` 才能提交 completed；failed/interrupted、连接断开、序号错误或内容超限必须显示失败/中断，不伪造成功。

## 有界契约

- 同一 AgentSession 同时只允许一个请求回复的用户输入；后续普通输入、replan 与任务协议不受影响。
- 单个 delta 上限 16 KiB，累计可见回复上限 64 KiB；到达上限后停止追加正文并以 `truncated=true` 完成。
- 单次回复最长等待 5 分钟；超时回传 `interrupted` 并释放临时关联。
- 通知携带 `input_id`、`session_id`、`reply_id`、严格递增 `sequence`、phase、可选正文与截断标记。Yonder 拒绝跨会话、未知输入、重复/跳号和终态后的通知。
- 回复通知是短生命 UI 数据，不参与任务恢复；Yonder 或 Connector 重启后显示“回复已中断”，不从 Codex 私有历史猜测或补抓正文。

## 拒绝路线

- 让桌面 UI 直连 Codex App Server，或为回复再开放本地 HTTP/TCP；
- 把回复正文写入任务事件、Outbox、SQLite或日志；
- 解析 Codex TUI 输出、读取 rollout/transcript/内部数据库；
- 把 `agent.input` accepted 当成回答成功；
- 多个输入共享同一 active turn 时猜测回复归属；
- 为了恢复显示而持久化问题、截图或回复。

## 平台结论

- macOS Codex CLI：允许进入 AG-S9 产品实施与独立验证。
- Windows：随 AG-S8 保持 Deferred，不以 macOS 证据宣称跨平台完成。
- ChatGPT Desktop：保持 Unsupported，直到存在可支持附着当前 thread 的公开接口。
