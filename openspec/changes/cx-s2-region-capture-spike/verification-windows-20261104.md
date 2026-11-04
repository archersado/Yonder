# Windows Verification Goal：200% DPI 核心捕获样本

日期：2026-11-04
结论：**PASS（真实 192 DPI 单显示器 WGC 核心样本）；不等于完整 CX-S2 通过。**

## 通过范围

- Windows 11 x64、192 DPI（200%）主屏上，公开 WGC 探针对自绘无敏感四色窗口连续运行三次。
- 240×160 逻辑窗口每次均返回 480×320 物理像素；四色中心、缩放和 DisplayConfig 检查通过。
- 每次关闭 frame/session/pool 并释放 D3D 纹理；不保存截图、不安装输入监听，探针无残留进程。
- 结构化证据：`spikes/region-capture/evidence/windows-20261104/result.json`。

## 决策边界

本证据只支持把 Windows Preview 候选扩展到真实验证的 192 DPI。120/144/168 DPI、异轴 DPI、跨屏、应用切换、受保护内容、权限撤销、真实显示模式变化和物理断开仍须失败关闭或保留后续门禁。
