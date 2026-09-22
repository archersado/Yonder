# Gateway 任务协议

只读契约沿用 AD-OCT-02/04；Gateway 会话握手与任务登记扩展见 AD-AG-02。协议库不提供传输或认证，握手门禁由 Application GatewaySession 执行。

任务快照包含 owner_agent_id。Application 分派必须另行接收可信 AuthContext；请求 agent_id 不匹配时拒绝，Agent 只能查询自身归属，本机用户上下文可看全量。该内部类型不是凭据验证，不能从 JSON 自行构造；详细边界与新库 v2/旧库拒绝策略见 AD-OCT-04。

支持 task.get、task.events 和 task.list。列表默认返回非终态任务，limit 为 1..100；after_task_id 为排他游标，include_finished=true 时包含终态。协议1.16可用`running_only=true`在分页前只筛选运行态，且不受`include_finished`扩大。响应 next_after_task_id=null 表示本次查询无下一页。跨页更新需刷新首页，不承诺跨页快照一致。

## 生成与检查

在仓库根目录运行：

```bash
cargo run -p yonder-protocol --example generate --locked
cargo run -p yonder-protocol --example generate --locked -- --check
cargo test --workspace --locked
```

`generated/` 全部由 Rust 类型生成，请勿手改；测试只比较生成结果，不自动覆盖文件。接入 CI 时执行第二、三条命令。Schema 表达静态结构，Rust 入口执行截止时间、ID 和序号数值范围校验；TypeScript 不替代运行时校验。

ts-rs 12.0.1 对枚举的 serde `deny_unknown_fields` 会输出忽略警告；Rust/Schema 仍保留该约束，契约测试验证未知字段确实拒绝。未为了消除警告关闭输入保护，也未全局屏蔽 serde 警告。

当前 query handler 从 TaskStore 读取状态、序号、事件、已登记步骤及有界动作/Observe分类；不传输输入正文、窗口树、截图或下一步意图。此模块的 agent_id 只是声明；不得在未绑定认证身份的情况下直接暴露给外部请求。

协议1.1会话可在存储支持且身份为Agent时协商task.create；参数为agent_id、capability、deadline、idempotency_key、description。返回当前任务快照，同键同说明重试不重复创建，不同说明返回-32009。1.0仅查询；身份必须由可信连接绑定，description不作为可执行指令。

协议1.2提供task.cancel（仅未开始任务），参数task_id/expected_sequence及统一身份/能力/deadline；返回snapshot。旧序号-32011，执行过任务-32012需停止确认。绑定LocalUser可控制已有任务，Agent须握手且只控制所属；created取消与事件/Outbox同事务，重复不增事件。详见AD-TM-04。

2026-09-14：AD-TM-07名称子范围协议1.3。协商1.3的task.create必须提供name（1–256 UTF-8字节，trim非空，无控制字符）；snapshot/list返回同一Agent名称。1.0–1.2快照保持原字段形状，1.1–1.2无名称创建兼容，旧版携带name拒绝-32010。相同幂等键不同名称或说明拒绝-32009。生成产物仍仅来自Rust。

2026-09-15：Accepted AD-AG-04 步骤声明子范围协议1.4。`task.step.declare` 只为归属 Agent 的 `created` 任务登记纯文本步骤，`task.step.get` 返回任务与最近声明；不执行标签或改变任务状态。声明、同状态事件与 Outbox 同事务，`task_id + step_id` 幂等。1.4事件可含 `step_declaration`，1.0～1.3保留连续事件但移除该字段。

2026-09-16：Accepted AD-TM-08结果事务子范围协议1.5。`task.events`可含 `attempt_result`，只表达完整执行身份、observed/unknown、动作成功分类、Observe有效性及稳定unknown原因；不表达语义计划。1.4及更低版本保留连续`running→running`事件但移除该字段，避免旧客户端收到未知字段。

2026-09-16：协议1.6新增 `task.control` 与 `task.control` 能力，登记执行中任务的pause/cancel/takeover pending控制。成功响应只表示“正在停止”；停止确认、定位与Recording不由请求伪造。created任务仍使用1.2 `task.cancel`。

2026-09-16：Accepted AD-BU-02新增协议1.7 `task.step.advance` 与1.8 `browser.execute`。Agent只提交所属任务、期望序号和动作种类；Yonder生成执行身份、读取持久化Browser引用，并在每次动作后记录Observe。macOS仅在ego-lite Bridge可用时声明执行能力；缺失依赖的平台返回`dependency_missing`。

2026-09-16：Accepted AD-CU-05新增协议1.9 `computer.execute` 与1.10 `task.complete`，用户修订后的协议1.11把动作请求统一为`tool_name + arguments`。CUA SDK `listToolsJson()`是动作及参数Schema的唯一来源；Yonder只递归拒绝Agent提交原生目标/执行身份，随后注入自身解析的目标并调用SDK。attempt、Worker与host身份仍由Yonder生成，每次动作后强制Observe；真实硬件输入会终止Worker并返回`unknown/user-input`，任务转为`interrupted`且不重试。Windows按用户决定暂缓并声明`dependency_missing`。

2026-09-17：协议1.12新增`computer.step`，在一次调用内完成步骤声明、CUA动作、Observe与安全步骤边界推进。协议1.13为`task.control`增加可选`focus_phase/focus_failure`；定位事实仅由可信宿主在停止确认后发布，旧协议投影移除新增字段。

2026-09-18：Accepted AD-BU-03新增协议1.15 `task.browser.get`。调用方复用`task.read`授权读取已提交的Browser引用；无引用返回`reference=null`。查询不启动ego-lite、不改变控制权或任务状态，也不返回页面内容、截图或浏览历史。

2026-09-18：Accepted AD-DS-03新增协议1.16 `task.list.running_only`。旧请求省略字段时行为不变；新过滤由存储在游标和页大小限制前执行，避免客户端页内过滤遗漏后页运行任务。

2026-09-18：Accepted AD-TM-10新增协议1.17 `task.wait_for_user`。归属Agent只可在已Observe并推进的步骤边界提交有界等待原因；状态、原因事件与Outbox同事务，随后释放任务占用。1.17事件读取返回`wait_reason`，旧会话不返回；恢复仍需独立显式流程。

2026-09-18：Accepted AD-TM-11新增协议1.18 `task.fail`。归属Agent只可把最新已Observe失败并推进到stopped边界的CUA任务提交为failed；`task.complete`与`task.fail`分别只接受成功/失败结论，unknown不终结。

2026-09-22：Accepted AD-TM-01新增协议1.19任务展示元数据。`task.get`与`task.step.get`返回完整快照中的可信来源、当前步骤、观察结果和下一步意图；`task.list`仍返回摘要。来源由登记入口绑定，观察与意图作为状态事件、展示事件和Outbox序号同事务提交；1.18及更低版本会话剥离新增字段，旧库迁移后的历史任务来源标记为`legacy`。
