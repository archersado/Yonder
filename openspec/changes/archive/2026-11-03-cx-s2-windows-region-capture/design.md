# 设计

## 原生捕获

Windows模块在阻塞线程临时进入PMv2 DPI感知和WinRT MTA。以物理选区寻找唯一`HMONITOR`，读取`rcMonitor`与有效DPI；只接受96 DPI且完全位于单块显示器、像素数不超过16,777,216的矩形。

通过`IGraphicsCaptureItemInterop::CreateForMonitor`建立捕获项，创建带BGRA支持的D3D11设备和free-threaded frame pool。启动会话后最多等待2秒取得首个可用帧；用staging texture仅复制选区，Map后将BGRA转RGBA并以内存PNG编码。RAII清场守卫覆盖所有成功/失败路径，释放frame、session、pool和COM/DPI上下文；不写临时截图。

## 状态与反馈

`region_preview_hide_for_capture`仍唯一进入`capturing`。捕获命令的所有失败路径都调用`review_without_image`，使会话进入无附件`reviewing`。前端清除旧图片并展示“仅提问”确认卡；非100%缩放给出明确说明，其余错误给出稳定无图反馈。发送、语音、重选和取消继续复用既有命令。

每次会话带单调代次；异步捕获完成时必须与启动代次一致，旧结果不得修改新会话。Windows只在圈选层或确认卡存在期间每250毫秒比较显示器位置、尺寸和缩放签名，变化时复用统一清场；结束后代次使观察线程退出。

## 保留范围

本Change不实现Windows来源应用、应用切换监听、高DPI换算、跨显示器选区或GDI/BitBlt兜底。物理断开仍须独立原生证据；完整CX-S2保持`verifying`。
