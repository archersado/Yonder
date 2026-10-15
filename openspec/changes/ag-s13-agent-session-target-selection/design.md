# 设计

`AgentInputHub` 将输入连接保存为单一 `Option<(token, SessionSink)>`。`register` 在锁内替换槽位并清除属于旧 session 的回复关联，锁外向旧 sink 发关闭信号；新 sink 立即成为唯一投递目标。

`unregister(session_id, token)` 仅在两个标识同时匹配当前槽位时清空，因此旧连接被抢占后的延迟退出不会影响新连接。同一 session ID 重连仍依赖新 token 防止 ABA。

Desktop 删除 `agent_session_list`、`agent_session_select`、会话变化事件及所有会话选择 UI。Agent 管理只保留注册授权和本地语音回复设置。该状态短生命且非任务事实，不进入协议或持久化。

协议 1.51 增加 Yonder→Connector 的 `agent.session.close`，参数仅含当前 `session_id` 与封闭原因 `preempted`。Desktop 消费 Hub 关闭信号时先发送该帧再结束旧 Socket；Codex Connector 将其映射为不可重连的终止结果。EOF、连接重置和连接拒绝仍表示 Yonder 重启或传输故障，沿用 AD-AG-10 的有界退避。关闭帧到达前已经发送但未确认的输入保持 `unknown`，不自动重放。

AG-S13 真实回复验证同时修正 AD-AG-10 的订阅假设：共享 daemon 中“已加载”的 thread 仍须由当前 Connector 经同一 proxy 执行 `thread/resume(excludeTurns=true)`，否则输入可投递但本客户端收不到回复通知。连接器只在取得同 thread ID 回执后向 Yonder 注册能力。
