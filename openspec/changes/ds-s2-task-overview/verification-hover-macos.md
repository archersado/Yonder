# DS-S2 悬停菜单独立 Verification Goal

日期：2026-09-14。Story：DS-S2；Change：ds-s2-task-overview。状态：本机悬停入口子范围PASS；完整Story未Done、不Archive，Windows按用户要求暂缓。

## 验收与实现

用户明确将点击菜单改为悬停菜单。清醒小龙悬停后显示，轻点仅反馈，双眼轻点只唤醒；按下取消待打开，拖动不打开。Enter/Space仍可进入菜单。悬停不请求原生焦点，菜单可跨间隙操作，关闭/失焦隐藏，托盘可恢复。复用既有NSWindow窗口内逻辑坐标读取，不记录轨迹，无新增依赖；串行查询250毫秒加200毫秒停留计时，实际未激活WebView调度可能更慢，本阶段不以严格性能阈值阻断功能。

## 原生证据

工具：apps/desktop/check-task-menu-hover-macos.swift。清醒入口使用CGEvent鼠标移动，未以AXPress代替悬停；关闭和托盘按钮用AXPress。切换当前编辑器使Yonda未激活，按CGWindow实际可见边界计算位置，离开后等待1.5秒确保原生查询观察，再重新进入。读取真实空任务库，无演示任务。

最终命令：swift apps/desktop/check-task-menu-hover-macos.swift apps/desktop/evidence/task-menu-hover-observed-20260914。退出0，PID94356、窗口36479，pet_hover_opens_menu、hover_preserves_inactive、real_empty_state、close_hides、tray_restores_same_window均true。result.json和native-empty.png为证据。焦点保持在实际菜单可见后重新读取；人工查看截图为圆角任务菜单与暂无任务。

早期工具使用辅助功能旧位置造成鼠标目标偏差；Tauri全局鼠标位置实现还混用Retina像素/逻辑点，改为NSWindow窗口内同坐标系。原生宿主只用原生悬停观察，浏览器用DOM悬停，避免两套进入/离开通知相互取消计时。此前短暂离开未被未激活WebView观察、固定0.8秒恢复检查过早，不能据此判定最终版本入口失败；失败记录不改写为通过。最终工具采用有界状态等待，未为此增加产品延迟。

## 前端回归

ego-browser TaskSpace5，check-task-space.mjs返回hoverOpens、clickDoesNotOpenAgain、reducedKeyboardOpens、dragDoesNotOpen全部true；注入仅测试用native布尔值，不写产品任务库。该空间已finish，不保留夹具页。离线锁定构建通过。

## 范围限制

只证明当前macOS的悬停、空库、关闭与托盘恢复；不扩大为真实多任务执行、全部屏幕位置、Windows、所有物理键盘/拖动通过。原点击AX入口验证仅为历史，用户变更后的菜单入口以本Goal为准。

后续用户改为仅有未结束任务才悬停、同桌面与移出隐藏；此处无条件空库悬停记录仅为历史。最新实现与分范围验证见verification-task-state-hover-macos.md。
