# EX-S2 macOS 计划片段执行 Verification Goal

## 2026-10-08 事件驱动交回边界复核

状态：PASS。

- `task.plan.execute` 已统一进入 `ExecutionRuntime`：交回原因先成为内存 `HandedBack` 事件和活动任务快照，Driver→Observe→下一槽位或交回不等待 SQLite、Outbox、慢脑读取或唤醒投递。
- 独立 `RuntimeProjectionWorker` 通过有界通知队列批量投影 SQLite/events/Outbox，失败保留内存积压并重试，成功后才连续 ACK checkpoint；达到硬限只在下一副作用前进入 `persistence-backpressure`。
- Desktop 的慢脑唤醒继续发生在 Gateway 响应写回与执行锁释放之后，顶部展示读取同一内存投影；无连接或投递失败不反写 Driver 结果，也不重试副作用。
- 当前 `dev` 复核：`cargo test --offline --locked -p yonder-application --lib` 74/74 PASS，覆盖 `reducer_advances_without_a_persistence_ack`、`plan_cursor_advances_and_hands_back_without_a_persistence_ack`、projector失败重试以及 `unknown_result_and_handback_are_immediate_runtime_facts`；`cargo test --offline --locked -p yonder-desktop --lib` 28/28 PASS，覆盖独立SQLite连接投影与交回/重规划顶部汇总。
- 本结论关闭任务清单中的事件驱动交回实现项，不替代下述正式QQ音乐步骤核验与企业微信语义/确认样本。

## 2026-10-08 AD-EX-05 步骤核验增量

状态：代码与合约验证通过；正式 QQ 音乐原生回归 BLOCKED。

- `cargo test --offline --locked -p yonder-application --lib`：当前 `dev` 74 项通过，覆盖 Jev 三值步骤结论、低置信交回和脱敏观察摘要。
- `cargo test -p yonder-adapters --lib`：本增量相关测试通过；完整运行 77/78，唯一失败的既有 Command 临时 PID 时序用例单独重跑通过。
- 正式 `jev_worker.mjs` 配合固定 0.6.0 SDK fixture 已验证 `verify-step` 返回 complete，且不生成动作。
- `python3 apps/desktop/check-cua-package.py`：Sky 0.7.1、Jev 0.6.0、签名 Bridge 和打包 Worker 全部通过。
- 正式产品任务 `task_fc1d0e8c5f4db936cec416645803e91b` 经 `yonder mcp → Gateway` 创建；GUI 重启后两次 `launch_app(com.tencent.QQMusicMac)` 均在进入 Jev 前返回 `worker-failed`，序号 10/15，任务已于序号 17 正式取消。故本轮不能声称真实 QQ 音乐步骤核验通过，待 Sky Worker 会话恢复后重跑。
- 2026-10-08 MCP重新连接后，以正式产品任务 `task_b9e7241c78d7880acf822d6b9c7c2d56` 完成三版受限片段复验。`launch_app`与搜索入口聚焦分别在序号5/9得到`observed(action_succeeded=true, observe_valid=true)`；随后文本输入、绑定新鲜Observation的元素点击及封闭`SPACE`激活分别在序号13/17/21得到`unknown(driver-action-unconfirmed, observe_valid=false)`。每次均先交回慢脑、使用新plan/step/attempt身份且未自动重试旧副作用，任务最终在安全边界取消于序号23。
- 因三个候选都在Driver结构事实成立前停止，正式样本尚未进入Jev三值步骤核验，不能把自动化通过外推为产品PASS。阻断点已收敛为CU-S4的Sky动作确认能力；结构化证据见 `apps/desktop/evidence/ex-s2-qqmusic-step-verification-20261008/result.json`。修复CU-S4并重建正式产品后重跑，本节继续保持BLOCKED。

状态：macOS 同步连续执行验证通过（2026-09-28）；Windows 按用户决定暂缓。

## 目标

