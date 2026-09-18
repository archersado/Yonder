# DS-S2 小龙轻量任务菜单独立 Verification Goal

日期：2026-09-14；Story：DS-S2；Change：ds-s2-task-overview。AD：AD-DS-01、AD-ST-01。状态：本机首批菜单入口、真实空库与核心回归PASS，完整任务/控制/双平台验收未完成，不Archive。

## 最终交互

按用户后续变更，取消独立面板作为默认入口。正式apps/desktop同时拥有200像素小龙与默认隐藏的320×400透明无装饰任务菜单。轻点/Enter/Space保留反馈并打开任务菜单；拖动不打开、开始时隐藏菜单；休眠双眼先唤醒。菜单定位归Rust，同屏工作区夹入，空间不足翻到上方；关闭/Escape/失焦隐藏。完整环绕多功能图标仍未实施。

正式宿主用系统app_data_dir下host.lock及tasks.db，任务查询仅接受task-space本地窗口；MVP未加密SQLite，主状态与事件/Outbox不变。无外部任务执行入口，无演示任务。

## 浏览器验证

ego-browser TaskSpace 3，使用同一p1。apps/desktop/check-task-space.mjs包含显式测试夹具，未进入产品任务库：21任务跨页可达、详情缺失字段反馈、读取失败保留过期页、全部筛选重置第一页均通过；小龙轻点仅一次打开，模拟减少动态效果下Enter仍打开、拖动不误开且调用原生拖动/隐藏均通过。

最初file页面使用module脚本停在读取中，现场显示无资源条目，改为无模块依赖的defer脚本并以闭包隔离后正常执行。测试夹具用于UI回归，不声称21个任务已在真实桌面执行。该TaskSpace已finish，未保留测试页。

## 原生验证与失败历史

独立面板初版原生空库/关闭/托盘验证通过，证据apps/desktop/evidence/task-space-20260914；仅保留历史，不作为最终形态。

新菜单早期task-menu-20260914因启动/状态下未找到清醒按钮未通过；task-menu-retry-20260914取得真实空任务画面，但关闭隐藏检查未通过。按钮/Escape改用统一原生task_menu_close，只允许本地菜单窗口，不再由UI传目标label。下一次启动早期task-menu-close-20260914辅助功能入口未就绪；验证工具补有界就绪等待，不据早期失败断言产品入口损坏。

最终工具`swift apps/desktop/check-task-space-macos.swift apps/desktop/evidence/task-menu-settled-20260914`退出0：PID 88013，window_id 35357，pet_activation_opens_menu、real_empty_state、close_hides、tray_restores_same_window均true。native-empty.png人工查看为完整圆角透明任务菜单，result.json保留结构化证据。该工具使用AXPress激活小龙与关闭按钮，不能替代物理鼠标/所有屏幕位置验证；真实任务库目前为空，不将空库样本扩大为真实并行执行通过。

## 单一桌宠与回归

研发迁移时旧E0预览PID 61042和新宿主曾同时可见，用户指出两个小龙后已通过旧托盘正常退出E0。最终pgrep仅PID 88013正式宿主，无旧spike。新宿主更新时先正常退出旧正式实例再启动，未留下第二小龙。旧源码保留Spike历史，后续不向其追加产品功能。

离线锁定构建通过；desktop库3、窗口几何/忙碌门禁2、Application6、Adapter10，共21项回归通过。架构关联与git diff --check通过。复用已有Tauri 2.11.5、tauri-build 2.6.3及objc2原生窗口能力，主Workspace首次登记对应Tauri传递依赖，无浏览器替代栈。

## 未通过范围

Windows继续按用户要求暂缓。真实两任务并行、步骤/等待原因与外部浏览器引用、逐任务控制、完整收起预约及任务开始唤醒接线尚未完成。正式停靠前在TaskHost锁内重新检查数据库/准入占用，busy/unknown拒绝，但宿主尚无执行器；在开放派发前必须完成RestPermit协调，不将只读activity当长期隐藏许可。

菜单大小/布局后续可按实际体验调整；未取得物理点击外部、全部边缘/多屏、原生键盘Escape等完整闭环。没有本轮PR/完整Story Done，不Archive。

## 用户反馈后的鼠标路径复验

用户指出普通点击没有菜单，返回实施阶段。新增check-task-menu-click-macos.swift，先切换Finder使Yonda未激活，再按辅助功能提供的位置发送原生鼠标按下/抬起，不以AXPress代替清醒小龙点击。修复前运行task-menu-mouse-before-20260914，inactive_before_click=true，退出4，未取得菜单，复现失败。已安装Tauri默认accept_first_mouse=false；按既有轻点要求启用原生首次点击传递，不新增交互入口。后续结果另记。

## 最终入口覆盖

用户随后要求悬停显示、取消点击触发；旧AX轻点打开记录仅为历史。最终验证见verification-hover-macos.md，本机悬停子范围PASS，完整Story继续实施，不Archive。
