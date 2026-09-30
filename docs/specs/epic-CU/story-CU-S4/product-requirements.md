# CU-S4 产品需求

## 问题与目标

企业微信等自绘桌面应用可能拒绝trycua 0.25.0的后台键鼠路由。Yonder需要验证新版Driver能否以动作级前台投递建立真实焦点，并在动作后恢复用户原工作窗口；任何请求回执都不能替代后置Observe。

## 需求来源与分类

- 原始需求：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)“产品定义”“MVP主干链路”与[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)“CUA方案”，要求跨平台模型无关Driver、任务可见可控、每步后Observe。
- 后续用户变更（2026-09-30）：以正式Yonder链路验证企业微信长步骤任务；后台动作无法推进时继续修复，并同意验证单次前台投递路线。
- 架构约束：Accepted AD-E0-02固定0.25.0唯一Driver；Accepted AD-CU-05要求连续CUA会话不插入第二执行器；Accepted AD-AG-09要求计划片段、精确窗口target、不可核实副作用不重试。
- 待审设计建议：以0.30.4作为唯一升级候选，先在隔离Spike验证，再决定是否替换0.25.0。不是既定产品范围。

## 验收条件

- FGD-01：冻结候选`@trycua/cua-driver@0.30.4`及其完整性信息；不使用浮动版本。
- FGD-02：同一样本记录0.25.0与0.30.4的工具Schema差异，确认精确窗口target、窗口截图坐标、前台投递参数和动作效果字段。
- FGD-03：0.30.4对隔离窗口执行前台像素点击后，必须由独立Observe确认目标输入框取得焦点；仅请求被接受不算通过。
- FGD-04：随后精确窗口文本输入必须写入固定非敏感标记，并由新鲜Observe读取同一控件值确认；失败、超时或`unverifiable`不得重试。
- FGD-05：动作完成或失败后，原前台应用必须在有界时间内恢复；若平台只支持持久前置则淘汰“单次前台并恢复”路线。
- FGD-06：目标PID、窗口、截图代次或坐标失效时必须拒绝，不能改投当前前台、桌面坐标或模糊同名窗口。
- FGD-07：Worker关闭后拒绝新动作且无子进程残留；不引入第二CUA栈或常驻服务。
- FGD-08：升级候选必须兼容Yonder的`listToolsJson/callTool`SDK边界，或明确列出最小单版本迁移；不得长期兼容两套生产参数。
- FGD-09：Windows证据暂缓并明确标记未验证；macOS通过不得外推Windows。
- FGD-10：产品协议只允许`enter-target-query`与`draft-message-ref`的`type_text`携带有限`x/y`，以及既有点击语义携带`x/y`；Agent不得提交`delivery_mode`、正文或任意键盘宏。
- FGD-11：Worker必须为上述窗口坐标动作注入同一精确窗口target与`foreground`，从意图引用瞬时展开文本；Driver未返回`confirmed`、Observe失败或原前台无法恢复时不得推进。
- FGD-12：顶部浮窗持续展示聚焦、输入和验证步骤；只有confirmed且Observe有效才显示成功图标，普通用户输入不自动接管。
- FGD-13：2026-09-30 后续用户变更要求解决 Yonder 定位应用内元素持续失败、而截图型 Computer Use 可继续的问题；Yonder必须保持AX优先，只在AX空/多义、目标不可操作或动作未确认时补采同一可信窗口截图。
- FGD-14：视觉降级不得重放刚失败的动作，不得改投全桌面、当前前台或模糊同名窗口；已取得截图即使动作结论为unknown也必须返回归属慢脑。
- FGD-15：通用桌面搜索等非发送文本可由`computer.step`提交窗口局部坐标；Worker必须注入精确窗口与foreground，Agent不得提交PID、窗口、session或Driver target。
- FGD-16：已Observe但动作失败只到达安全边界，顶部浮窗显示未核实/失败而非成功；正常confirmed且元素充分的路径不采集截图。
- FGD-17：2026-09-30 后续用户变更要求慢脑一次提交一个有界计划片段；协议支持时，通用桌面任务不得退化为逐个 `computer.step`。顶部浮窗必须在执行前显示片段总步骤数与当前附近的有界步骤列表，并持续区分已完成、执行中、待执行与交回重规划。
- FGD-18：通用片段只增加“聚焦控件、输入文本、激活控件”三类封闭动作，映射到既有 `click/hotkey/type_text/press_key` Driver 工具；不得接受任意快捷键、任意按键、循环、分支或自由脚本。
- FGD-19：受限`cmd+f`搜索快捷键必须与窗口坐标动作一样由Worker注入动作级foreground和精确窗口target；不得要求慢脑另插`bring_to_front`，Driver在动作后恢复原工作窗口。

## 范围与非目标

Spike不发送消息、不使用用户正文、不改产品Gateway/协议/SQLite/UI，不直接把新SDK打入正式包。企业微信只保留既有无发送失败事实；验证使用隔离fixture和固定标记。

## 验收映射

| 来源 | 验收 |
|---|---|
| 产品简报与补充材料 | FGD-03、FGD-04、FGD-06、FGD-07 |
| 2026-09-30用户变更与正式样本 | FGD-02、FGD-03、FGD-04、FGD-05、FGD-13、FGD-14、FGD-15、FGD-16、FGD-17、FGD-18 |
| AD-E0-02、AD-CU-05、AD-AG-09 | FGD-01、FGD-06、FGD-07、FGD-08 |
| Windows暂缓决定 | FGD-09 |
| Accepted AD-CU-07产品Apply | FGD-10、FGD-11、FGD-12 |
