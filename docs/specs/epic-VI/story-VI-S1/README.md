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

2026-09-25用户确认“停顿后整段发送”时，macOS候选只有20秒总时限与手动停止，尚无说话后短暂停顿的自动结束判定；VI1-04/VI1-10当时未通过。双平台统一静音样本和AD-VI-01仍是完整Story门禁；Agent当前会话投递仍受AG-S5独立门禁约束。

2026-09-25 macOS候选已实现PCM RMS语音活动判定、有效语音后0.90秒静音收尾、手停/静音/总时限单终态门闩，以及跨FFI稳定会话ID和Rust单次final领取。确定性原生样本、重复/迟到回调和前端提示回归通过；隔离签名宿主完成构建与启动，但合成指针无法触发透明WebView入口，因此真实麦克风的声学自动结束与整段单条投递仍保留为未通过门禁。见[独立Verification Goal](../../../../openspec/changes/vi-s1-voice-input-spike/verification-macos-session-finalization.md)。

2026-09-18 macOS已授权会话子范围PASS：正式Yonda桌宠内显式开始、真实麦克风中文转写、无确认自动投递、完成后立即重新取得麦克风、手动停止、Agent断开错误及Esc取消均通过；PCM和转写正文未进入证据。见[独立验证](../../../../openspec/changes/vi-s1-voice-input-spike/verification-macos-explicit.md)。应用级权限重置后adhoc预览包仍沿用授权，采集中重置也未产生撤权事件；本机只有一个输入设备。首次权限拒绝、真实运行中撤权、设备切换与Windows样本仍待验证，AD-VI-01保持Proposed。

2026-09-23：macOS语音采集已补齐音频设备变化处理；`AVAudioEngineConfigurationChangeNotification` 触发时停止采集、释放资源并提示失败，不再把设备切换误报为继续聆听。该子范围已通过定向桌面包构建，但仍缺真实设备切换、运行中撤权和Windows证据，不改变完整Story状态。

2026-09-23：macOS语音错误提示已按Speech框架错误域和错误码分类，能区分权限撤销、服务中断、组件缺失、服务关闭与无语音；该项仍需真实运行中撤权证据，不能视为完整验收通过。

2026-09-23：OpenSpec Delta已补齐Requirement语句与Scenario块，`openspec validate vi-s1-voice-input-spike`当前通过；这不改变真实硬件与Windows证据缺口。

## OpenSpec 与验证

[技术Spike](../../../../openspec/changes/vi-s1-voice-input-spike/proposal.md)只比较候选并收集证据，不复制参考插件的Electron主进程、云端会话或密钥配置，不代替产品实施Proposal。
