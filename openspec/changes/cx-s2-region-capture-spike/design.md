# 设计

macOS候选使用CoreGraphics枚举显示器与坐标/缩放，使用ScreenCaptureKit或系统公开区域截图接口读取用户显式选定区域。Windows候选使用Windows Graphics Capture、DisplayConfig与Per-Monitor DPI。两者输出同一结构化样本：显示器逻辑框、像素尺寸、缩放、请求矩形、返回像素尺寸、色块校验、权限和清场结果。

样本只捕获Spike自己绘制的检查窗口内部，运行中像素验证后立即释放。结构化证据不写入像素、窗口标题、应用内容或输入。如无屏幕录制权限，只输出`permission-required`，不弹窗后继续捕获。

Spike不实现真实全屏覆盖层、全局热键、鼠标监听、截图持久化、Agent提交或CUA暂停。这些只能在AD接受后进入产品Proposal。
