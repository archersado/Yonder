# 设计

保留架构中的 BUA Bridge 端口和 `external_task_ref` 概念，不提供运行时实现。任何调用应返回明确的 `capability_unavailable`，不得回退到 CUA 或其他浏览器自动化引擎。恢复 Story 后仍直接调用 ego-lite Task Space。
