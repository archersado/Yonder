# 提案：圈选提问 macOS Preview

Story：CX-S2。来源：用户要求先在macOS测试、CX-S2 AC、Accepted AD-CX-01 的 Preview 决定。

变更：从桌宠/托盘显式启动当前显示器短生命圈选层，用户选择框选或笔画；笔画以外接矩形捕获至受控内存。选区旁确认卡完整显示且不保留蒙层；Esc、取消、超时和权限错误均清理。Preview 不保存、不上传、不调用 Agent，也不实现常驻指针模式。

Architecture Impact：conforming（仅实现 Accepted AD-CX-01 的 macOS Preview 边界）。
