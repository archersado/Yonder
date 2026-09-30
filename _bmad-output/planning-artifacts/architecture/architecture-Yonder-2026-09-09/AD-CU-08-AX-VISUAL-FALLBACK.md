# AD-CU-08 CUA 元素优先与同窗口视觉降级

- 状态：Accepted（macOS-only，Windows 暂缓）
- Story：CU-S4
- OpenSpec：`cu-s4-ax-visual-fallback`
- 日期：2026-09-30

## 问题

正式 Yonder 企业微信和 QQ 音乐样本证明：应用可以被可信启动并绑定精确窗口，但 WebView、自绘或跨进程渲染界面可能只公开空或不稳定的 AX 元素树。现有 Worker 在正常路径关闭截图；唯一元素后台动作被 Driver 拒绝或判为不可核实时，后置 Observe 仍可能只返回空元素且不携带截图。TM-S9 运行时还会丢弃 `UnknownObserved` 已取得的视觉 Observation，导致归属慢脑没有新鲜事实可用于坐标重规划。

后续同实例对照证明 Codex Computer Use / Sky 能取得完整应用级AX transcript，而trycua仍为空。依据Accepted AD-CU-09，Sky取代trycua成为macOS唯一产品Driver；本决定的元素优先、同目标截图降级、每步Observe、unknown不重试和单一桌面租约保持不变，不保留第二执行栈。

## 决定

1. CUA 仍先读取已绑定精确窗口的 AX 状态；元素唯一且动作 `confirmed` 时不请求截图。
2. AX 树为空、元素缺失/多义、Driver 返回 `refused`、`partial`、`unverifiable`、`suspected-noop`，或动作后无法形成受支持事实时，Worker 只补采同一 PID/window 的窗口截图。不得改采当前前台或全桌面，不得重放原动作。
3. 已取得的视觉 Observation 即使动作结论为 `unknown` 也必须沿 Application/Gateway 返回给归属 Agent；SQLite 投影滞后不得阻塞该运行时事实。
4. Agent 可依据该临时窗口截图提交窗口局部坐标动作。通用 `computer.step` 的坐标 `type_text` 与受保护计划坐标文本一样，由 Worker 注入精确窗口 target、受监管 session 和 `foreground`；Agent 仍不能提供 PID、window、session、snapshot 或 element token。
5. 已 Observe 但动作失败只表示到达安全步骤边界，不得在顶部浮窗显示成功图标；`unknown` 保持未核实并交回。两者均不得自动重试。
6. 截图继续位于 Yonder 私有临时目录，受既有大小、类型、清理与协议版本门禁约束；日志、事件和顶部浮窗不显示截图、输入正文、AX 树或完整 Driver Payload。
7. 协议 1.40 为非特定应用的桌面计划新增封闭的“聚焦控件、输入文本、激活控件”语义。归属慢脑必须从同一 Gateway 一次提交完整有界片段；宿主在派发首步前投影全部槽位，并按AD-CU-07在顶部浮窗显示总数及当前附近至多四步，后续只更新步骤状态。支持 1.40 时不得为了兼容而把片段拆成多个 `computer.step`。
8. 封闭的`cmd+f`搜索聚焦与窗口坐标动作统一采用动作级foreground和精确窗口target。它不是独立`bring_to_front`步骤；Driver必须在动作后恢复原工作窗口。缺少foreground Schema或后置确认时交回慢脑，不以后台请求回执推进。

## 排除项

- 不引入截图常开、第二模型循环、第二 CUA Driver、AppleScript 或桌面全局坐标兜底。
- 不把视觉推断塞入 Driver；坐标选择仍由经 Gateway 接入的归属慢脑完成，Jev 只在已验证片段内选择候选。
- 不改变副作用 `unknown`、发送确认或结果待核实不重试语义。
- Windows 原生证据按用户决定暂缓，macOS 结论不得外推。

## 验收

- 空 AX 树的 confirmed 动作只额外产生一次同窗口只读截图。
- 动作未确认时返回同窗口截图，不重放动作；运行时保留 `UnknownObserved` 视觉证据。
- 通用窗口坐标文本由 Worker 强制 foreground 和精确窗口 target。
- confirmed 且元素充分的正常路径不采集截图；失败步骤不显示完成状态。
- 隔离测试覆盖错误目标、截图失败、unknown、清理和敏感数据边界；正式 QQ 音乐样本通过 Yonder Gateway 复验。