在 macOS 正式桌面组合根中，以同一已认证 Gateway 会话提交一个受限 CUA 计划片段，并在一次 `task.plan.execute` 调用内连续执行多个槽位。证据必须同时证明：单一已验证慢脑候选不调用 Jev；多候选时 Jev 只返回已提交候选；每个动作均经既有桌面租约与 Observe，成功才推进片段游标；低置信、预算耗尽或异常交回会写入任务事件及 Outbox；不保存模型输入、截图、键入内容或完整 Agent Payload。

## 已通过的结构验证

- `cargo test -p yonder-application`：44/44；其中 Gateway 用例确认 `task.plan.submit` 与 `task.plan.execute` 必须先协商协议 1.31，旧版协商或不可用组合根均会被拒绝；单候选选择测试以会 panic 的 Jev Port 证明确定性步骤不调用 Jev；
- `cargo test -p yonder-adapters`：67/67；其中 SQLite 用例确认片段不可变重放、CAS、槽位推进及交回均与事件/Outbox 同事务；连续执行用例证明两个槽位在一次调用内依次形成 Observe/停止边界，步数上限为 1 时只派发第一步并原子交回；
- `cargo test -p yonder-desktop --lib`：11/11。

这些结果只证明协议、Application、SQLite 与桌面组合根可构建并保持既有回归；不替代真实 Keychain 凭据、辅助功能授权、前台桌面目标和远端 Jev 的 macOS 原生样本。

## 已通过的 Gateway 原生样本

2026-09-27 以正式 debug bundle、隔离 `/tmp/ex2-*` HOME 和私有 stdio 运行 `apps/desktop/check-plan-fragment-submit-macos.py`。自动登记的 `local-test-agent` 成功完成协议 1.31 握手、任务创建和 `task.plan.submit`；结果为 `hello_ok=true`、`plan_capabilities=true`、`accepted=true`、`sequence_advanced=true`。样本没有调用 Jev、未派发 CUA 动作、未保存候选参数或用户内容，因此只证明 EX-S2 的 Gateway/CAS/SQLite/Outbox 提交子范围。

## 待运行原生样本

需在用户主动提供的可控测试窗口中，以无敏感正文的 `computer_click` 候选运行一次。输出仅保留 task/plan/sequence、选择类别、Observe 成败、Outbox 数量与权限状态；若缺少 Keychain API Key、辅助功能权限、网络或测试窗口，应记录为环境阻塞，不将 EX-S2 标为 PASS。Windows 仍按用户决定暂缓。

2026-09-27 已以 `apps/desktop/check-plan-fragment-execute-macos.py` 运行固定输入的隔离窗口样本。新增单窗口、每进程唯一标题的 EX-S2 fixture；正式 `MacWorkFocus` 优先将 AX `AXWindowNumber` 与已捕获的 WindowServer `window_id` 绑定，只在属性缺失时回退至同一 AX 引用去重，仍拒绝不同物理窗口的歧义。Gateway 1.31 握手、Jev 配置和 `task.plan.execute` 能力均已确认可用。修复 worker 错误地偏向 `handback` 的提示词后，Jev 已选择唯一已提交候选；HID 监控不再把窗口激活产生的鼠标移动归类为用户接管，样本已取得一次 CUA 已观察并推进片段槽位的 `advanced` 结果。标准 AppKit 文本字段不能稳定反映 AX 写入，验证 fixture 已改用显式 `setAccessibilityValue` 控件。夹具现在在未创建任务、未派发 CUA 前有限次重捕获 AppKit 首次激活的瞬态几何变化，并以 `input_matches` 作为唯一写入断言；修复后焦点预检通过，实际远端调用仍可合法地选择 `handback`。尚未取得包含新控件匹配断言的一条完整 PASS；Windows 仍按用户决定暂缓。

本轮原生样本还复现了 `unknown` 结果落库后用户输入中断与片段交回之间的 CAS 竞争。Application 现会在交回 CAS 冲突时重新读取任务；若任务已离开 `created/running`，返回该中断事实而不追加交回事件，其他冲突仍按失败处理。

