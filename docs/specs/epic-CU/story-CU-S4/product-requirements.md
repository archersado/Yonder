# CU-S4 产品需求

## 问题与目标

企业微信等自绘桌面应用可能拒绝trycua 0.25.0的后台键鼠路由。Yonder需要验证新版Driver能否以动作级前台投递建立真实焦点，并在动作后恢复用户原工作窗口；任何请求回执都不能替代后置Observe。

## 需求来源与分类

- 原始需求：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)“产品定义”“MVP主干链路”与[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)“CUA方案”，要求跨平台模型无关Driver、任务可见可控、每步后Observe。
- 后续用户变更（2026-09-30）：以正式Yonder链路验证企业微信长步骤任务；后台动作无法推进时继续修复，并同意验证单次前台投递路线。
- 架构约束：Accepted AD-E0-02/AD-CU-07修订固定0.30.4为唯一Driver；Accepted AD-CU-05要求连续CUA会话不插入第二执行器；Accepted AD-AG-09要求计划片段、精确窗口target、不可核实副作用不重试。
- 已接受设计：0.30.4先经隔离Spike验证，正式QQ音乐快捷键样本确认其动作级foreground为0.25.0所缺能力后，替换旧固定版本且不保留双栈。

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
- FGD-20：2026-09-30后续用户变更要求macOS产品改用Codex Computer Use / Sky，不保留trycua生产依赖、环境选择或运行时回退。
- FGD-21：同一任务启动应用后必须绑定唯一应用身份；每步从新鲜应用级AX transcript解析元素并优先使用`element_index`，不得复用旧index或改投当前前台。
- FGD-22：AX transcript只用于本次运行时定位，不进入SQLite、事件、Outbox、日志或顶部浮窗；慢脑继续只接收有界Observation。
- FGD-23：通用聚焦、输入和激活动作必须映射为Sky支持的动作并在动作后重新Observe；无法形成后置事实时交回且不自动重试。
- FGD-24：调用方给出合法bundle id而官方应用目录尚未收录已安装应用时，Worker只可在固定系统应用根读取Info.plist并唯一解析完整路径；不得扫描全盘、接受环境目录或以显示名猜测，零命中或多命中必须安全交回。
- FGD-25：协议1.41只允许通用`activate-control + click`视觉坐标携带`click_count=1|2`，并只允许`activate-control + press_key`使用`ARROWDOWN`导航自绘候选；其他语义、元素点击、发送动作、大于2的次数及其他导航键必须拒绝。动作仍须以后置截图或元素事实证明目标被激活。
- FGD-26：任务终结必须按任务实际运行时归属路由。只有内存执行Runtime持有该任务时才从内存快照终结；计划片段兼容链未登记到该Runtime时继续使用其已Observe、已推进且仍持有桌面租约的既有安全终结路径，不得因全局Runtime存在而误报“任务不存在”。
- FGD-27：2026-10-06用户变更要求窗口绑定状态外置。成功启动产生的可信应用绑定必须由Rust Adapter任务运行态跨步骤保持；Node Worker或签名MCP Client重建后，后续步骤仍按同一bundle id重新解析唯一窗口，不得退回当前前台。绑定不得跨任务、持久化或由Agent/UI覆盖。
- FGD-28：Sky对无业务副作用的坐标控件激活若在动作投递后不返回，Yonder必须在宿主总超时前停止旧Client并重新Observe同一应用；不得重放点击，且只有可验证的界面变化才能继续。消息发送、删除等副作用动作不得使用该收敛路径。

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
| 2026-09-30 Sky单栈决定与AD-CU-09 | FGD-20、FGD-21、FGD-22、FGD-23、FGD-24、FGD-25 |
| 2026-10-06窗口绑定外置用户变更与AD-CU-09修订 | FGD-27 |
| 2026-10-06企业微信非AX企业入口回归 | FGD-28 |
