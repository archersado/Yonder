# 独立 Verification Goal：Windows 无敏感区域捕获

日期：2026-11-03
结论：PASS（仅100%缩放、负坐标副屏子范围）。完整CX-S2与Windows高DPI未通过。

## 结果

- Windows Graphics Capture以`CreateForMonitor`捕获显示器并读取首个可用帧；D3D11 staging texture按完整显示器物理原点裁剪。
- 1920×1080、96 DPI、原点`(487,-1080)`的副屏对240×160无敏感四色窗口连续三次返回同尺寸，四色中心全部通过。
- 2880×1800主屏真实为192 DPI；显式PMv2感知后由共享捕获模块直接返回`capture-unsupported-scale`，未返回像素、未宣称通过。
- 证据不含像素；窗口、帧、会话和探针进程均释放，未保存截图。
- 结构化证据记录UTC时间、源码revision/工作区状态、Windows构建号、GPU/驱动、执行命令和显示器边界，便于复现。

## 证据

- `spikes/region-capture/evidence/windows/result.json`
- `spikes/region-capture/run-windows.ps1`
- `spikes/region-capture/windows-probe/`

本结论只授权100%缩放Windows Preview。125%/150%/200%、主屏截图、应用切换、显示器变化、受保护内容、设备丢失和物理断开仍需后续证据。
