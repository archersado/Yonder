# Proposal：CX-S2 Windows 受限圈选截图

关联Story：CX-S2；关联决策：Accepted AD-CX-01（Windows 100%缩放Preview子范围）。

## Why

Windows圈选捕获当前固定返回“仍在验证中”，确认卡没有图片；非权限类失败还会让PreviewSession滞留`capturing`，导致无图文字提交和重新圈选可能失败。

## What Changes

Desktop Windows Adapter使用Windows Graphics Capture捕获目标显示器稳定帧，以完整显示器物理原点裁剪并在内存编码PNG/Base64。只在96 DPI显示器启用；其他DPI、跨屏、超时或捕获失败统一转为无图确认，保留文字/语音、重选和取消。复用现有PreviewSession、临时附件和Agent输入路径。

Architecture Impact：conforming；不新增协议、持久化、Gateway能力、全局输入监听或默认权限请求。显示器拓扑只在圈选会话期间以有界轮询检测并复用统一清场；Windows应用切换、来源应用和高DPI扩围不在本Change内。
