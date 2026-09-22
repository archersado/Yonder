# EX-S1 macOS 独立验证记录

Story：EX-S1  
OpenSpec：ex-s1-jev-ultrafast-spike  
日期：2026-09-22  
结论：macOS 隔离子目标 **PASS**；Windows 未验证，`AD-EX-02` 保持 Proposed。

## 验证目标

验证 Jev 能否在固定 Observe 派生的有界候选 ID 中选择 CUA、ego-lite BUA、Document、Command 的下一步操作，并与完整状态逐步决策基线对照。探针不接产品 Gateway、SQLite、账号或真实用户任务，不产生真实副作用。

## 环境与命令

- 平台：macOS / `darwin`
- Node：`v24.21.0`
- SDK：`@typesafe-ai/sdk@0.6.0`，模型 `jev-latest`
- 样本：`ex-s1-v1`，8 条，重复 3 轮，共 24 次 Jev 决策
- 决策超时：1500ms；SDK 重试：0；请求间隔：250ms

```bash
npm --prefix spikes/jev-ultrafast run check
TYPESAFE_API_KEY=<key> npm --prefix spikes/jev-ultrafast run probe -- --repeat 3 --output spikes/jev-ultrafast/evidence/macos-r3.json
```

原始结构化指标见 [verification-macos-evidence.json](verification-macos-evidence.json)。证据不包含 API Key、完整状态或模型响应正文。

## 结果

| 指标 | 门槛 | 结果 | 判定 |
|---|---:|---:|---|
| 任务成功率 | ≥90% | 100%（24/24） | PASS |
| 误动作 | 0 | 0 | PASS |
| 慢脑 token 降低率 | ≥50% | 50.142% | PASS |
| Jev P95 / 基线 P95 | ≤1.2 | 1.012 | PASS |
| 单次决策最大时延 | ≤1500ms | 993.844ms | PASS |
| 成功路径交回率 | ≤20% | 0% | PASS |
| 取消至停止新动作 | ≤500ms | 1.101ms | PASS |

失败样本按设计必须交回，因此整体交回率为 50%；≤20% 门槛只适用于成功路径，结果为 0%。费用字段为 `null`：官方公开文档未提供可用单价，未虚构费用。Jev 请求与 token 已完整记录。

## 结论与限制

macOS 隔离样本显示官方 `systemOne` 选择题接口可以在有界候选上稳定选择，且压缩候选摘要达到冻结 token、质量、时延与取消门槛。该结论只覆盖本 Spike 的固定样本，不证明真实 CUA/BUA/Document/Command Observe 质量，不接产品 Gateway，也不改变 `AD-EX-02` 状态。Windows 证据与费用单价仍缺失；两者补齐前不得进入 EX-S2 实施。
