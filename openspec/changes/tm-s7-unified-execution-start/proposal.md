# TM-S7 统一执行启动状态

Story：TM-S7；依据 Accepted AD-TM-13。

所有能力首次副作用前统一提交 `created→running`、步骤、attempt、事件与 Outbox。复用现有 CUA/BUA 启动事实，建立 Document/Command 后续接线门禁。

Architecture Impact：architecture-change。具体资源枚举、协议和迁移已在 AD-TM-13 定案；本 Change 不实现尚未就绪的 Document/Command 端口。
