# 设计

- Adapter 按 `(task_id, sequence, kind='source')` 读取既有 source payload，并结合任务不可变 `owner_agent_id`；缺失即无创建来源历史，损坏即失败。
- Application 在协议 1.24 输出可选 `creation_event`；1.23 及以下缺省，继续执行当前授权、连续性和编码预算。
- Task Space 只显示来源与 Agent 标识，不显示描述、幂等键、凭据或 Payload；不把创建归属当成当前授权。
- 不迁移、不回填、不增加写入者；macOS 使用独立正式宿主验证，Windows 暂缓。
