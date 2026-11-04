# 独立 Verification Goal：DS-S1 Windows 原生托盘与退出生命周期

日期：2026-10-29（本机时区跨至 2026-10-30）；平台：Windows 11 专业版 `10.0.26200`。结论：**PASS（Windows 托盘找回与正常退出子范围）**。

## 修复

正式桌面在加入 DS-S2“任务总览”等入口时，覆盖了 DS-S1 既有“显示小龙 / 退出 Yonda”基础菜单，退出文本还错误变成“退出任务面板”。本轮在保留任务总览、圈选、资源包、Jev 和 Agent 管理入口的同时恢复 `show-pet` 与正确退出文本。`show-pet` 只显示、取消最小化、聚焦既有 pet 窗口并派发既有 `yonda-show`；`quit` 继续调用 `app.exit(0)`，未新增窗口、协议、状态源或持久化。

## 原生证据

- 最新调试可执行文件 SHA-256：`1b2caf0bcf0714b64a20135788f3fbc35c1e85193532c1c7422281fd79dd1d4b`；桌面 crate 测试 8+4 项通过，Windows 构建通过。
- Windows 隐藏托盘区真实图标由 UI Automation 识别为 `Yonda / SystemTray.NormalButton`。原生菜单截图 `native-tray-menu.png` 显示七项，首项“显示小龙”、末项“退出 Yonda”，中间既有增量入口均保留。
- 验证先通过 Win32 把当前桌宠窗口置为最小化；从真实菜单屏幕点反查到 `ControlType.MenuItem / 显示小龙` 并调用系统 `InvokePattern`。原主窗口句柄 `1581846` 更新为当前句柄 `11017486`，当前窗口可见且非最小化，证明找回的是同一宿主中的 pet 窗口而非第二个应用实例。
- 退出前递归记录宿主及 WebView2 共 12 个进程，整棵归属树 TCP 连接数为 0。随后实际语义激活 `ControlType.MenuItem / 退出 Yonda`，宿主退出码为 0；约 108.312ms 的轮询观测上界后，记录的全部 PID 均已消失，`remaining_owned_process_ids=[]`，且 `forced_cleanup=false`。
- 机器可读结果与可复现夹具：`apps/desktop/evidence/ds-s1-windows-20261029/native-tray-exit-result.json`、`verify-native-tray-exit.ps1`。本轮未创建任务、未启动 ego-lite、未执行 CUA。

## 边界

108.312ms 是一次退出等待观测值，不是新增性能 SLA。进程断言只覆盖退出前由本次宿主递归归属的成员，不声称覆盖未来独立 Worker。菜单组成发生变化后仍需 macOS 当前版本回归；Windows 人工拖动、核心状态延迟、九态完整真实链路、macOS 剩余项和 DS-S1 总 Goal 均未完成，因此 Change 不 Archive。
