# AD-EX-05 快脑步骤完成核验

状态：Accepted  
日期：2026-10-08  
关联：EX-S2、CU-S5、TM-S3、AD-EX-01、AD-TM-24

## 决策问题与来源

2026-10-08 真实 QQ 音乐 CUA 样本证明，Driver 的动作回执与通用
`action-succeeded / observe-valid` 只能说明动作和 Observe 已发生，不能说明当前计划步骤
的语义效果已经成立。例如播放器可能已进入播放态，但播放的并非当前步骤指定的歌曲。
若每个此类判断都交回外部慢脑，计划片段会退化为逐动作慢脑循环。

用户明确决定：Jev 快脑可以判断当前步骤是否完成。本决定只授权计划片段内的步骤
效果核验，不改变开放式用户目标的最终核验所有权。

## 决定

每个真实动作后仍由 Driver 强制 Observe。Application 先校验动作结果和封闭结构事实；
结构事实成立但不足以证明步骤语义时，Application 可以调用 Jev 对当前步骤作三值核验：
`complete`、`incomplete`、`uncertain`。

快脑核验输入只能包含：任务/步骤身份、用户可见步骤标签、已派发的封闭动作语义、
计划声明的预期 Observe、Driver 结构事实，以及本轮新鲜 AX/UIA transcript 派生的有界、
脱敏、只驻留内存的观察摘要。不得发送截图、坐标、Driver 参数、消息正文、接收人、
密码框、安全界面、完整模型 Payload 或历史任务上下文。摘要不得写入 SQLite、事件、
Outbox 或日志。

高置信 `complete` 才允许推进当前槽位并投影成功图标。高置信 `incomplete` 停止当前
片段并形成可观察交回；`uncertain`、低置信、服务不可用、观察摘要缺失或响应不合法也
安全交回慢脑。`unknown` 副作用仍禁止自动重试；本决定不授权快脑生成新动作、修改
计划、重放动作或绕过确认。

Jev 的步骤完成结论不是用户目标完成事实。计划片段耗尽后仍按 Accepted AD-TM-24
进入等待目标核验，归属慢脑必须基于最新 Observation 经同一 Gateway 提交
`task.goal.verify`，随后才可 `task.complete`。

## 依赖与实现边界

Application 拥有核验请求、置信阈值、三值解释和交回策略；Jev Adapter 只实现同一
决策 Port 的远端调用，不持有任务状态。Driver 不调用 Jev，React UI 不判断完成。
顶部浮窗只显示“快脑正在核验步骤”“步骤已完成”或最小交回原因，不展示观察摘要、
置信度、模型原文或思维链。

该能力先在 EX-S2 的 macOS CUA 基线实现并验证；Windows 继续按既有决定暂缓。推广到
BUA、Document 或 Command 前必须分别证明其观察摘要满足数据边界和副作用规则。