2026-09-27 验证器已改为临时 HOME 的全新任务库，仅在迁移完成后复制 Jev 的非秘密配置，凭据仍只由 Keychain 提供；并为 fixture、AX 辅助进程和全流程加入有界读取、25 秒总时限与进程清理。该隔离组合根在 `gateway.hello` 前以 `SIGABRT` 退出；macOS 崩溃报告定位为 Tauri/tao 的 `did_finish_launching` 回调 panic。该问题发生在 Gateway、Jev、任务创建和 CUA 之前，不能作为 EX-S2 失败证据，也不能以回退到主任务库的方式绕过。待桌面启动隔离问题另行修复后，重跑此独立 Goal；Windows 仍按用户决定暂缓。

2026-09-28 验证改用既有 `local-agent-gateway` 无 GUI 独立宿主初始化并服务临时 TaskHost 目录；debug 宿主仅可通过绝对、非链接的 `YONDER_TEST_CUA_RESOURCE_DIR` 读取已签名 bundle 内的 CUA/Jev 资源，正式资源发现路径不变。该路径避免 Tauri 启动崩溃，已到达 `gateway.hello`、协议 1.31、计划提交和 `task.plan.execute`，能力为可用；执行返回安全停止 `-32012`，尚未取得 Jev 已派发的动作。临时任务库与用户 HOME 分离，Keychain 凭据仍不导出；独立宿主的 Keychain 访问身份与 GUI bundle 的差异仍待单独验证。

2026-09-28 已以正式 debug GUI bundle、真实 App Data 与 Keychain 身份运行两条独立固定 fixture 样本。Jev 面板配置为 remote、CUA 启用、单次预算 60 秒；此前 TaskHost 将 Adapter 硬编码为 3 秒，远端选择在该界限后被错误映射为 `-32012`。Adapter 现取面板预算与 30 秒硬上限的较小值，不重试。修复后两次均完成 `task.plan.execute` 并返回合法 `handback`，证明 Jev 调用已完成且未超时；模型未选择已提交动作，故未派发 CUA，尚未构成完整 PASS。

验证 worker 已按 TypeSafe SDK 的 `choice` 约定为每个候选提供说明：可派发候选说明其状态为 true 时可执行，`handback` 明确限定为没有可派发候选时才选择。正式 GUI 新样本仍返回 `handback`；该交回保持为 Jev 的安全决策，不以测试目的删除候选、伪造模型响应或直接绕过 Jev 派发 CUA。

## 2026-09-28 同步连续执行终局证据

按用户确认的快慢脑职责修订 AD-AG-09 后，`task.plan.execute` 已改为在单次 Gateway 调用栈内同步消费片段剩余槽位。每轮仍复用 `execute_agent_step`，只有动作结果为 `Observed(action_succeeded=true)`、attempt 已推进到 `stopped` 且片段槽位 CAS 成功后，才读取并执行下一槽位；响应返回后没有后台执行。

正式 debug `Yonda.app` 重新构建并临时签名后，以 `apps/desktop/check-plan-fragment-execute-macos.py --formal-gui` 在唯一隔离 AppKit 窗口执行两个真实 CUA 动作：`Tab` 后 `Escape`。两个槽位均为慢脑提交的单一已验证候选，因此没有请求 Jev；同一次 `task.plan.execute` 返回 `fragment-complete`，随后 `task.complete` 成功。结构化结果为：

```json
{"platform":"macos","recording_started":false,"phase":"completed","plan_execute_capability":true,"continuous_native_actions_observed":true,"task_completed":true,"passed":true}
```

本样本不包含联系人、消息正文或发送动作，不启动 Recording。此前自定义文本夹具的 `type_text` 返回 `action_succeeded=false` 时，系统已正确停止连续推进并交回慢脑；该负向事实同时证明失败动作不会被当成成功槽位跨越。

