# VI-S3 Agent 语音回复与打断

Story: VI-S3  
Epic: VI  
Status: design-review  
OpenSpec: -

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

2026-10-14 用户确认把语音回复加入 Yonder，并接受“ASR 负责输入、TTS 负责输出；本地优先、异步流式、有序播放”的方向。现有 VI-S1 明确不包含语音播报，AG-S9 已提供与当前输入关联的有界 Agent 流式回复，因此本 Story 独立承接输出侧能力，不改写 VI-S1 范围。

[Proposed AD-VI-03](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-VI-03-LOCAL-VOICE-REPLY.md) 定义跨平台候选、状态所有权与隐私边界。Windows/macOS 必须用同一中文样本验证首包延迟、顺序、打断、缺失语音包和离线行为；ADR 接受前不得生成产品实施 Proposal。

## OpenSpec 与验证

尚未创建。下一步应建立有期限的 `vi-s3-local-voice-reply-spike`，只验证系统原生 TTS 与 Application Port 契约，不接入第二套长期执行栈。
