# AD-TM-06 删除入口取消并保留数据

状态：Accepted；日期：2026-09-14。Architecture Impact：architecture-change（撤销实验清理协议，格式兼容）。用户明确“取消任务，任务数据不删除”，覆盖本次AD-TM-05清理默认设计。卡片统一接管/取消任务按钮，不创建含义相同的两取消按钮；取消仅更新合法状态并追加事件Outbox，任务说明/历史/归属/幂等映射保留，全部可查。执行过任务仍须停止确认，接管记录门禁不变。

撤销task.delete/TaskDelete/Deleted响应和所有清理用例，不保留不可达的危险清理实现；tm-s5-terminal-delete变更withdrawn，不Archive/Done。AD-TM-05保留历史但Superseded。原产品未来用户明确删除历史的独立需求不自动套用当前按钮。

当前研发正式库已升级实验schema4，但只读检查证明两任务、deleted标记0，未发生实际清理。兼容迁移在同一IMMEDIATE事务检查所有tasks.deleted=0且events.kind均transition，移除未使用的两实验列并回到schema3；说明、任务、事件、序号、Outbox、幂等不变。若检测已删除标记/非transition事件或不匹配格式则拒绝启动并保持数据，不能凭空恢复或抹掉删除事实。未知版本仍拒绝。新库及schema2按原schema3初始化迁移，不再升级4。变更只允许去掉未使用列，不清理记录。

验证：临时schema4样本迁移前后六项事实一致、带标记格式拒绝无写、取消仍保留正文/事件且创建重试返回cancelled；原生卡片接管禁用/取消入口与历史可见。本机两真实任务及说明计数保持，Windows暂缓。完整停止接管/记录Story不Done。
