# 设计

Desktop macOS Adapter通过系统前台应用API返回有界显示名称。`PreviewSession`在`begin`时接收并校验该名称，`reselect`只清除截图和选区状态、保留来源；`clear`同时清除来源。Desktop用JSON安全投影到既有`yonda-region-open`事件，确认卡单独显示，不修改`region_preview_submit`。

平台API失败、空值、控制字符或超过128字符时保存为`None`，UI稳定显示“当前桌面”。验证只记录来源是否非空、是否跨重新圈选保持及结束后字节为零，不保存真实应用名称。
