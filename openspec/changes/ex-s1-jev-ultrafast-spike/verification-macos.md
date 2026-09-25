# EX-S1 macOS 独立验证记录

Story：EX-S1  
OpenSpec：ex-s1-jev-ultrafast-spike  
日期：2026-09-22  
结论：macOS 隔离子目标 **PASS**；Windows 未验证。验证当日（2026-09-22）`AD-EX-02` 保持 Proposed；2026-09-24 后续用户变更仅接受 macOS-only 缩范围结论，不改变本记录的双平台未通过事实。

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

失败样本按设计必须交回，因此整体交回率为 50%；≤20% 门槛只适用于成功路径，结果为 0%。验证当日费用字段为 `null`：当时官方公开材料未提供可用单价，未虚构费用。Jev 请求与 token 已完整记录。

2026-09-24 补证：TypeSafe 官方首页现已公开 `$42 / 10 亿 input tokens`，FAQ 说明当前价格可盈利而非临时亏损补贴；官方文档索引仍未发现 output token 计费规则。保持原始证据文件不变，另以 [价格结构化证据](verification-pricing-20260924.json) 回算：Jev 路径 8088 个 input tokens 为 `$0.000339696`，完整状态基线 17061 个 input tokens 为 `$0.000716562`，已知 input 费用下降 52.594%。该数值不包含任何未公开的 output 费用，不等同最终账单。

2026-09-25 补证：TypeSafe 官方发布页已明确 Jev output tokens 免费，且继续列出 input 单价为 `$0.042 / 百万 tokens`。保持 2026-09-24 的历史快照不变，新增[价格独立复核记录](verification-pricing-20260925.md)与[结构化证据](verification-pricing-20260925.json)。在同一 Jev 官方价格基准下，Jev 候选摘要路径的完整公开价格为 `$0.000339696`，完整状态基线为 `$0.000716562`，费用下降 52.594%；两侧 output tokens 均为 834 且价格为零。该对照用于验证候选摘要的 Jev 成本变化，不是外部慢脑供应商的实际账单。

## 结论与限制

macOS 隔离样本显示官方 `systemOne` 选择题接口可以在有界候选上稳定选择，且压缩候选摘要达到冻结 token、质量、时延与取消门槛。该结论只覆盖本 Spike 的固定样本，不证明真实 CUA/BUA/Document/Command Observe 质量，也不接产品 Gateway。2026-09-24 后续用户变更据此接受了 macOS-only 产品决策接线；Jev 官方 input/output 价格证据已补齐，但 Windows 证据仍缺失，完整计划片段执行和双平台终局结论仍不得据此解锁。
