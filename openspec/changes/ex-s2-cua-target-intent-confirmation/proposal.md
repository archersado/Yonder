# Proposal：EX-S2 CUA目标意图与发送确认

关联Story EX-S2、AG-S4、CU-S2及Accepted AD-AG-09。Architecture Impact：architecture-change（协议1.35意图引用、协议1.36消息框聚焦语义、Application内存Registry、计划动作语义和顶部本机确认）；不新增持久化表、任务状态、第二Planner或Driver。

## Why

当前计划只能用`press_key/type_text`粗略冒充会话解析，且没有取得一次性发送确认的正式入口。慢脑无法在不把联系人/正文写入计划的前提下形成可执行多步片段，真实企业微信任务因此只能停止或盲按键。

## What Changes

- 新增Gateway消息意图提议，敏感值只驻留Application有界内存，返回两个不透明引用。
- 扩展封闭CUA动作语义，区分聚焦搜索、输入目标、激活目标、聚焦消息框、填写草稿和发送。
- Application在派发时解析引用；Worker元素优先，坐标候选仅用于视觉兜底。
- 到达发送槽位后在顶部浮窗展示本机确认，批准引用派发前一次消费。
- CLI/MCP从Rust协议生成工具，Agent不能直接批准。

## Non-goals

不把敏感值写入SQLite/事件/Outbox/日志/Jev；不允许Jev生成坐标或文本；不实现通用Recipe；Windows继续暂缓。
