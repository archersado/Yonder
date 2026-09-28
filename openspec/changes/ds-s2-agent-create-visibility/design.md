# Design：DS-S2 Agent 新建任务即时透出

将原本仅在 `main.rs` 的 pet 邻近任务空间展示函数移动到 desktop library，供受限 Local Socket 宿主复用。Socket 在成功处理请求后读取 `TaskHost::presentation()`：仅当该次消费返回 `listening`，且没有 CUA/语音/圈选窗口在前台时显示 task-space。窗口内部仍以现有 `yonda-tasks-open` 事件查询任务快照；宿主不把任务正文或列表直接注入前端。

显示错误只记录为本机呈现失败，不传播为 Gateway `task.create` 失败。该路径不接受 UI 输入，不构成新的任务通知协议。
