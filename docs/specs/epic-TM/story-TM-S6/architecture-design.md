# TM-S6 架构设计

## 边界与依赖

依据 AD-TM-02-RUNNING-STATE-QUERY.md；Application 复用已有 TaskStore.running(1)，SQLCipher Adapter 全局筛选当前表，无 Adapter 互调。无新依赖、索引、迁移和 wire 模型。后续补充的收起预约仅协调进程内准入，不属于任务事实。仅可信宿主调用，Gateway 不暴露全局查询。

## 状态与契约

running_state 返回 Rust 内部 RunningState 枚举 Running / NoRunningTask / Unknown。结果是一次读取快照；没有 running 行不等于空闲。执行占用、未确认停止、启动恢复和新任务准入竞争仍必须由后续宿主整合；不得缓存此结果充当隐藏授权。

## 失败与验证

所有存储错误归 Unknown，不重试、不修复数据库。合成 SQLCipher 双连接验证跨页跨归属、提交可见性、重开与恢复、读取错误和查询无写入。核心库测试不替代 Windows/macOS 桌面证据。

## 执行占用汇总

依 AD-TM-02 补充决定，Admission.has_occupancy 只锁内读取 occupied 非空与否；activity_state 先查 running_state，再查占用，无数据库/准入嵌套锁。None 实例返回 Unknown。任一来源忙则 Busy；双空为 NoKnownWork；其余 Unknown。实例只能由已完成恢复的可信宿主提供，不能创建临时空实例绕过真实占用。

AC5–8 用真实 SQLCipher 与 Admission 验证终态但仍占用、空资源后台任务、丢弃凭证和读取失败；Application 单元检查锁中毒。查询与后续新任务准入仍可交错，未来实际收起必须由 DS 与准入串行协调。

## 收起预约

依 AD-TM-02 收起预约补充，使用唯一 Admission 同一互斥锁协调 rest_reserved 与 occupied，新增 RestPermit 和 reserve_rest(store, Option<&Admission>)。数据库核对在预约后、锁外进行；失败清理尚未用于展示的预约。PresentationBusy 只表示展示尚未解除，不是任务失败或第二任务状态。调用方收到该拒绝须先完成展开再首次准入。不可通过临时 Admission 绕过。

线程屏障验证收起/准入竞争；SQLCipher 验证读失败和 running 拒绝及清理、成功凭证阻止 start 写库。窗口接线与双平台证据仍待 DS-S1 门禁。
