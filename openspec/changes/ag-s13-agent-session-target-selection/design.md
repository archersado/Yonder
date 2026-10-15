# 设计

`AgentInputHub` 将输入连接保存为单一 `Option<(token, SessionSink)>`。`register` 在锁内替换槽位并清除属于旧 session 的回复关联，锁外向旧 sink 发关闭信号；新 sink 立即成为唯一投递目标。

`unregister(session_id, token)` 仅在两个标识同时匹配当前槽位时清空，因此旧连接被抢占后的延迟退出不会影响新连接。同一 session ID 重连仍依赖新 token 防止 ABA。

Desktop 删除 `agent_session_list`、`agent_session_select`、会话变化事件及所有会话选择 UI。Agent 管理只保留注册授权和本地语音回复设置。该状态短生命且非任务事实，不进入协议或持久化。
