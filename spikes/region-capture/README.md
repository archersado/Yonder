# CX-S2 圈选捕获 Spike

`run-macos.sh`编译并运行短生命macOS探针：它绘制一个240×160点的四色无敏感窗口，使用公开`SCScreenshotManager.captureImage(in:)`捕获该区域，校验像素尺寸与四个色块后立即释放图像。

探针不保存截图，不监听鼠标/键盘，不接产品UI或Gateway。结构化输出只含尺寸、缩放、显示器数量和布尔校验。

`run-permission-macos.sh`创建一个全新临时应用身份，只调用`CGPreflightScreenCaptureAccess`。它不请求权限、不截图；预检未授权时只输出`permission-required`并清理临时App。

`run-lifecycle-macos.sh`验证非激活短生命选择层的Esc与超时清场。Esc使用合成AppKit原生事件，只验证应用内分发路径；不声称为物理键盘证据。探针不安装全局监听，关闭后检查窗口已移除且前台应用未变。

`run-selection-macos.sh`用系统CGHID合成事件对一个480×320点非激活局部选择层执行按下、拖动和松开，验证160×100点的AppKit选区与关闭路径，然后恢复原鼠标位置。它是系统输入路径的合成样本，不是物理鼠标证据。
