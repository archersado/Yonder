## ADDED Requirements

### Requirement: 全量运行状态查询
可信宿主 SHALL 从当前状态事实源查询所有 Agent 的运行任务，不受列表页影响，仅返回 Running、NoRunningTask、Unknown。

#### Scenario: 页外任务运行
- **WHEN** 前 100 条没有运行任务而后续其他 Agent 任务已提交 running
- **THEN** 返回 Running

#### Scenario: 数据不可读
- **WHEN** 当前状态查询失败
- **THEN** 返回 Unknown，不写库、不回退为空闲

#### Scenario: 当前状态更新
- **WHEN** 最后一个 running 已提交完成或显式恢复为 interrupted
- **THEN** 新查询返回 NoRunningTask，该值不授权收起桌宠

#### Scenario: 查询无副作用
- **WHEN** 重复查询已提交状态
- **THEN** 任务、事件和 Outbox 保持不变，结果最多读取一个任务快照

### Requirement: 执行占用参与汇总
可信宿主 SHALL 使用唯一 Admission 的实际占用与数据库共同观察工作状态，不从终态推断已停止；结果 SHALL NOT 作为无竞争的收起许可。

#### Scenario: 后台任务未释放
- **WHEN** 数据库无 running，但空资源后台任务尚未释放或凭证被丢弃
- **THEN** 返回 Busy，查询不释放资源

#### Scenario: 来源不可用
- **WHEN** 宿主未提供已恢复的 Admission，或某来源读取失败且另一来源未确认忙碌
- **THEN** 返回 Unknown

#### Scenario: 已知忙碌优先
- **WHEN** 一个来源确认忙碌，即使另一个读取失败
- **THEN** 返回 Busy，不推断失败来源健康

#### Scenario: 双空观察
- **WHEN** 当前表无 running 且唯一准入实例无占用
- **THEN** 返回 NoKnownWork，后续隐藏仍需防止新任务准入竞争

### Requirement: 收起预约与执行准入互斥
宿主 SHALL 在收起前取得唯一 Admission 的预约，并保留至确认展开；所有执行 SHALL 经同一 Admission。

#### Scenario: 并发争用
- **WHEN** 新任务和收起同时申请
- **THEN** 至多一方成功，收起成功时新任务返回 PresentationBusy

#### Scenario: 数据库否决
- **WHEN** 已预约但数据库存在 running 或读取失败
- **THEN** 返回拒绝并撤销尚未用于展示的预约，不修改任务

#### Scenario: 保留至展开
- **WHEN** 收起预约凭证被丢弃或尚未确认展开
- **THEN** 新任务仍不能准入；只有显式确认展开并释放后才恢复准入
