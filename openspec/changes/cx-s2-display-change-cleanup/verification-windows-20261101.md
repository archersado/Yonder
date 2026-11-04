# Windows Verification Goal：显示变化统一清场

日期：2026-11-01
结论：**PASS（Windows原生消息与产品清场接线子范围）；真实显示模式变化/拔屏仍未通过。**

## 通过范围

- Windows仅在圈选会话打开期间创建message-only HWND，接收`WM_DISPLAYCHANGE`；不接收可能由无关USB设备触发的通用`WM_DEVICECHANGE`。清场后投递`WM_CLOSE`、退出消息线程并注销窗口类。
- Windows单元测试通过原生消息窗口完成通知、单次回调、停止、资源释放和重复启动；desktop测试5/5通过。
- 产品原生验证通过托盘真实打开“圈选提问（预览）”，确认观察器存在后投递合成`WM_DISPLAYCHANGE`：预览由可见变为隐藏，标题记录`reason=display-change`且资源计数为零，观察器HWND消失。
- 最终通过真实托盘“退出 Yonda”正常退出，退出码0；本次不创建任务、不截图、不监听输入、不执行CUA。
- 同期修复共享清场原因白名单遗漏`display-change`的问题，并增加单元断言。

证据：

- `apps/desktop/evidence/cx-s2-display-change-windows-20261101/synthetic-display-notification-result.json`
- `apps/desktop/evidence/cx-s2-display-change-windows-20261101/verify-synthetic-display-notification.ps1`
- `apps/desktop/src/windows_display_watch.rs`

## 未通过范围与原因

合成`WM_DISPLAYCHANGE`只证明Windows Adapter和产品清场接线，不能证明操作系统在真实分辨率/DPI改变或物理显示器断开时的通知顺序。本机当前只有一个显示器，无法验证副屏、负坐标或拔屏。

## 解决方案

用户允许短暂改变主屏显示模式后，使用Windows公开显示配置API切换并立即恢复，验证同一结构化结果；连接副屏后再覆盖负坐标、不同DPI和物理断开。所有样本都必须先保存原配置、finally恢复，并不得保存桌面内容。
