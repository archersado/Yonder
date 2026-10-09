# AD-CU-12 有界 Driver 错误可见性

- 状态：Accepted（macOS-only，Windows 暂缓）
- Story：CU-S6
- OpenSpec：`cu-s6-ephemeral-element-handles`
- 日期：2026-10-08

2026-10-09 修订：正式 QQ 音乐样本确认，Sky 动作调用正常返回、动作后 Observe 有效且同一目标仍明确保持原状态时，继续把结果一律归入不可恢复 unknown 会让任务过早结束。该情形现定义为可证实无效果的受限恢复边界；它不同于超时、断连、Observe 失败或目标无法核实。

## 问题

Sky Worker 已把 SDK 错误收敛为 `failure_stage`，但 Adapter 随后统一映射成 `observe-failed` 或 `worker-failed`。正式企业微信任务只能看到 `suspected_noop/observe-failed`，无法区分坐标动作超时、被 Driver 拒绝、调用已返回但界面未变化、视觉引用失效或动作后 Observe 失败。继续猜测坐标和倍率既低效，也可能重放结果待核实的副作用。

## 决定

1. Driver 只向上返回封闭、安全、无业务正文的错误分类：`driver-action-timed-out`、`driver-action-rejected`、`driver-action-unconfirmed`、`driver-target-stale`、`driver-observation-failed`、`driver-transport-failed`。
2. SDK/MCP 原始错误正文、调用参数、应用名、AX transcript、截图、PID、窗口 ID 和完整 Payload 不得进入任务事件、Outbox、SQLite、Gateway、顶部浮窗或普通日志。
3. Worker 在坐标动作调用异常后仍只执行一次动作后 Observe，不重放动作。超时映射为 `driver-action-timed-out`，明确拒绝或无法分类的动作调用错误映射为 `driver-action-rejected`；调用返回但后置事实没有变化映射为 `driver-action-unconfirmed`。
4. 动作前引用、尺寸、transcript 或元素代次不匹配映射为 `driver-target-stale`，且不派发动作。动作前后 Observe 本身失败映射为 `driver-observation-failed`；MCP 握手、认证或执行通道失效映射为 `driver-transport-failed`。
5. 这些分类复用既有 Attempt Unknown Reason、同事务任务事件和有界内存 Driver 轮次，不新增第二日志通道，不让持久化阻塞执行热路径。
6. Rust Protocol 1.49 是 wire enum、Schema 与 TypeScript 的唯一来源。协议 1.48 及以下客户端继续收到兼容的 `timed-out`、`observe-failed` 或 `worker-failed`，不得收到无法解码的新枚举值。
7. 顶部浮窗在当前 Driver 轮次显示对应中文错误；任务面板时间线和 Gateway `task.events` 显示同一错误事实。不得只在开发控制台输出。
8. `driver-action-unconfirmed` 只有在动作调用已正常返回、动作后 Observe 有效、目标元素能按最新 Observation 中的角色与标签唯一重新解析，且该目标仍明确保持动作前状态时，才可在同一 Worker attempt 内恢复。恢复最多追加一次动作派发，必须使用重新解析后的新索引，并再次强制 Observe。
9. 受限恢复不得沿用旧数字索引，不得用于坐标动作、发送、删除、支付、安装、提权、文本输入或任何需要确认/一次性引用的动作。第二次仍无效果、目标不唯一、状态无法证明未变化，或任一调用超时/拒绝/断连时，立即返回原有 `driver-action-unconfirmed`/对应 unknown 并交回慢脑。
10. 两次动作及中间重新解析均属于一个可观察 Driver 轮次；顶部浮窗显示“首次未生效，正在重试（1/1）”，最终事件仍只记录收敛后的单个 attempt 结论，不新增持久化重试状态或第二事实源。

## 取代关系

本决定补充 AD-CU-09～AD-CU-11 的 unknown/handback 规则。“结果待核实的副作用不得自动重试”继续适用于超时、断连、Observe 失败、坐标动作与任何无法证明未发生的副作用；仅把满足第 8～9 条、可由新鲜 Observe 证实未产生效果的非敏感元素激活从不可核实 unknown 中拆出。动作回执不是成功、每次派发后 Observe 和 Sky 单执行栈保持不变。

## 验收

- Worker 隔离夹具分别覆盖坐标动作超时、拒绝、成功回执但无界面变化和视觉引用失效。
- Adapter 将受信 `failure_stage` 映射为封闭内部枚举，拒绝任意字符串穿透。
- Protocol/Schema/TypeScript、Gateway 1.49 与旧版本降级、SQLite unknown reason 往返均有测试。
- 顶部 Driver 轮次与任务面板展示安全中文错误，不包含 SDK 原文或业务内容。
- Worker 夹具覆盖首次元素激活无效果后按角色/标签重新解析新索引、仅重试一次并成功，以及第二次仍无效果后交回；断言坐标与敏感/一次性动作零重试。
