# VI-S3 Agent 语音回复与打断

Story: VI-S3  
Epic: VI  
Status: implementing
OpenSpec: vi-s3-local-voice-reply-spike

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

2026-10-14 用户确认把语音回复加入 Yonder，并接受“ASR 负责输入、TTS 负责输出；本地优先、异步流式、有序播放”的方向。现有 VI-S1 明确不包含语音播报，AG-S9 已提供与当前输入关联的有界 Agent 流式回复，因此本 Story 独立承接输出侧能力，不改写 VI-S1 范围。

[Proposed AD-VI-03](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-VI-03-LOCAL-VOICE-REPLY.md) 定义跨平台候选、状态所有权与隐私边界。Windows/macOS 必须用同一中文样本验证首包延迟、顺序、打断、缺失语音包和离线行为；ADR 接受前不得生成产品实施 Proposal。

## OpenSpec 与验证

已建立有期限的 [vi-s3-local-voice-reply-spike](../../../../openspec/changes/vi-s3-local-voice-reply-spike/proposal.md)，只验证系统原生 TTS 与 Application Port 契约，不接入第二套长期执行栈。该 Spike 不等于产品实施 Proposal；双平台结论与 AD-VI-03 接受前，产品接线仍未 ready。

2026-10-14 macOS 隔离夹具已实现并完成实施侧三轮自检：本机发现 18 个中文声音，三句原生队列严格按序，控制面停止为 0～119ms，资源释放通过。系统在停止后以 `didFinish` 而非 `didCancel` 返回唯一终态，因此正式 Application 必须以 generation 失效屏蔽原生终态歧义。该证据未覆盖有声输出、真实断网、输出设备切换、Windows 或非实施者复核，不接受 AD-VI-03，也不授权产品接线。
