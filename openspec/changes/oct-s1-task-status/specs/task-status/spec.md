# 任务状态

状态：Frozen历史Delta。已实施的基础要求和验证保留追溯；文中标注待实施的跨模块范围已迁入AG/TM/CU/FI/DS/ST/EN对应Story，不再由本Delta授权Apply。

## Requirement：Gateway 握手

系统 SHALL 按 AD-OCT-05 对每个已认证连接单独协商协议与可用能力。

### Scenario：先握手再查询

- **WHEN** 新会话未成功 gateway.hello 就请求任务
- **THEN** 返回 -32002 且不访问存储
- **AND** 主版本 1 握手成功后可查询，同主版本选择较低 minor；主版本不兼容返回 -32010 并关闭该会话的查询门禁

### Scenario：握手不能升级身份

- **WHEN** hello 或后续任务请求的 agent_id 与固定 AuthContext 不符
- **THEN** 返回 -32003，不更换身份或扩大任务范围
- **AND** 重复合法握手幂等，其他会话和新建会话不会因此获得握手状态

### Scenario：只公布实际能力

- **WHEN** 候选核心握手成功
- **THEN** 返回协议 1.0、宿主平台及 task.read 能力的版本/可用性/原因
- **AND** 不读取任务库、不启动执行器，不把握手视为 OS 或云端身份认证

## Requirement：任务归属与可信身份

系统 SHALL 按 AD-OCT-04 由可信 Application AuthContext 决定创建归属和读取范围，禁止从请求 agent_id 推导身份。

### Scenario：Agent 查询隔离

- **WHEN** 已绑定 Agent 身份调用 task.list/task.get/task.events
- **THEN** 只能访问自己的任务，列表必须先按归属过滤再分页
- **AND** 请求 agent_id 与绑定身份不符时返回 -32003；他人任务和不存在任务统一返回 -32004

### Scenario：本机全量任务空间

- **WHEN** 受信任本机用户身份查询任务
- **THEN** 可读取当前用户库全部任务及其 owner_agent_id
- **AND** JSON 无法自行声明或升级为该身份；创建归属由可信上下文写入，迁移状态不改变归属

### Scenario：保护无归属旧库

- **WHEN** 打开 schema_version=1 或未知版本任务库
- **THEN** 拒绝且保留原文件，不重建、不自动赋予归属；迁移另建 Change
- **AND** 新空库初始化 schema_version=2，状态/事件/Outbox 继续同事务

## Requirement：多任务空间与资源约束

系统 SHALL 按 AD-OCT-03 提供全部授权任务的统一总览，并允许无冲突后台任务在有界容量内并行。该增量待实现，协议细节必须先更新 AD-OCT-02。

### Scenario：全部进行中任务

- **WHEN** 用户打开任务空间，非终态任务数超过一页
- **THEN** 可通过分页访问全部 created、running、waiting-for-user、paused、interrupted 任务，并查看状态、来源、步骤及等待原因
- **AND** 完成、失败、取消任务进入历史；筛选不改变任务执行状态，不用虚构百分比替代未知进度
- **AND** 列表来自 Application/SQLCipher，Agent 请求按已认证 AuthContext 限制范围，不能伪造 agent_id 越权

### Scenario：独立后台任务并行

- **WHEN** 两个任务不争用互斥资源且有可用执行容量
- **THEN** 两者可以同时 running，分别更新状态与逐任务 sequence
- **AND** 不将两个任务的 sequence 跨任务比较；列表刷新可以发现新任务及终态变化

### Scenario：资源冲突排队

- **WHEN** 多个 CUA 任务争用同一个前台桌面，或多个任务写入同一文件
- **THEN** 互斥资源最多由一个执行者占用，其他任务显示等待原因，不伪报为 running
- **AND** BUA MVP 仍限单并发；用户接管后的自动恢复仍禁止

### Scenario：未选中的任务仍在运行

- **WHEN** 任何任务仍 running，即使它不在当前页或不符合当前筛选
- **THEN** 小龙保持忙碌，不自动隐藏；状态未知时也不隐藏
- **AND** 最后一个运行任务结束后才重新开始完整三分钟闲置计时

### Scenario：取消互不影响

- **WHEN** OCT-S2 的取消入口对一个任务发起取消
- **THEN** 只停止该任务的后续动作，其他独立任务继续
- **AND** 确认执行停止后才释放其资源，结果 unknown 时不自动重试

### Scenario：浏览器任务入口

- **WHEN** 当前平台具有已验证的 BUA 能力并存在关联浏览器任务
- **THEN** 任务空间显示其统一状态并可打开既有 ego-lite Task Space
- **AND** 不在 Yonda 重建 Browser Task Space；无 Runtime 时不提供执行入口

## Requirement：现有实现的持续集成

系统 SHALL 检查正式 Workspace 依赖方向、Rust 协议生成一致性及 PR 的 Story/OpenSpec/验证记录关联。

### Scenario：禁止越层依赖

- **WHEN** 普通、开发、构建或平台条件依赖违反模块方向，或 Domain 引入外部库
- **THEN** 检查失败；依赖改名不得绕过检查

### Scenario：PR 缺少关联

- **WHEN** PR 正文缺少唯一的 Story、OpenSpec 或 Verification 字段，引用不存在的文件，或 Story 与 Change 未双向引用
- **THEN** 检查失败，不把验证记录存在视为验证已通过

