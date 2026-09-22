# DS-S2 右键菜单独立验证

日期：2026-09-23。Story：DS-S2；Change：`ds-s2-task-overview`。状态：macOS右键入口子范围PASS；Windows按用户决定暂缓，完整Story不Done/Archive。

## 实现范围

按用户变更，任务菜单由清醒小龙悬停自动打开改为右键显式唤起。悬停仍只承载状态表现，不再触发菜单；轻点仅动作反馈，拖动不打开，休眠右键先唤醒。右键、Enter/Space和托盘共用既有`task_menu_show`查询真实未结束任务并定位面板；无任务不显示。菜单打开后聚焦任务面板，失焦、关闭和Escape收起。悬停轮询、200ms计时和跨间隙自动收起逻辑已删除。

## 原生证据

证据目录：[`apps/desktop/evidence/task-menu-right-click-20260923/`](../../../apps/desktop/evidence/task-menu-right-click-20260923/)。

- `agent-result.json`：父进程通过`--local-agent-stdio`完成Gateway握手，真实创建`created`任务并成功取消，任务与历史保留。
- `native-result.json`：真实进程上`hover_does_not_open`、`inactive_before_click`、`right_click_opens_menu`、`menu_focused`、`blur_hides`均为true；`passed=true`。
- `native-right-click-menu.png`：右键打开后的原生任务面板截图。

原生工具：[`apps/desktop/check-task-menu-right-click-macos.swift`](../../../apps/desktop/check-task-menu-right-click-macos.swift)。工具按指定PID操作，避免误碰其他Yonda进程；鼠标移动与右键使用真实CGEvent，菜单/焦点/窗口可见性使用AX与CGWindow事实。切换Finder触发失焦隐藏，不使用内存对象伪造任务或窗口。

## 回归与门禁

- `cargo test --offline --locked -p yonder-desktop`：6项通过、0失败。
- `node --check apps/desktop/ui/pet.js apps/desktop/ui/task-space.js apps/desktop/check-task-space.mjs`：语法检查通过。
- `swiftc -parse apps/desktop/check-task-menu-right-click-macos.swift`：通过。
- `openspec validate ds-s2-task-overview`：通过。
- ego-browser：`browser-result.json`返回`hoverDoesNotOpen`、`rightClickOpens`、`clickDoesNotOpen`、`keyboardAndDrag`、`unknownOnFailure`等全部true。夹具只注入UI测试值，不写产品任务库。

## 范围限制

本验证只覆盖当前macOS一台真实机器的右键入口、真实任务存在性、面板焦点与失焦收起；不扩大为Windows、所有屏幕位置、完整DS-S3环绕图标或真实执行链路。历史悬停验证保留为上一入口版本的证据，不再代表当前鼠标入口。
