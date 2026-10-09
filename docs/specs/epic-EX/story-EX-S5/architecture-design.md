# EX-S5 架构设计

## 所有权与数据流

Application 定义 `ExecutionExperienceStore` Port 和纯值对象，负责准入、脱敏、去重与排序；SQLite Adapter 实现建议性投影。该投影不是任务当前状态，不参与任务 sequence、事件或 Outbox 的事务，也不能覆盖 ExecutionRuntime 或历史任务事实。

```text
已核验完成任务 ──异步/事后──> 经验投影（稳定非敏感特征）
                                      │ 精确索引，最多 3 条
handback ──> TaskHost 有界召回 ───────┘
   └────────> AG-S5 既有输入通道 ──> 归属慢脑读新鲜事实并 replan
```

经验键只允许平台、Driver、应用稳定标识、动作语义、控件角色和交互方式；值只允许通用前置条件、通用成功条件、成功/失败计数及最近核验时间。任何来源字段在写入前须通过封闭枚举、长度和控制字符校验。

## 晋升与召回

首版仅在归属慢脑提交 `task.goal.verify(achieved)` 后，从该任务最后一个已 Observe 成功的非敏感动作提取经验。动作须能归一化为封闭策略；发送、输入文本、坐标点击、任意快捷键、shell 和文件写入不晋升。相同键使用计数更新而非新增无界记录。

发生 handback 时，TaskHost 从当前片段/最近 attempt 形成不含临时目标的查询键。召回按完全匹配优先、成功率和最近核验时间稳定排序，最多三条。读取设置严格条数与耗时预算；异常返回空集合。Agent 输入明确标注提示是历史经验、必须结合新鲜事实验证。

## 依赖与失败

Desktop 只依赖 Application Port 和 Adapter 实现；Application 不引用 SQLite；Driver/Jev/React 不访问经验投影。召回失败不修改任务状态、事件、Outbox 或浮窗执行状态。旧坐标、旧 AX 索引和旧窗口绑定永不进入查询结果，因此不能跨任务重放。
