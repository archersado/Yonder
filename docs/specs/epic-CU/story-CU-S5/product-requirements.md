# CU-S5 产品需求

## 问题与目标

Yonder 目前可能把“已打开企业切换面板”等中间步骤当作整项任务成功。产品必须像持续 Computer Use 循环一样，在动作后读取新鲜状态，并在终结前单独判断用户目标是否真正达成。

## 需求来源与分类

- 原始需求：产品简报“执行原则”和“任务可见可控”要求每步 Observe、失败可交回、用户看到真实状态。
- 后续用户变更（2026-10-07）：要求参考 Codex 的循环逻辑，修复企业切换未完成却结束的问题，并具备类似的执行正确性保障。
- 架构约束：AD-CU-04 强制动作后 Observe；AD-EX-01 规定 Jev `DONE` 不替代归属 Agent 终结；AD-AG-07 规定首次计划、replan 与最终决定均走同一 Gateway；AD-TM-23 规定活动状态由事件驱动 Runtime 持有。
- 已接受设计：AD-TM-24 规定显式目标核验事实和完成门禁。

## 验收条件

- GVR-01：片段槽位耗尽只进入等待目标核验，不自动代表用户目标完成。
- GVR-02：归属慢脑通过 Gateway 提交封闭的 achieved/not-achieved 核验，并绑定当前 CAS 序号与最新 Observation/attempt 结果序号。
- GVR-03：`task.complete` 必须引用最新 achieved 核验；无核验、not-achieved、他人核验、陈旧核验均拒绝。
- GVR-04：核验之后的任一新动作、控制或计划事件使旧核验失效。
- GVR-05：not-achieved 保持任务可继续并交回慢脑；unknown 副作用不得自动重试。
- GVR-06：核验进入 ExecutionRuntime 事件流并异步投影；SQLite 延迟或失败不得卡住当前 Driver→Observe。
- GVR-07：CUA、BUA、Document/Office、Command 使用同一终结门禁，不在 Driver 内实现目标判断。
- GVR-08：顶部浮窗显示等待核验、核验通过或交回重规划，不展示思维链、截图正文和完整 Payload。
- GVR-09：Rust 协议是 Schema 与 TypeScript 的唯一来源；旧客户端收到明确能力不支持错误。
- GVR-10：macOS 正式 Gateway 样本证明中间界面不能完成任务，最新目标状态通过后才能完成；Windows 暂缓且不得外推。

## 验收映射

| 来源 | 验收 |
|---|---|
| 产品简报与补充材料 | GVR-01、GVR-05、GVR-08 |
| 2026-10-07 用户变更 | GVR-01～GVR-04、GVR-10 |
| AD-CU-04、AD-EX-01、AD-AG-07 | GVR-02、GVR-05、GVR-07 |
| AD-TM-23、AD-TM-24 | GVR-03、GVR-04、GVR-06 |
| 协议单一来源约束 | GVR-09 |
