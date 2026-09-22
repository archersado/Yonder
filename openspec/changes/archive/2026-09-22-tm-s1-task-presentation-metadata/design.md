# 设计

`name/source/current_step/observation/next_intent` 均为有界展示事实。来源由可信组合根写入，不能由请求自报；当前步骤复用 `StepDeclaration`；观察摘要仅由可信执行器在 Observe 后提交；下一步意图仅由所属 Agent 或可信控制用例提交。

每次接受写入都在同一 SQLite 事务中更新当前展示值、递增任务 sequence、追加有类型事件并写 Outbox。终态只允许补充审计事实，不回退步骤或改写终态。日志和 Outbox 不记录正文，UI 按纯文本渲染。

`task.get` 返回完整快照，`task.list` 只返回有界摘要；缺失字段为 `null`，不用空字符串或占位文本冒充。旧库迁移必须先备份，历史任务 `source=legacy`，其余新字段为空。