真实企业微信样本进一步证明单槽位正向链路：`launch_app(com.tencent.WeWorkMac)` 在 1466ms 内完成 Observe，片段返回 `fragment-complete`，任务最终 `completed`，没有消息输入、发送或 Recording。四槽位的「启动→前置→Tab→Escape」样本目前在跨 Space 前置阶段被 CU-S2 的 HID 来源分类安全中断；任务成为 `interrupted`，没有跨越失败槽位。该结果不影响隔离窗口连续执行 PASS，但企业微信真实连续样本仍保持待完成。

## 2026-09-29 主动慢脑唤醒与顶部汇总证据

正式 `dev` debug bundle 以协议 1.33 运行。任务 `task_e5a58104f7e2a07f5c6f18984521238c` 的无副作用失效应用样本在序号 7 产生 `handback` 后，Desktop在 Gateway 响应写回和锁释放后，经同一 AG-S5 `agent.input(source=replan)` 会话把任务号、序号及有界原因投递给唯一归属的 `codex-cli`。当前 Codex 活动 turn 实际收到输入；慢脑随后经正式 MCP/Gateway 读取 7 条新鲜事件，未重试失败动作，并提交含 2 步的 `replan-wake-safe-followup`，序号推进到 8。

桥接读取活动 turn 时不再请求完整历史，而是先以 `thread/read(includeTurns=false)`读取状态，再以`thread/turns/list(limit=1, itemsView=notLoaded)`取得 steer 所需 ID。由此避免长会话完整历史超过 App Server 帧上限；隔离桥合约同时覆盖活动 turn 的 `turn/steer` 与空闲 turn 的 `turn/start`。

原生 AX 验证和窗口截图证明顶部浮窗同时出现“慢脑已提交重新规划：2 个步骤”及两条新步骤；证据位于 `apps/desktop/evidence/ex-s2-replan-wake-20260929/`。后续执行中新的 TextEdit 启动尝试返回 `unknown(worker-failed)`；慢脑读取 13 条事件后没有自动重试该结果待核实的副作用，验证任务按测试清理流程取消。该后续 Driver 结果不改变本 Goal 对“交回主动唤醒、同 Gateway 重新规划、顶部汇总”的 PASS 结论，也不被记录为动作成功。

## 2026-09-30 unknown 后重新规划执行槽证据

正式企业微信样本复现了 `unknown(observe-failed)` handback 后旧 attempt 永久占据执行槽的问题。SQLite schema 23 现以独立 `handback_sequence` 保留原始 unknown 结论并形成停止投影；旧片段游标不推进，桌面任务租约不释放。Adapter 回归证明归属慢脑读取事实后，可以同一任务、同一租约、全新 step/attempt 提交和执行新片段；复用旧 step id 仍被拒绝。

正式 MCP/Gateway 样本中，任务在序号 17、23、33、43 多次按新鲜截图交回并成功启动新片段，证明状态机不再卡死。Worker 回收后新片段先以封闭 bundle id 重绑企业微信，截图从错误的当前前台 VS Code 恢复为企业微信。后续 trycua 像素文本动作仍返回不可核实且截图无文本，系统继续安全交回并未发送消息；该 Driver 正向能力缺口由 CU-S4 Goal 继续跟踪。

## 2026-10-07 Jev 新鲜 Observe 候选门禁

Application 不再把计划候选自行声明的 `preconditions` 当作已验证事实。一次 `task.plan.execute` 调用内，每个成功动作的强制 Observe 只派生封闭布尔事实；下一槽位先由 Application 过滤：无合格候选直接交回、唯一合格候选确定性执行、至少两个合格候选才连同本轮事实摘要调用 Jev。动作成功后再次复核 `expected_observe`，不满足时不推进片段。Jev Worker 不接收 transcript、截图、坐标或业务正文。

结构回归通过：`cargo test -p yonder-application -p yonder-adapters` 为 Application 63/63、Adapter 78/78；新增用例证明多候选缺少新鲜事实时不调用 Jev、唯一合格候选不调用 Jev、多个合格候选只向 Jev 暴露已验证候选及封闭事实。`node --check crates/adapters/src/jev_worker.mjs` 与 `openspec validate ex-s2-plan-fragment-macos --strict` 通过。

