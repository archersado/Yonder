# EX-S2 macOS 计划片段执行 Verification Goal

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
