# 浏览器任务

本模块基于 `ego-browser` Skill `2.0.0`（2026-09-09）审阅提取。ego-lite 是浏览器 Runtime；Yonder 仍负责归属任务、Task Space 引用、执行状态、Observe、交接和终态。

## 执行顺序

1. 在唯一 Yonder 任务上调用 `browser_execute(create)`，取得并保存 Yonder 登记的 ego-lite Task Space 引用。
2. 只恢复该引用对应的 Task Space。一个用户目标只使用一个空间；失败、超时或新一轮执行也不得另建空间规避问题。
3. 复用空间内已有 Page。普通 DOM 页面优先 snapshot 与语义选择器；canvas、富文本、表格等缺少有效语义时才使用截图和坐标。
4. 当前页面状态足以决定多个连续动作时，在同一执行轮完成这些动作，再等待最终可观察条件并取得一次新 Observe；中间状态会改变后续选择时才额外 Observe。
5. 每次有效页面动作后，用 `browser_execute(observe)` 将受限事实提交给 Yonder。动作回执只证明动作已派发，不等于结果成功。
6. 需要用户登录、处理权限或接管时，先调用 Yonder 的 `browser_execute(hand-off)`，停止 Agent 页面动作。用户交回后调用 `browser_execute(take-over)` 并恢复同一空间。
7. 目标完成后调用 `browser_execute(finish)`；只保留用户明确要求的页面，再由归属 Agent提交任务终态。

## 围栏

- 不直接创建未关联 Yonder 任务的 ego-lite Task Space。
- 不启动 Playwright 或第二个浏览器，不猜测 ego-browser 未公开的 API。
- 选择器必须唯一；动作未产生预期结果时先 Observe，不盲重试或立刻降级坐标。
- 用户接管、空间失活或归属变化时立即停止，不绕过控制状态。
- BUA 动作、页面正文、截图和完整 URL 不写入 Skill 日志；仅把 Yonder 接受的有界事实作为任务证据。
