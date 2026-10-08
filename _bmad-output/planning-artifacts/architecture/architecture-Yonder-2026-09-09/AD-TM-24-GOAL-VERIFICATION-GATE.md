# AD-TM-24 目标状态核验与任务完成门禁

状态：Accepted（2026-10-07，用户要求参考 Codex Computer Use 的执行正确性闭环）。关联 CU-S5、TM-S9、EX-S2、AG-S1。Architecture Impact：architecture-change（任务完成语义、Gateway 协议、活动运行时事实与事后投影）。

## 问题

现有执行链把 Driver 动作已返回、动作后 Observe 有效和用户目标已完成混为一体。计划片段槽位耗尽仅代表没有更多候选；`task.complete` 也只检查最近一步成功，因而可能在“打开了企业切换面板”后错误完成“切换企业并发送消息”的整项任务。

## 决定

- 区分三类事实：动作已投递、步骤效果已 Observe、用户目标已核验。前两者不能替代第三者。
- 计划片段耗尽后进入“等待目标核验”，不自动成为任务完成依据。归属慢脑必须基于同一 Gateway 返回的最新 Observation 提交 `task.goal.verify`。
- 目标核验只接受封闭结论 `achieved` 或 `not-achieved`，携带任务 CAS 序号和所依据的 Observation/attempt 结果序号；不接受模型思维链、截图正文或自由文本证明。
- `achieved` 生成不可复用的 `verification_id`。`task.complete` 必须引用当前最新、属于同一任务和归属 Agent、结论为 achieved 的核验；核验之后发生任何新的执行或控制事件都会使其失效。
- `not-achieved` 保持任务非终态并显式交回慢脑重新 Observe/规划；不得重放 unknown 副作用。
- 活动任务的核验事实由 `ExecutionRuntime` 持有并即时投影 UI/Gateway；SQLite 仅作为异步事后投影和兼容链检查点，不得阻塞 Driver 或下一次 Observe。
- CUA、BUA、Document/Office 与 Command 共用该完成门禁；首个产品验证以 CUA 为样本，不为单一 Driver 建立私有状态机。

## 交互与安全

顶部浮窗在片段耗尽后显示“等待慢脑核验最终结果”；核验未通过显示“目标未完成，已交回慢脑”，通过后才允许显示任务完成。浮窗只显示结论和步骤状态，不显示模型推理、截图、正文或完整 Payload。

## 验证

独立 Verification Goal 必须证明：无核验不能完成；陈旧核验不能完成；`not-achieved` 会交回且可提交新片段；最新 `achieved` 核验可完成；数据库延迟不阻塞活动运行时；macOS 正式 Gateway 样本不会把中间界面误判为用户目标完成。Windows 按用户决定暂缓。

## 2026-10-08 修订：Driver 事实与慢脑结论分离

用户进一步明确，完成判断必须由归属慢脑读取 Driver 返回的最新状态后作出，Application 不得根据动作种类硬编码推导 `target-resolved`、`composer-ready`、`delivery-confirmed` 或其他业务寓意。

- 计划内只允许使用 Driver 直接报告的结构事实：动作是否明确成功、Observe 是否有效、目标窗口是否可见。Application 可以校验这些事实的新鲜度、身份和安全边界，但不得把 `activate-control` 等动作种类翻译成业务完成事实。
- Driver 返回的受控 Observation 继续经同一 Gateway 提供给归属慢脑。目标歌曲是否播放、企业是否切换、消息是否送达等开放业务状态，只能由慢脑基于最新 Observation 提交 `task.goal.verify`。
- 旧的业务寓意 Observe 条件保留 wire 解码兼容，但不再由 Application 合成，也不能成为步骤成功或任务完成依据。新计划使用结构事实；无法从 Driver 状态直接证明时返回慢脑判断。
- 本修订不允许慢脑绕过安全门禁，也不允许重放 `unknown` 副作用。`task.goal.verify` 仍只保存封闭结论，不保存推理正文。
