# 验证结果

日期：2026-09-10

## 已通过

- Native Messaging 编解码：UTF-8、分片输入、连续帧和 1 MiB 超限拒绝。
- Host 标准输出仅包含长度前缀 JSON；自动检查通过。
- Windows 11 前台窗口探针通过：返回 `Code` 进程，标题存在、长度 58；证据未记录标题正文。
- Windows 实机已安装 Chrome 与 Edge，并存在 Windows Node 运行时。
- Windows 原生 Host 已成功编译，并通过标准输入输出 ping 往返：`{\"ok\":true}`。
- Chrome/Edge 的 HKCU Native Messaging Host 已注册，manifest 使用固定精确来源 `chrome-extension://mofdddjaniddgalgegfdjegiegpneokc/`。
- 最小扩展已生成：不申请 `history` 权限，Recording 默认关闭，只在显式点击后发送普通窗口的标签页事件。
- Edge 实连通过：用户手动加载扩展并开始 Record 后，Host 收到 8 条消息；日志仅记录 UTC 时间与 UTF-8 字节数，最近一条为 194 字节。
- Chrome 实连通过：隔离 Profile 中扩展无禁用原因；用户按 `Ctrl+Shift+Y` 后，Host 事件计数由 8 增至 9，通道结束后无遗留 Host 进程。
- Chrome Recording 边界通过：控制页产生 `recording.started → tab.updated/tab.activated → recording.stopped`，停止后无标签事件，显式断开后无遗留 v3 Host 进程。
- Chrome/Edge 均未启用隐私窗口权限；扩展不申请 `history` 权限，并在发送前拒绝 `tab.incognito`。
- Windows 操作 Hook 由用户显式按 Enter 后启动，10 秒内只统计键盘 43、鼠标 572；结束证据为 `recording=false`、`hooks_released=true`，进程已退出。

## 结论

Windows 范围通过，采用 WinEvent/UI Automation + Recording 期间低级 Hook + Chromium Native Messaging；产品统一使用 Rust。macOS 按当前范围延期。
