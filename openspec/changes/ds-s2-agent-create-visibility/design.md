# Design：DS-S2 Agent 新建任务即时透出

将原本仅在 `main.rs` 的 pet 邻近任务空间展示函数移动到 desktop library，供受限 Local Socket 宿主复用。Socket 在成功处理请求后消费已提交创建提示；语音/圈选窗口在前台时延后，CUA 控制条不再阻断 task-space。任务空间无焦点显示并派发现有 `yonda-tasks-open` 自动事件；页面只在该自动事件下切到“全部”并重新查询真实快照，手动打开保留当前筛选。宿主不把任务正文或列表直接注入前端。

任务空间查询固定请求协议 1.32 `newest_first=true`。SQLite 在登记事务写入 `created_at`，按创建时间和任务 ID 倒序稳定分页；历史迁移值为 0。前端不反转单页、不伪造或临时置顶任务。显示错误只记录为本机呈现失败，不传播为 Gateway `task.create` 失败。
