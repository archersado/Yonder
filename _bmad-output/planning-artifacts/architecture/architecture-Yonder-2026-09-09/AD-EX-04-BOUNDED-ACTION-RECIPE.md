# AD-EX-04 有界动作配方优化路线

状态：Proposed
日期：2026-09-22
关联：AD-EX-01、AD-EX-02、AD-EX-03、EX-S1、EX-S2

## 决策问题

AD-EX-01 的当前基线是：Jev 在一个有界计划片段内，每次 Observe 后从可信候选中选择下一个动作和目标。该架构保留逐动作安全校验与 Observe，但在稳定表单、固定 UI 流和重复操作中，可能导致 Jev 调用次数与端到端时延偏高。

用户提出参考 Claude Code workflow 或 ego-lite 生成浏览器执行代码的模式：慢脑预先给出步骤内更完整的执行描述，Driver 按批执行，Jev 只在 Observe 出现问题时介入修复。该提议不能被理解为让模型或 Driver 执行任意代码、通用 DSL 或第二执行栈；必须先作为 Proposed 路线观察后续执行数据，再决定是否采纳。

## Proposed 候选：Bounded Action Recipe

计划片段可扩展一个**类型化动作配方**（Bounded Action Recipe）：慢脑一次提交有序步骤，每步只包含预允许动作、目标谓词、参数引用、声明式 guard 和失败策略。Application 逐步解释 Recipe，在每次实际动作前后执行 Observe 与安全校验；guard 通过时直接派发下一条类型化动作，guard 失败、目标缺失、目标多义或预算接近耗尽时才调用 Jev 选择修复候选或上交慢脑。

示意：

```json
{
  "recipe_version": 1,
  "steps": [
    {
      "action": "type",
      "target_predicate": { "role": "textbox", "label": "收件人" },
      "param_id": "recipient",
      "guard": { "target_found": true }
    },
    {
      "action": "click",
      "target_predicate": { "role": "button", "label": "发送" },
      "guard": { "target_found": true }
    }
  ]
}
```

这不是可执行代码或通用工作流语言。禁止任意循环、条件表达式、Shell、自由文本生成、动态代码解释和跨 Observe 复用目标引用。CUA 继续通过 AX 解析目标并调用平台动作；BUA 继续委托 ego-lite Browser Task Space，浏览器侧内部实现不得变成 Yonder 的第二任务状态或执行栈；Document 与 Command 继续只接受类型化操作和结构化命令。

## 保留当前基线

在后续真实执行数据证明 Recipe 明显更优前，Yonder 的产品架构和实施计划保持 AD-EX-01 的「Jev 每步决策」版本。Recipe 不进入 Story、OpenSpec 或运行时代码；不得因本 Proposed ADR 授权协议字段、数据库迁移或 Driver 批处理实现。

## 观察与采纳门槛

Recipe 只能作为 AD-EX-02 后续观察中的对照路线，与每步 Jev 基线使用相同样本和任务集。比较指标至少包含：

- 端到端时延中位数与 p95。
- Jev 调用次数、请求量与费用。
- 慢脑补参或 replan 次数。
- 成功率、误动作率、旧目标动作率、上交率。
- `unknown`、取消、接管和用户输入中断的正确处理率。
- 目标缺失、目标多义、页面/窗口变化后的恢复表现。

具体数值门槛须在实验前冻结。最低要求是：安全与控制指标不得劣于每步 Jev 基线，且必须在稳定场景中显著降低 Jev 调用或时延；否则维持基线并关闭本路线。Windows 仍按用户决定暂缓，不能把 macOS 单平台数据作为 Accepted 结论。

## 后果

- 本 ADR 仅记录候选优化路线，不改变 AD-EX-01 的当前职责边界。
- Recipe 由慢脑提交，Jev 不能生成或改写 Recipe；Jev 只能在失败时选择当前 Observe 的修复候选、预授权恢复分支或上交。
- Driver 不解释 Recipe，不接收代码；Application 是唯一解释和派发方。
- 每个实际动作后仍强制 Observe，敏感操作仍须确认，副作用失败仍进入 `unknown` 并交回慢脑。
- 若后续数据支持采纳，必须先更新本 ADR 为 Accepted，再修订 EX-S1/EX-S2 设计与 OpenSpec，最后实施协议和验证。
