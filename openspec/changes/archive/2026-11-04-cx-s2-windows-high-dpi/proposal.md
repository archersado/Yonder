# Proposal：CX-S2 Windows 200% DPI圈选截图

关联Story：CX-S2；关联决策：Accepted AD-CX-01（Windows 96/192 DPI Preview）。

## 问题

当前Windows产品仅允许96 DPI。用户在真实192 DPI（200%）主屏圈选后，捕获模块返回`capture-unsupported-scale`，确认卡按安全降级隐藏图片。独立WGC统一样本已经证明该主屏的逻辑/物理尺寸、四色和资源释放连续三次正确。

## 变更

Desktop Windows Adapter把受支持范围扩至同轴96/96与192/192 DPI，继续复用现有CSS viewport到物理work area换算、`CreateForMonitor`、`rcMonitor`裁剪、内存PNG和PreviewSession。其他DPI、异轴DPI、跨屏、超限或捕获失败仍进入无图确认。

Architecture Impact：conforming；不新增协议、持久化、默认权限请求、全局监听或状态所有者，不放宽像素与附件上限。

## 非目标

不支持125%/150%、跨屏、Windows来源应用/应用切换、受保护内容专用样本、撤权、真实模式变化或物理拔屏；不把完整CX-S2标记Done。
