# Design：Windows 200% DPI圈选截图

产品窗口的CSS矩形由`physical_region`按目标显示器物理work area与viewport比例执行一次floor/ceil换算。Windows捕获线程进入PMv2后，`MonitorFromRect`、`rcMonitor`、WGC item和D3D纹理均按物理像素工作，因此192 DPI不再额外乘2。

`supported_dpi`只接受同轴96或192；120/144/168和异轴DPI保持`capture-unsupported-scale`。选区仍须完全位于单个`rcMonitor`且不超过16,777,216像素。WGC首个可用帧通过纹理边界检查后裁剪、BGRA转RGBA并在内存编码PNG；超时、设备/帧变化或任何失败继续转无附件`reviewing`。

Spike先用独立C++探针证明真实192 DPI核心路线，再更新AD/Story和本Change；产品实现后用共享Rust模块复跑96 DPI负坐标副屏与192 DPI主屏。证据只保存DPI、坐标、尺寸、色块与资源状态，不保存像素。
