# Design

创建契约与幂等登记同事务，完成摘要与终态事实同一 Runtime 事件或兼容 SQLite 事务。摘要是 Agent 声明，不替代独立目标核验。File/Document 仅在成功 Observe 后由 Application 生成 opaque ref；Agent 请求不接受 artifact ID 或路径。

Gateway 1.52 对新任务强制 `input_summary`/`expected_output`，完成强制 `result_summary`；旧版本剥离字段。Task Space 使用核心快照字段，不增加额外大查询。所有限额在 Application 和协议入口双重校验。