合入 `dev` 并重建正式 debug `Yonda.app` 后，使用正式产品 MCP/Gateway 创建任务 `task_f7b49cc5ccec5f278de0171f0dffbcbf` 与 `task_5ecf9c907cee3f6f96d66e9c0b3fbfcc`。两条样本都在首个单候选 CUA 动作产生真实 Observe 之前返回 `unknown(worker-failed, observe_valid=false)`，序号均停在 6 并安全交回；因此没有进入第二槽位，不能作为远端 Jev 正向选择证据。验证任务已取消。该阻塞属于当前 Sky CUA Worker 启动/执行链路，不允许用直接 Driver、伪造 Observe 或绕过 Gateway 的方式替代；本轮只关闭结构实现项，真实产品正向样本仍待 CUA Worker 恢复后补跑。

## 2026-10-08 正式产品快脑派发复验

Sky MCP 冷启动预算修正后，正式 `Yonda.app` 通过安装包内 `yonder mcp → Gateway → Application → Jev → Sky` 运行任务 `task_6d52a58506b048f4a30da849d7b97ffe`。首槽位 `launch_app(com.apple.TextEdit)` 在序号 5 形成 `observed(action_succeeded=true, observe_valid=true)`；第二槽位的两个合法候选经本轮 Observe 筛选后进入 Jev。修正前模型会把与动作并列的 `handback` 选中并在动作前交回；修正后 `handback` 由 Application 作为低置信/异常的唯一安全边界，不再进入模型候选集。本次 Jev 实际选择并派发了第二槽位：序号 8 声明快脑步骤，序号 9 开始 `attempt_9`。

第二个 Sky 动作的后置观察在序号 10 返回 `unknown(observe-failed, observe_valid=false)`，片段于序号 11 安全交回，未将动作或任务误报为成功。因此本样本证明“新鲜 Observe → Jev 选择 → 真实派发”链路 PASS；Driver 后置 Observe 的正向完成率仍须由 CU-S4 继续收敛，不在本结论中提升。验证后任务于序号 12 取消清理。Windows 仍按用户决定暂缓。

## 2026-10-08 新鲜 AX 动作空间终局证据

依据用户修订，不再以人为构造的应用恢复路径触发快脑。正式 debug `Yonda.app` 经安装包内 `yonder mcp → Gateway → Application → Jev → Sky` 运行任务 `task_91bc7bd44fe101091c0e0798104fcda9`。慢脑片段第一槽位只负责打开已签名隔离 App 并形成新鲜 Observe；第二槽位只提交一个 `activate-control + click {}` 语义模板“点击执行验证主动作按钮”。Application 从同次 AX Observe 展开多个真实按钮候选，按本步意图把封闭角色、最长 128 字节短标签和不透明候选 ID 交给 Jev；未向 Jev 暴露 transcript、截图、坐标、PID、窗口身份或 Driver 参数。

Jev 选择目标按钮后，Application 仅在当次内存请求注入 `observation_ref + observed_element_index`，Sky 在点击前复核句柄属于最新 Observe。夹具收到真实点击并把 AX 标签变更为“验证主动作已完成”，动作后 Observe 独立确认状态变化。结构化结果为：

```json
{"platform":"macos","entry":"bundled-yonder-mcp","passed":true,"task_id":"task_91bc7bd44fe101091c0e0798104fcda9","disposition":"awaiting-goal-verification","elapsed_ms":19762,"primary_selected":true,"event_count":12,"observed_attempts":2}
```

随后归属慢脑以最新 observation sequence 提交 `task.goal.verify(achieved)`，再携同一 verification ID 完成任务。该样本证明“唯一慢脑语义模板 → 新鲜 AX 多候选 → Jev 自然选择 → 临时句柄注入 → Driver 复核并执行 → 动作后 Observe → Goal Verification → completed”正式产品链路 PASS；它不声称单次样本已经证明快慢脑相对纯慢脑的统计性能优势。Windows 仍按用户决定暂缓。
