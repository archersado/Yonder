当前归属 Story：TM-S1；规划：`docs/specs/epic-TM/story-TM-S1/README.md`。旧编号保留历史追溯。

状态：Frozen（历史承接，不再新增实施）。2026-09-18依AD-DEV-01完成后续范围迁出；已实施的Domain状态、任务存储、只读查询、归属、握手与进程内准入证据保留。Gateway、桌面、执行、文件、凭据和平台验证已分别迁入AG/TM/CU/FI/DS/ST/EN模块，禁止继续在本混合Change中实施。本状态不表示迁出的Story已完成，本Change也不Archive。

# 任务状态可见

2026-09-11 任务归属增量关联 AD-OCT-04，Architecture Impact：architecture-change。变更查询可信身份边界、快照字段及新建库格式；旧库保护和部署限制见 AD-OCT-04，迁移另建 Change。

关联 OCT-S1 与 ARCHITECTURE-SPINE、AD-E0-06、AD-OCT-01、AD-OCT-02。Architecture Impact：conforming。

2026-09-11 多任务增量关联 AD-OCT-03（Proposed），该增量 Architecture Impact：architecture-change；增加统一任务空间、task.list 契约规划与资源准入规则。尚未实施或发布协议变更，不解除 E0 门禁。

模块：domain、application、protocol、adapters、desktop、CLI。遵循 AGENTS.md 单向依赖。先实现无技术依赖的 Domain 状态变更计算；后续协议类型、数据库迁移和 IPC 实现必须补充相应增量设计后落地。
