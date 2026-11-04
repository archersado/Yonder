# CX-S2 圈选提问

Story: CX-S2  
Epic: CX  
Status: verifying
OpenSpec: cx-s2-windows-region-capture

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

用户可理解名称确定为“圈选提问”，不再使用“指针模式”。macOS既有子范围不变。Windows 96 DPI负坐标副屏及192 DPI主屏WGC统一样本均连续三次通过；按2026-11-04用户要求，Preview扩围至已验证的96/192 DPI，其他DPI安全降级为无图确认。应用切换、来源应用、物理断开及其他高DPI仍是后续门禁。

## OpenSpec 与验证

已创建 [cx-s2-region-capture-spike](../../../../openspec/changes/cx-s2-region-capture-spike/proposal.md)。不得直接复制参考插件的全屏常驻覆盖层、全局输入监听或云端会话实现。[Windows 100% DPI受限产品接线](../../../../openspec/changes/archive/2026-11-03-cx-s2-windows-region-capture/proposal.md)已由非实现者复核PASS并归档；完整CX-S2仍保持`verifying`。

[Windows 200% DPI扩围Change](../../../../openspec/changes/archive/2026-11-04-cx-s2-windows-high-dpi/proposal.md)已完成192 DPI真实样本、产品共享Rust模块和确认卡验证，并由非实现者复核PASS；不扩大到其他缩放或完整Story。

[macOS Preview Change](../../../../openspec/changes/archive/2026-09-20-cx-s2-macos-preview/proposal.md)已归档。跨显示器和 Windows 保留后续门禁。

[macOS Preview 验证目标](../../../../openspec/changes/archive/2026-09-20-cx-s2-macos-preview/verification-macos.md)已由非实现者复核PASS并归档。该结论只覆盖macOS单显示器的选择、临时截图、确认卡与清场；完整CX-S2继续保持`verifying`，Agent提交、多显示器、Windows等范围仍需后续Change。

[Agent 会话临时附件 Spike](../../../../openspec/changes/archive/2026-09-20-cx-s2-agent-attachment-spike/proposal.md)与独立复核已PASS并归档，AD-CX-02已Accepted。[圈选提交产品Change](../../../../openspec/changes/archive/2026-09-20-cx-s2-agent-attachment-submit/proposal.md)已完成能力协商、Rust协议、同会话附件传输与确认卡发送；macOS原生四结果与独立Verification Goal均PASS并归档。Windows、多显示器和云端产品WSS仍是后续门禁，因此完整CX-S2保持`verifying`。

[无截图文字提交Change](../../../../openspec/changes/archive/2026-09-20-cx-s2-text-only-submit/proposal.md)已完成直接点击与屏幕录制权限缺失后的“仅提问”路径；macOS原生accepted/rejected/unknown、临时未授权bundle与独立Verification Goal均PASS并归档。

[圈选前暂停Change](../../../../openspec/changes/archive/2026-09-21-cx-s2-pause-before-select/proposal.md)已完成：CUA持有前台桌面租约时先提交可信`pause`并确认步骤边界停止，再显示圈选层；小龙、托盘、unknown拒绝与无租约回归的macOS证据及独立Verification Goal均PASS并归档，不触发定位或Recording。

当前归档路径：openspec/changes/archive/2026-09-21-cx-s2-voice-submit/

[圈选语音提交Change](../../../../openspec/changes/archive/2026-09-21-cx-s2-voice-submit/proposal.md)已完成：在既有确认卡内显式开始macOS语音，部分转写只显示，最终非空转写复用当前selection提交与附件清理并自动发送；带图、无图、取消和普通direct语音均经独立原生复核PASS，且没有双投递。关闭、取消、重新圈选和超时都会停止本轮收音。

[确认卡来源应用Change](../../../../openspec/changes/archive/2026-09-21-cx-s2-source-application/proposal.md)已归档：在用户显式启动时读取一次前台应用显示名称，只在本轮PreviewSession和确认卡内显示；重新圈选保留原来源，结束即清零，不进入Agent正文、日志或数据库。macOS真实前台应用路径与数据边界经独立Verification Goal复核PASS。

[应用切换清场Change](../../../../openspec/changes/archive/2026-09-22-cx-s2-app-switch-cleanup/proposal.md)已归档：macOS可见圈选层或确认卡切换应用时统一清场；截图流程主动隐藏窗口时不会误清场，当前`dev`正式预览bundle连续两次独立复核PASS。Windows保留后续门禁。

2026-09-22：[显示器参数变化清场Change](../../../../openspec/changes/cx-s2-display-change-cleanup/proposal.md)已实现并在macOS主显示器模式变化样本中PASS；原生关闭路径先完成清场时与屏幕通知共享同一不变量。物理显示器断开、副屏/负坐标、运行中撤权与Windows仍保留门禁。

2026-09-18 macOS单显示器子范围PASS：公开ScreenCaptureKit区域API对自绘无敏感窗口的像素尺寸、2倍缩放与四色校验连续三次通过，截图不落盘且无进程残留。全新临时bundle身份的未授权预检连续两次返回`permission-required`，没有请求权限或截图。非激活选择层的合成Esc/超时清场与CGHID选区拖动均连续三次通过。详见 [macOS Verification Goal](../../../../openspec/changes/cx-s2-region-capture-spike/verification-macos.md)。副屏/负坐标、运行中撤权、物理显示器断开、物理键盘/鼠标和Windows仍未验证。

2026-11-01 Windows单显示器核心捕获子范围PASS：公开Windows Graphics Capture对自绘四色窗口连续三次完成240×160逻辑像素到360×240物理像素（150% Per-Monitor DPI）的尺寸与四色校验，DisplayConfig可用，像素立即释放且不落盘、不监听输入。详见 [Windows Verification Goal](../../../../openspec/changes/cx-s2-region-capture-spike/verification-windows-20261101.md)。本机无副屏/负坐标，物理选择/Esc、权限拒绝/撤销和受保护窗口仍未通过，因此AD-CX-01继续只接受macOS Preview，Windows产品边界不扩展。

2026-11-01 Windows显示变化清场接线子范围PASS：短生命message-only HWND在圈选预览期间接收`WM_DISPLAYCHANGE`，并排除可能被无关USB变化触发的通用`WM_DEVICECHANGE`；合成原生通知使真实预览窗口隐藏、PreviewSession/语音/WebView共享清场且观察器销毁，应用随后从托盘正常退出；并修复共享结构化原因漏记`display-change`的缺陷。合成通知不替代真实改分辨率、拔屏、副屏/负坐标证据。
