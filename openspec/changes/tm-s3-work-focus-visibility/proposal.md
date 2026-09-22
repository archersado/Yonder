# TM-S3 接管定位可见性核验

关联 Story TM-S3、Accepted AD-TM-07/08 与既有 `tm-s3-takeover-work-focus`。当前定位仅核验 AX 前台与焦点；本增量在同一 macOS 原生 Adapter 中补充 WindowServer 可见性核验，避免跨 Space 时把不可见窗口误报为已定位。无协议、持久化、状态机或执行栈变更。

## 范围

- 前置后同时核验可信窗口仍属于原 PID/窗口 ID、保持原 frame，且出现在 WindowServer 的 `OnScreenOnly` 列表。
- 未通过一律返回既有 `VerificationFailed`，任务保持 paused；不切换到同名窗口、不移动窗口、不重试。
- 以固定隔离窗口样本验证当前 Space 可见性。不同 Space/多显示器需要真实设备人工样本；Windows 按用户要求暂缓。

## 非范围

不调用 AppleScript、私有 Space API 或创建辅助应用；不实现 Recording、交回或自动恢复。
