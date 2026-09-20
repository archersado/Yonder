# VI-S1 单轮语音输入与转写

Story: VI-S1  
Epic: VI  
Status: implementing
OpenSpec: vi-s1-voice-input-spike

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

本Story来自2026-09-17用户提出的插件迁移需求，不属于原产品简报MVP主干。Proposed AD-VI-01与双平台Spike已建立；Windows/macOS音频采集、静音判定、ASR和权限路线尚未完成统一验证。

macOS无录音能力清单已通过：`zh-CN`识别器可用并支持本机识别，首次探针未请求权限。首次授权拒绝/撤权、设备切换与Windows样本仍未完成。

2026-09-18 macOS已授权会话子范围PASS：正式Yonda桌宠内显式开始、真实麦克风中文转写、无确认自动投递、完成后立即重新取得麦克风、手动停止、Agent断开错误及Esc取消均通过；PCM和转写正文未进入证据。见[独立验证](../../../../openspec/changes/vi-s1-voice-input-spike/verification-macos-explicit.md)。应用级权限重置后adhoc预览包仍沿用授权，采集中重置也未产生撤权事件；本机只有一个输入设备。首次权限拒绝、真实运行中撤权、设备切换与Windows样本仍待验证，AD-VI-01保持Proposed。

## OpenSpec 与验证

[技术Spike](../../../../openspec/changes/vi-s1-voice-input-spike/proposal.md)只比较候选并收集证据，不复制参考插件的Electron主进程、云端会话或密钥配置，不代替产品实施Proposal。
