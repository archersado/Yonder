# TM-S7 统一执行启动状态

Story：TM-S7；依据 Accepted AD-TM-13。

所有能力首次副作用前统一提交 `created→running`、步骤、attempt、事件与 Outbox。复用现有 CUA/BUA 启动事实，并由 DO-S2、CM-S1 的独立产品 Change 完成 macOS Document/Command 接线；本 Change 维护统一语义和跨 Story 验收映射，不复制能力实现。

Architecture Impact：architecture-change。具体资源枚举、协议和迁移已在 AD-TM-13 定案；Document/Command 端口、授权与 Gateway 契约仍归各自 Story，不在本 Change 建立第二实现。
