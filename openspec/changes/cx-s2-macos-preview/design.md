# 设计

macOS Adapter 复用已验证的 ScreenCaptureKit/CoreGraphics 选择与像素换算。Desktop 宿主只允许当前显示器存在一个 `idle → selecting → reviewing → cancelled|failed` 会话；确认卡仅展示临时缩略图和“发送功能待接入”。选区数据仅在进程内，关闭、取消、超时或进程退出即释放。

任何 CUA 桌面租约、屏幕权限缺失、受保护内容或异常都 fail closed。此 Change 不调用 AG-S5、TaskStore、Outbox、Recording 或 CUA Driver。
