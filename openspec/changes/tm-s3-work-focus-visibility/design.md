# 设计

`work_focus_macos.c` 在既有 AX 焦点核验成功后，以原始 `pid + window_id` 查询 `kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements`。只有 layer 0 的同一窗口存在才返回 focused；缺失映射为既有稳定失败 `VerificationFailed`。

这项核验使用 WorkRef 已绑定的身份，不新增 UI/Agent 输入，也不落库。`focused/failed` 继续由已存在的 `focus_takeover` 事务写入任务序号、事件及 Outbox。

可运行验证复用 `apps/desktop/check-work-focus-macos.py`：正常定位后的隔离窗口必须既为 key window 又处于 active Space。真实跨 Space/多显示器样本作为后续独立证据，不能用单屏夹具替代。
