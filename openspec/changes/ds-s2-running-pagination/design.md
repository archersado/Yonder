# 设计

Rust `ListParams.running_only` 缺省为 `false`。Application 将该条件传到既有 `TaskStore::list`；SQLite 查询在 `LIMIT` 前筛选，并继续应用 owner、游标、排序。

Gateway 1.16 记录会话是否可用该过滤；低版本发送 `running_only=true` 返回能力版本错误，省略或 `false` 保持旧行为。桌面可信本地查询直接使用当前协议。

UI 进行中页发送 `running_only=true` 并直接渲染服务端页；全部页发送 `false`。回归样本把超过一页的非运行任务排在运行任务之前，确认首个进行中页仍返回运行任务。
