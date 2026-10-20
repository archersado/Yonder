# TM-S8 架构设计

## 边界与依赖

`Codex 慢脑 → yonder MCP stdio → 当前用户私有 UDS → Agent Gateway → 已验证计划片段 → Yonder/Jev 快脑有界选择与连续执行 → Application/统一 start_execution → 能力 Port/Adapter → Observe/结果 → SQLite 事件与 Outbox → Task Space → 片段外经同一 Gateway 交回 Codex → 归属 Agent complete/fail`。

Jev 只能看受限候选语义、有界 Observe 摘要与预期条件，不能获得正文、联系人、截图、Driver 参数或自由规划权；单候选可确定性执行，多候选样本必须真实调用 Jev。片段外 replan 仍由 Codex 经同一 Gateway 提交。BUA 通过 ego-lite 引用；Office 组合 File Authorization、File Port 与 Document Port；Command 组合 propose、本机一次性批准与结构化执行。四类 Adapter 不互调，React 不拥有任务状态。

## 状态与契约

按 Accepted AD-TM-21，计划执行在首个 Driver 动作前被 Jev 交回时，SQLite 交回事务也必须完成 `created→running`，并原子写入 sequence、事件、Outbox 与下一意图；控制条不得拥有另一份执行状态。启动迁移同时核验 schema 18 的三张审计业务表：全有则不变，全缺才安全建空表，部分缺失失败关闭，绝不重建业务记录。

片段交回后的 replan 沿用同一任务：即使尚无 attempt 和 Admission，占用也必须只在新片段首个副作用前由 ExecutionRuntime 幂等取得，不得把 `running` 误判为不可恢复或建立第二任务。

## 失败与验证

验收进程必须连接用户正在看的 Yonder 实例。不得启动隔离 TaskHost、直接调用 Driver、用 Python/Swift 创建或推进任务、直接修改 SQLite，或把原生夹具结果当产品闭环。原生脚本只允许读取窗口/进程和生成证据，不得替代业务动作。

每条任务记录同一链路的可读阶段，不保存正文、截图、完整命令输出、完整 Agent Payload、内部 ID 或本机路径。失败停在真实阶段并进入 Apply；不能为通过验证绕过确认、权限、文件锁、Desktop 租约或 Observe。

快慢脑对照证据采用脱敏 trial manifest：每条只保存任务类别、模式、配对编号、起止时间、显式用户等待、目标核验结果、慢脑调用/handback/动作/unknown 数量和不透明任务引用。汇总器只读该 manifest，计算完成率、Wilson 95% 区间、中位数、最近秩 P95及双成功配对耗时比；它不创建或推进任务，不连接 Driver、Socket 或 SQLite。任务必须由真实 Codex 通过产品 MCP 执行，汇总工具不是第二个 Planner 或验证旁路。

为避免顺序和热缓存偏差，配对按任务类别交替 `fast-slow → slow-only` 与 `slow-only → fast-slow`；每个模式使用独立任务，但复位到同一可观察初态。超时预算在任务开始前固定，失败和 unknown 保留在完成率分母中。涉及用户确认的样本只用于确认链路专项，不进入首轮速度主样本；若后续纳入，必须同时报告含等待与扣除等待两组结果。

### 统一顶部执行浮窗

2026-09-28 用户明确将顶部步骤浮窗从 CUA 扩展到所有任务执行能力。Gateway 已有 `execution_presentation_hint` 覆盖 `browser.execute`、`computer.execute/step`、`plan.execute`、`file.execute`、`document.execute` 与 `command.execute`；桌面组合根必须以该可信提示建立只读展示，而不是只消费 CUA 子集。SQLite 任务/步骤/计划片段仍是权威事实，浮窗只持有短生命周期投影，不新增任务状态。

同一视觉外壳按执行种类区分交互：CUA 标题为“Yonder 正在控制您的电脑”并保留显式“接管电脑”；BUA、Office、Command 和其他非桌面执行标题为“Yonder 正在执行任务”，不提供接管电脑入口。CUA 的接管请求仍只由 `cua_execution_presentation_hint` 授权，非 CUA 即使伪造 UI 调用也必须被宿主拒绝。新的执行请求可刷新当前浮窗投影；终态响应清除对应任务，单次 RPC 返回不伪造终态。

Command 的 `awaiting_user` 发生在执行启动之前，不得复用“正在执行”投影。Desktop 从成功 `command.propose` 回执中只提取 `task_id + command_id`，在同一顶部锚点打开独立的 LocalUser-only 授权卡；卡片再从内存 `CommandApprovalRegistry` 读取结构化预览并提交批准或拒绝。命令正文、参数和环境变量不进入任务事件、Outbox 或执行浮窗，授权卡也不拥有任务状态。批准引用仍只能由归属 Agent 经 Gateway 消费一次。
