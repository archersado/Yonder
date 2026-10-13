# Design：活动 task.list Runtime 覆盖

Gateway 先按既有查询取得有界 SQLite 页和稳定 `next_after_task_id`。随后仅遍历返回页内任务：若同 ID Runtime 快照序号不旧于持久项，则复用 `task.get` 的状态映射覆盖 sequence、status、current step 与 HandBack 意图。

覆盖后再按请求的 `include_finished` 与 `running_only` 过滤。Runtime 中存在但当前 SQLite 页尚未包含的任务不注入列表，避免破坏基于创建时间和 task ID 的分页边界；新建任务仍由既有创建事务保证进入列表。Runtime 不可用时保持历史 SQLite 查询。

该处理只读内存，不等待 projector、不写 SQLite，也不改变 `task.events` 的连续尾部拼接。
