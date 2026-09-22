# TM-S2 资源准入与执行生命周期

Story: TM-S2
Epic: TM
Status: verifying
OpenSpec: tm-s2-execution-attempt
Increment: [tm-s2-attempt-result](../../../../openspec/changes/tm-s2-attempt-result/proposal.md)
Continuous Step Increment: [tm-s2-observed-boundary-advance](../../../../openspec/changes/tm-s2-observed-boundary-advance/proposal.md)
Failure Increment: [tm-s2-agent-fail](../../../../openspec/changes/tm-s2-agent-fail/proposal.md)

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

核心准入/start/finish 已存在旧 Change；真实执行器、文件身份与派发前检查尚待设计接线。

2026-09-16实施连续步骤边界：observed attempt可在无控制时原子普通停止，running任务随后声明并准备下一步骤；任务级Permit保持占用。

## OpenSpec 与验证

[执行尝试准备](../../../../openspec/changes/tm-s2-execution-attempt/proposal.md)与[结果事务](../../../../openspec/changes/tm-s2-attempt-result/proposal.md)均已生成并完成各自独立 Verification Goal；完整 Story 尚未 Archive。

2026-09-15依据Accepted AD-TM-08先实施[执行尝试准备子范围](../../../../openspec/changes/tm-s2-execution-attempt/proposal.md)：将已接受步骤与Application生成的完整执行身份、running迁移、事件和Outbox原子绑定；不派发真实动作或启用接管。

执行尝试准备子范围及schema7迁移验证PASS，见[独立Verification Goal](../../../../openspec/changes/tm-s2-execution-attempt/verification-goal.md)。真实Driver派发、Observe、停止控制与Windows仍待后续，完整Story不Archive。

2026-09-22跨Change复核：CU-S2已用真实prepared attempt经产品CU Port派发到受监管trycua Worker，并在macOS完成后置Observe；该证据关闭本Story的macOS真实派发/Observe子缺口。停止控制、完整生命周期、PR隔离审阅与Windows仍保留，完整Story不Archive。

2026-09-16：按 Accepted AD-TM-08 继续实施 attempt 结果事务与协议 1.5 增量；只记录动作/Observe 分类，不保存正文或截图。结果落库后仍不释放执行占用，步骤边界停止归 TM-S3。

尝试结果事务子范围PASS：observed/unknown与事件/Outbox原子提交，协议1.4兼容、1.5增量可见，真实macOS动作结果落库且占用保留；见[独立 Verification Goal](../../../../openspec/changes/tm-s2-attempt-result/verification-goal.md)。

2026-09-18：按 Accepted AD-TM-11 实施已观察失败终结。协议1.18、Gateway、SQLite既有attempt事实、CLI/MCP及桌宠failed出口已接通；`complete/fail` 对称拒绝相反动作结果，unknown仍不终结。macOS子范围PASS，见[独立 Verification Goal](../../../../openspec/changes/tm-s2-agent-fail/verification-goal.md)；Windows按用户要求暂缓，完整Story不Archive。
