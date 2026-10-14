# 设计

`AgentInputHub::register` 已生成进程内 token，本变更将其作为不透明 `session_ref` 返回 Desktop UI。Hub 增加 `active_explicit` 区分唯一会话自动选择与用户选择，并集中处理 register/unregister/select 状态转换；输入投递继续只读取 Hub 的 active sink。

Tauri 增加 `agent_session_list` 与 `agent_session_select`，仅允许 Agent 管理窗口调用。摘要包含 Agent ID、当前进程内序号、当前目标及截图/回复能力，不包含 session_id。UI 不缓存路由事实，每次打开和选择后重新读取 Hub。

该状态短生命且非任务事实；进程退出即清空。选择失败不影响现有圈选草稿恢复语义。
