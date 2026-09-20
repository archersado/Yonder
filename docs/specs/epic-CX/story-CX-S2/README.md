# CX-S2 圈选提问

Story: CX-S2  
Epic: CX  
Status: verifying
OpenSpec: cx-s2-agent-attachment-spike

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

用户可理解名称确定为“圈选提问”，不再使用“指针模式”。依赖VI-S1单轮语音、Windows/macOS屏幕区域捕获Spike及AG-S5 Agent用户输入通道。用户已要求先在macOS测试，Accepted AD-CX-01授权受限 Preview：仅当前显示器的显式选择、临时截图、取消清场和确认卡；不发送到Agent、不持久化、不支持多显示器或Windows。

## OpenSpec 与验证

已创建 [cx-s2-region-capture-spike](../../../../openspec/changes/cx-s2-region-capture-spike/proposal.md)。不得直接复制参考插件的全屏常驻覆盖层、全局输入监听或云端会话实现。Windows依用户决定暂缓，不视为通过。

[macOS Preview Change](../../../../openspec/changes/archive/2026-09-20-cx-s2-macos-preview/proposal.md)已归档。对 Agent 提交、语音组合、跨显示器和 Windows 保留后续门禁。

[macOS Preview 验证目标](../../../../openspec/changes/archive/2026-09-20-cx-s2-macos-preview/verification-macos.md)已由非实现者复核PASS并归档。该结论只覆盖macOS单显示器的选择、临时截图、确认卡与清场；完整CX-S2继续保持`verifying`，Agent提交、多显示器、Windows等范围仍需后续Change。

下一增量为 [Agent 会话临时附件 Spike](../../../../openspec/changes/cx-s2-agent-attachment-spike/proposal.md)：按 Proposed AD-CX-02 验证同一`AgentSession`内的有界分块、引用和失败清理；Spike通过前不修改产品协议或启用发送。

2026-09-18 macOS单显示器子范围PASS：公开ScreenCaptureKit区域API对自绘无敏感窗口的像素尺寸、2倍缩放与四色校验连续三次通过，截图不落盘且无进程残留。全新临时bundle身份的未授权预检连续两次返回`permission-required`，没有请求权限或截图。非激活选择层的合成Esc/超时清场与CGHID选区拖动均连续三次通过。详见 [macOS Verification Goal](../../../../openspec/changes/cx-s2-region-capture-spike/verification-macos.md)。副屏/负坐标、运行中撤权、显示器变化、物理键盘/鼠标和Windows仍未验证。
