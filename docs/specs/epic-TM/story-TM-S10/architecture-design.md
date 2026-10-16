# TM-S10 架构设计

## 边界与依赖

按 Accepted AD-TM-25：Rust 协议为唯一模型源。协议 1.52 在 CreateParams/TaskSnapshot 增加 `input_summary`、`expected_output`，在 CompleteParams/TaskSnapshot 增加 `result_summary`。Gateway 根据协商版本要求字段并剥离旧客户端投影。

## 状态与契约

schema 27 使用独立 `task_io_contracts(task_id,input_summary,expected_output)` 与 `task_results(task_id,result_sequence,summary)`；创建幂等比较契约，旧任务无行。兼容终结由 Store 新用例一次事务提交结果、状态、事件与 Outbox；活动任务由 Runtime `Terminal` 事件携带摘要，Projector 在同一批事务落盘。

File/Document 成功回执由 Application 生成 `artifact_<attempt_id>`，不含路径、授权 ID、哈希或正文。Runtime 增加产物事件并在 Snapshot 暂存当前产物集合；Projector 为每次可信新增生成下一完整清单版本。兼容路径调用 Store 的 append-artifact 原子用例。失败和 unknown 不调用登记。

DS 只消费 TaskSnapshot 与 `task.artifacts`，不读取 description，不拥有契约或结果状态。Architecture Impact：architecture-change；依赖方向不变，无新第三方依赖。

## 失败与验证

契约缺失、超限、幂等冲突、结果摘要缺失、旧序号、未核验完成和伪造产物引用均失败关闭。验证覆盖协议生成、迁移回滚、Runtime 投影幂等、兼容事务、Gateway 版本隔离、File/Document 成功与失败样本及 Task Space 安全渲染。
