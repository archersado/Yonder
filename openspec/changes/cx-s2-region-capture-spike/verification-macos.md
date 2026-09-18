# 独立 Verification Goal：macOS 无敏感区域捕获

日期：2026-09-18  
结论：PASS（macOS单显示器、已授权捕获子范围）。完整CX-S2与AD-CX-01未通过。

## 结果

- 探针只绘制一个240×160点的四色无敏感无边框窗口，没有全屏覆盖层、全局鼠标/键盘监听或产品接线。
- 公开`SCScreenshotManager.captureImage(in:)`以WindowServer返回的240×161点窗口框捕获480×322像素，横纵缩放均为2。
- 红、绿、蓝、黄四个已知色块中心校验通过，连续三次结果一致。颜色判定允许系统色彩管理导致的小幅通道偏移，仍要求目标通道占优。
- 像素只在进程内校验并立即释放，未写出截图；运行后无探针进程或编译产物残留。
- 另以LaunchServices后台启动全新临时bundle身份，只调用`CGPreflightScreenCaptureAccess`。连续两次均返回未授权，分类为`permission-required`；未请求权限、未尝试截图，临时App与进程均清理。
- 非激活临时选择层以合成AppKit Esc原生事件与120毫秒超时分别验证清场；两个窗口关闭后均不再onscreen，局部事件监听已移除，不安装全局监听，探针未成为前台应用。连续三次通过；本证据只覆盖应用内Esc分发，不声称为物理键盘样本。
- 局部选择层通过系统CGHID合成事件执行按下、拖动和松开，连续三次均得到精确160×100点选区，完成后关闭并恢复原鼠标位置。探针未成为前台应用、未安装全局监听且未截图。该样本验证系统输入分发，不声称为物理鼠标证据。

## 证据与范围

- `spikes/region-capture/evidence/macos/result.json`
- `spikes/region-capture/evidence/macos/permission-result.json`
- `spikes/region-capture/evidence/macos/lifecycle-result.json`
- `spikes/region-capture/evidence/macos/selection-result.json`
- `spikes/region-capture/macos-probe.swift`
- `spikes/region-capture/macos-permission-probe.swift`
- `spikes/region-capture/run-macos.sh`
- `spikes/region-capture/run-permission-macos.sh`
- `spikes/region-capture/macos-lifecycle-probe.swift`
- `spikes/region-capture/run-lifecycle-macos.sh`
- `spikes/region-capture/macos-selection-probe.swift`
- `spikes/region-capture/run-selection-macos.sh`

本机只有1块显示器，无负坐标样本。已授权与未授权预检均已覆盖；运行中撤权仍未验证。副屏/负坐标、显示器断开、物理Esc/鼠标与Windows同一样本仍是门禁。本结果不授权产品圈选Port或Gateway。
