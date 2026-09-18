# 实施设计

依据 AD-TM-02，复用 TaskStore.running(1)，非空返回 Running，空返回 NoRunningTask，错误返回 Unknown。内部 Rust 类型不序列化，不修改外部 Agent 查询。查询不缓存、不写入，也不释放资源或恢复任务。

通过超过 100 条合成任务、双连接、重开库、显式恢复和读取故障验证 TM-S6 AC1–4。状态结果不能解决查询后新任务启动的竞争，禁止作为独立隐藏许可。

按 AD-TM-02 补充决定，新增 Admission.has_occupancy 只读方法与 activity_state(store, Option<&Admission>) 内部用例。None 为 Unknown；Running 或占用非空为 Busy；NoRunningTask 且占用为空为 NoKnownWork；其他 Unknown。不在锁内访问存储，不缓存，任一来源确认忙碌优先。结果不是原子快照，不授权隐藏。

收起预约先在同一准入锁内登记，再锁外查数据库。无运行任务才交付 RestPermit，失败撤销未展示预约。try_acquire 返回 PresentationBusy 阻止新执行；凭证丢弃保持阻断，确认展开后显式释放。无自动重试，不持锁调用窗口或存储。
