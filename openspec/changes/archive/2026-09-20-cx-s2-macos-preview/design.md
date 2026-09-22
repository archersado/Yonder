# 设计

Application 持有唯一的 `idle → selecting → capturing → reviewing → cancelled|failed` Preview 会话和有界截图字节；Desktop 组合根转发显式入口、选择、重新圈选和结束事件，macOS Adapter 复用已验证的 ScreenCaptureKit/CoreGraphics 区域截图与像素换算。不得由 WebView、Adapter 或任务库成为第二状态所有者。

选择层只覆盖当前显示器并提供框选与笔画两个显式模式；笔画只保留到松开鼠标，并以其外接矩形复用区域截图。确认卡完整显示在选区旁，不覆盖屏幕其余部分，仅展示临时缩略图和“发送功能待接入”。关闭、Esc、取消、30秒超时、重新圈选、捕获失败或进程退出时，Application 先丢弃截图字节与选区，再由 Desktop 隐藏窗口；重新打开不得看到上一会话预览。

打开前只读检查CUA桌面租约；已占用时返回`desktop-control-active`并保持`idle`，不触发覆盖层或CUA动作。屏幕权限缺失、受保护内容或异常均 fail closed，并返回不含截图正文的稳定分类。此 Change 不调用 AG-S5、TaskStore、Outbox、Recording 或 CUA Driver，不新增协议、持久化或依赖。
