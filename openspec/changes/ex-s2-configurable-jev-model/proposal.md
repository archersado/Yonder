# Proposal：EX-S2 可配置 Jev 模型

关联 Story：EX-S2；关联 ADR：AD-EX-03。

当前设置面板允许选择本地服务与端点，但 Application 运行时仍只接受远端 TypeSafe，且 Worker 把模型固定为 `jev-latest`。本 Change 将受限模型标识纳入 Application 配置、SQLite JSON 与独立设置面板，允许合法本地 base URL 进入既有 macOS Jev Adapter，并让候选选择与步骤核验使用同一配置快照。

2026-10-09 增量把决策门禁读取的置信度字段名与阈值一并纳入配置。字段名由用户填写，留空默认 `confidence`；旧 `confidence_metric` JSON 兼容迁移。已配置字段缺失或非法时安全交回，不自动探测其他属性。

不新增 Gateway、任务状态、执行栈或模型自由提示词；API Key、请求正文、候选内容和观察摘要的既有安全边界不变。Windows 验证继续暂缓。
