# Proposal：CU-S2 元素优先与窗口视觉证据

关联 Story CU-S2、Accepted AD-CU-04/AD-AG-09及EX-S2计划片段。Architecture Impact：architecture-change（Computer Observation与Gateway响应协议1.34）；不新增任务状态、持久化、Planner或Driver。

## Why

当前Worker在目标元素缺失时只能把动作归为失败，慢脑拿不到同一可信窗口的可视事实；同时后台应用动作若错误观察当前桌面，会把其他前台应用当作目标证据。需要让元素路径保持轻量，并仅在动作尚未执行的目标解析失败时返回一次窗口截图供慢脑replan。

## What Changes

- Worker对窗口级动作先读取不含截图的新鲜AX/UIA元素，唯一元素可解析时直接使用。
- 元素缺失或不唯一时不派发副作用，只采集一次同窗口临时截图并返回已知拒绝。
- 协议1.34允许`task.plan.execute`携带可选临时Computer Observation；旧协议保持原响应。
- 动作结果`unknown`但同次Observe有效时保留临时观察，不据此提升成功或自动重试。
- `launch_app`后的窗口级动作和Observe绑定刷新后的同一应用身份；只有显式desktop scope可观察全桌面。

## Non-goals

不扩展计划动作语义、不生成坐标、不把截图交给Jev、不实现发送确认、不持久化截图、不启用Windows路径。