### Scenario：目标平台库层检查

- **WHEN** PR 或 main 推送触发 CI
- **THEN** 分别在 Windows/macOS 运行 Workspace 测试和协议生成漂移检查
- **AND** 不将这些测试视为桌面或 Agent 原生 E2E

## Requirement：只读查询协议

### Scenario：进程内任务列表分页

- **WHEN** 可信本机调用方对其授权存储实例发起 task.list，limit 为 1..100
- **THEN** 默认返回全部非终态任务，include_finished=true 时包括终态；按 task_id BINARY 升序使用 after_task_id 排他分页
- **AND** 每页最多 limit 项，仅有后续项时返回 next_after_task_id；拒绝非法游标、越界 limit 和过期请求
- **AND** 跨页状态变化和游标前的新任务须刷新首页发现，不承诺跨页一致快照
- **AND** 尚未实现任务所有者及 AuthContext 约束前，禁止把该进程内分派暴露给外部 Agent

系统 SHALL 使用 AD-OCT-02 的 JSON-RPC 2.0 只读契约；Rust 类型是 Schema 和 TypeScript 唯一来源。

### Scenario：请求校验

- **WHEN** task.get/task.events 携带有效 id、agent_id、task.read、未过期 deadline 和 task_id
- **THEN** 从 TaskStore 查询并保留响应 id，events 采用排他序号游标和 1..100 条上限
- **AND** 拒绝超长请求、未知字段、缺失元数据、非法 ID、非规范序号和过期请求

### Scenario：精度和生成一致性

- **WHEN** 序号大于 JavaScript 安全整数上限
- **THEN** JSON 使用规范十进制字符串，不损失精度
- **AND** Rust 类型变更而生成产物未更新时，生成检查失败

## Requirement：状态与事件

系统 SHALL 只接受合法状态迁移，每个实际迁移的 sequence 增加一；拒绝操作不得产生状态或事件。

### Scenario：终态保护

- **WHEN** 已完成、失败或取消的任务收到启动或恢复请求
- **THEN** 拒绝请求，原状态与序号不变

### Scenario：重启恢复

- **WHEN** 重启恢复读取到 running 任务
- **THEN** 生成 interrupted 迁移，不重新执行动作
- **AND** 每批最多处理 100 个任务，每任务状态、事件和 Outbox 同事务，序号只增加一
- **AND** 其他状态不变；重复恢复不追加重复事件

### Scenario：恢复中途失败

- **WHEN** 恢复一批任务时某任务的 Outbox 插入失败
- **THEN** 该任务三表更新回滚并返回错误，之前已恢复的任务保持 interrupted
- **AND** 再次显式恢复只处理剩余 running 任务，不重放动作
- **AND** 普通数据库 open 不触发恢复；生产宿主必须在单实例启动阶段完成恢复后才开放执行（宿主接线待实现）

### Scenario：显式继续

- **WHEN** 外部 Agent 在重新观察后显式恢复 interrupted 任务
- **THEN** 生成 running 迁移和下一事件序号

## Requirement：持久化事务

系统 SHALL 在同一 SQLCipher 事务内创建或更新任务、追加事件并写入 Outbox。失败不得返回成功快照。

### Scenario：读取后发生竞争更新

- **WHEN** 两个连接读取同一任务序号，首个连接提交成功
- **THEN** 第二个连接携带旧序号提交时返回 Conflict
- **AND** 不覆盖首个连接写入的状态，不产生重复事件

### Scenario：Outbox 写入故障

- **WHEN** 状态更新和事件插入后 Outbox 写入失败
- **THEN** 三张表全部回滚，其他连接仍读取之前已提交状态

### Scenario：错误密钥与未知版本

- **WHEN** 使用错误数据库子密钥或读取未知 schema_version
- **THEN** 打开失败，不重建、不清空、不升级原文件
# ADDED Requirement：进程内多任务资源准入

## Scenario：确认停止后提交终态并释放
已确认停止且结果确定时，由占用凭证绑定的任务提交 completed 或 failed 及事件/Outbox，再释放该任务资源。事务失败返回原凭证并保持占用；旧序号不得完成任务或释放资源。释放失败必须与已提交终态区分报告。一个任务结束不释放其他任务占用，unknown 不进入此用例。

## Scenario：准入与持久化启动
可信启动用例只有在全部资源获取和 created→running 的 CAS/事件/Outbox 事务提交后才返回成功及占用凭证。资源冲突不改变任务；旧序号、非法状态或事务失败不返回凭证，释放本次未派发占用。其他任务的占用保持不变。暂停或中断任务不得通过 Start 自动恢复。

关联 OCT-S1、AD-OCT-06；Architecture Impact：architecture-change。核心准入不代表已接入真实执行器。

## Scenario：有界原子准入
容量限制内，不同文件后台任务可同时占用；相同文件、第二个桌面或浏览器任务拒绝并返回资源等待原因。请求全部资源一次获取，失败不部分占用；同 task_id 重复占用被拒绝。文件身份来自可信 Adapter，本批使用合成身份验证。

## Scenario：停止确认与用户接管
占用凭证丢弃或取消请求不会释放资源。仅确认执行停止后显式释放，可再次准入。用户接管阻断新的桌面任务，不阻断其他后台任务；显式归还后恢复准入。多线程同时申请时不能突破容量或资源独占，不持锁执行 I/O。
