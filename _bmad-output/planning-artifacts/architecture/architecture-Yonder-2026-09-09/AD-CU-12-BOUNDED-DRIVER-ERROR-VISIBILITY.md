# AD-CU-12 有界 Driver 错误可见性

- 状态：Accepted（macOS-only，Windows 暂缓）
- Story：CU-S6
- OpenSpec：`cu-s6-ephemeral-element-handles`
- 日期：2026-10-08

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

## 取代关系

本决定补充 AD-CU-09～AD-CU-11 的 unknown/handback 规则，不改变“动作回执不是成功”“每步动作后 Observe”“结果待核实的副作用不得自动重试”和 Sky 单执行栈。

## 验收

- Worker 隔离夹具分别覆盖坐标动作超时、拒绝、成功回执但无界面变化和视觉引用失效。
- Adapter 将受信 `failure_stage` 映射为封闭内部枚举，拒绝任意字符串穿透。
- Protocol/Schema/TypeScript、Gateway 1.49 与旧版本降级、SQLite unknown reason 往返均有测试。
- 顶部 Driver 轮次与任务面板展示安全中文错误，不包含 SDK 原文或业务内容。
