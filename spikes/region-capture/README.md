# CX-S2 圈选捕获 Spike

`run-macos.sh`编译并运行短生命macOS探针：它绘制一个240×160点的四色无敏感窗口，使用公开`SCScreenshotManager.captureImage(in:)`捕获该区域，校验像素尺寸与四个色块后立即释放图像。

探针不保存截图，不监听鼠标/键盘，不接产品UI或Gateway。结构化输出只含尺寸、缩放、显示器数量和布尔校验。

`run-permission-macos.sh`创建一个全新临时应用身份，只调用`CGPreflightScreenCaptureAccess`。它不请求权限、不截图；预检未授权时只输出`permission-required`并清理临时App。

`run-lifecycle-macos.sh`验证非激活短生命选择层的Esc与超时清场。Esc使用合成AppKit原生事件，只验证应用内分发路径；不声称为物理键盘证据。探针不安装全局监听，关闭后检查窗口已移除且前台应用未变。

`run-selection-macos.sh`用系统CGHID合成事件对一个480×320点非激活局部选择层执行按下、拖动和松开，验证160×100点的AppKit选区与关闭路径，然后恢复原鼠标位置。它是系统输入路径的合成样本，不是物理鼠标证据。

`run-windows.ps1`在已批准的96/192 DPI显示器中央绘制四色无敏感窗口，使用`CreateForMonitor`、WGC free-threaded frame pool与D3D11 staging texture连续捕获三次。结构化证据只保存显示器边界、DPI、尺寸、颜色布尔结果和错误分类；不保存像素。未验证缩放记录为`capture-unsupported-scale`并跳过，不冒充支持。

2026-11-04独立C++探针已在真实192 DPI主屏连续三次完成240×160逻辑窗口到480×320物理像素、四色、释放和不落盘校验，见`evidence/windows-20261104/result.json`。该结果只解除192 DPI技术门禁，不代表其他高DPI或完整产品交互通过。

Windows候选先编译后运行：

```powershell
powershell -ExecutionPolicy Bypass -File .\build-windows.ps1
& "$env:TEMP\yonder-region-capture-spike\windows-probe.exe"
```

在仓库根目录重跑独立C++ 192 DPI证据与共享Rust 96/192 DPI证据：

```powershell
powershell -ExecutionPolicy Bypass -File .\spikes\region-capture\run-windows-core.ps1
powershell -ExecutionPolicy Bypass -File .\spikes\region-capture\run-windows.ps1
```

两份结构化结果均记录生成时间、精确重跑命令、输入源码SHA-256和Probe可执行文件SHA-256；显示器或DPI在窗口定位后发生变化时，共享Rust Probe失败关闭，不归因该次证据。

探针使用Windows Graphics Capture只捕获自身绘制的240×160逻辑像素四色窗口，并用DisplayConfig与Per-Monitor DPI校验当前显示器。像素只在内存中校验后释放，不保存截图、不安装输入监听、不接产品Port或Gateway。
