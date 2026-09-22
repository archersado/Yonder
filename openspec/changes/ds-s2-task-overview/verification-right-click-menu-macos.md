# DS-S2 右键任务menu macOS验证

状态：PASS（2026-09-23）。

## 验证方式

- 构建正式 macOS 预览应用，并使用隔离 `HOME` 与真实 UDS/SQLite 任务库。
- 通过 MCP/UDS 创建真实任务 `task_ea73cc14eb1e23cb028bbddb5caac0f8`，名称为“右键菜单验证”。
- 运行 `swift apps/desktop/check-task-menu-right-click-macos.swift apps/desktop/evidence/ds-s2-right-click-menu-20260923`，使用原生 CGEvent 右键点击清醒小龙。

## 结果

- `right_click_opens_menu: true`
- `real_task_visible: true`
- `close_hides: true`
- `passed: true`

结构化日志：[result.json](../../../apps/desktop/evidence/ds-s2-right-click-menu-20260923/result.json)。
原生截图：[native-right-click.png](../../../apps/desktop/evidence/ds-s2-right-click-menu-20260923/native-right-click.png)。

该验证只覆盖 macOS 右键入口、真实任务菜单可见与关闭隐藏；Windows 仍按用户决定暂缓，不据此声明完整 DS-S2 Done。
