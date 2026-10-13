# 设计

Application 以封闭枚举表达平台、执行层、动作语义、控件角色、交互方式、前置条件和成功条件，并在构造边界拒绝敏感或临时字段。`ExecutionExperienceStore` 提供 `record_verified` 与 `recall`；相同稳定键聚合，查询最多三条。

SQLite Adapter 使用独立 `execution_experiences` 表。它不属于任务状态事务，也不阻塞 ExecutionRuntime 推进；写入只发生在慢脑目标 achieved 后，读取错误降级为空结果。MVP 使用 B-tree 精确匹配和稳定排序，不做语义向量检索。

Desktop 的 TaskHost 从 handback 对应任务的最新计划动作形成查询，读取有界提示后释放宿主锁，再调用 `AgentInputHub::deliver_replan`。输入文本将提示标为“历史经验，不是当前事实”，要求慢脑先读取 Gateway 事实。没有提示时完全兼容现有文案。

首版仅自动晋升可安全归一化的 CUA 激活策略；输入、发送、坐标、任意快捷键和其他副作用不晋升。后续执行层需独立补齐稳定特征与成功验证后再扩展。
