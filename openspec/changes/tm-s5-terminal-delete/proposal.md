状态：withdrawn。用户明确取消而非清理，AD-TM-06覆盖本提案；保留实验历史，不Archive/Done。

# TM-S5 用户已结束任务删除

关联docs/specs/epic-TM/story-TM-S5/三份设计、Accepted AD-TM-05/AD-ST-01。Architecture Impact：architecture-change（Rust删除响应及SQLite4迁移）。用户卡片删除确认后清理已结束任务说明及历史，保留最小防重放/无正文删除事件Outbox，不自动删除或启动Recording。Agent不得删除；不依赖系统密钥；完整Story不Done。
