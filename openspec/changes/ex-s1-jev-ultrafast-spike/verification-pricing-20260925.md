# EX-S1 Jev 价格证据独立复核

Story：EX-S1  
OpenSpec：ex-s1-jev-ultrafast-spike  
日期：2026-09-25  
结论：Jev 官方 input 与 output 价格口径 **PASS**；不替代 Windows 运行验证。

## 证据目标

补齐 2026-09-24 尚未找到的 output token 计费规则，并复核固定样本的价格计算。历史证据文件保持不变，新结果写入 [verification-pricing-20260925.json](verification-pricing-20260925.json)。

## 官方依据

TypeSafe 于 2026-09-15 发布的 [Introducing System One Models & Jev](https://typesafe.ai/blog/introducing-system-one-models-and-jev) 价格表同时列出：

- Jev input tokens：`$0.042 / 百万 tokens`；
- Jev output tokens：免费。

该页面是 TypeSafe 官方发布页，足以关闭“取得官方 output token 计费规则或明确不计费依据”的任务项。

## 独立回算

计算规则为 `input_tokens × 0.042 / 1,000,000 + output_tokens × 0`：

| 路径 | Input tokens | Output tokens | 同一 Jev 官方价格基准 |
|---|---:|---:|---:|
| 完整状态基线 | 17061 | 834 | `$0.000716562` |
| Jev 候选摘要 | 8088 | 834 | `$0.000339696` |

费用降低率为 `(0.000716562 - 0.000339696) / 0.000716562 = 52.59363460524002%`，四舍五入为 52.594%。

## 限制

- “完整”只表示 Jev 已公开的 input 与 output 两项价格均已计入，不代表税费、合同折扣或未来价格不变。
- 完整状态基线按同一个 Jev 官方价格基准回算，用于隔离候选摘要带来的成本变化；没有取得或比较外部慢脑供应商账单。
- 本复核不执行模型请求，不改变 2026-09-22 的运行证据，也不满足 EX-S1 的 Windows 双平台门禁。
