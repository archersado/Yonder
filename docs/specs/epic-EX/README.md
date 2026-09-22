# EX 执行快脑与快慢脑交接

Epic: EX

状态：draft（系统边界 [AD-EX-01](../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-EX-01-BOUNDED-FAST-BRAIN.md) Accepted；技术路线 [AD-EX-02](../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-EX-02-JEV-INTEGRATION-ROUTE.md) Proposed）  
Epic 分支：`epic/ex`  
来源：产品简报「产品定义」「MVP 主干链路」「两条执行路径」、补充材料「执行原则」「CUA 与 BUA 的 Task Space」、架构主干「产品边界」「任务、状态与恢复」「执行能力」；2026-09-21 用户新增快慢脑分工。

## 目标与边界

外部 Agent（慢脑）经现有 Agent Gateway 制定首次计划，并在执行偏离或局部计划不再适用时经同一 Gateway 提交 replan。Yonder 可内置 Jev 快脑循环，在计划授权的局部范围内从最新 Observe 派生的动态候选集中连续选择下一操作及目标；动作必须通过原有 CUA、ego-lite BUA、Document 或 Command 执行层及任务事务。目标是减少慢脑逐动作调用与 token，同时不降低成功率或用户控制能力。Jev 无自由文本生成能力，输入文本/文档改写/命令参数来自慢脑计划；不能把猜测的内容当动作候选。

```text
慢脑首次计划/replan → 现有 Gateway → Application（候选生成/策略/状态/租约）
                            ↓ Jev 选择操作 + 目标
                       既有执行 Adapter → 强制 Observe
                            ↓
              继续局部步骤 / 完成验证 / 经 Gateway 交回慢脑 replan
```

本 Epic 不复制 ego-lite Task Space，不新增通用 Agent、慢脑规划器、第二任务状态机、通用模型框架或 Driver 间调用，也不建立慢脑直连通道。BUA 在 ego-lite 既有 Task Space 内执行；Yonder 只持有其关联引用和本地执行事实。实际 Jev 服务形态、数据边界及接入协议先由 Spike 验证。

## Story 顺序

| Story | 交付 | 前置与门禁 |
|---|---|---|
| [EX-S1](story-EX-S1/README.md) | Jev 有界动作决策 Spike 与双平台对照 | 限时 2026-10-05；先冻结样本/淘汰门槛，输出 AD-EX-02 证据；Windows 暂缓期间不得接受技术路线 ADR。 |
| [EX-S2](story-EX-S2/README.md) | 快慢脑计划片段、决策循环、交回契约及最小 Jev 配置界面 | EX-S1 与 AD-EX-02 Accepted；复用 TM-S7 统一执行入口，未就绪时阻塞。 |
| [EX-S3](story-EX-S3/README.md) | CUA 与 BUA 的动态候选动作接线 | EX-S2；CU/BU 既有 Observe、目标新鲜度、租约与 Task Space 门禁。 |
| [EX-S4](story-EX-S4/README.md) | Document 与 Command 的有界执行接线和端到端对照 | EX-S2；DO/FI/CM 各自产品门禁；结构化写入参数由慢脑提供。 |

每个 Story 独立短分支、一个实施 OpenSpec Change、独立 Verification Goal；Spike Change 与产品实施 Change 分开。当前均为设计初稿，不生成实施 Proposal 或修改运行时代码。

## 现有 Story 影响与开工判断

| Story | 需调整或复用的契约 | 当前可推进范围 |
|---|---|---|
| AG-S1、AG-S3 | Gateway 会话身份继续唯一；AG-S3 现有步骤声明仅接受归属 Agent。EX-S2 须新增绑定已验证计划的可信内部步骤来源，不能冒充 Agent 或开第二 Gateway。 | AG-S1 本地连接身份绑定可按既有 Change 继续；快脑计划/replan 协议等 EX-S2，AG-S3 既有声明能力无需返工。 |
| TM-S2、TM-S7 | 复用单一任务状态、attempt/Observe/Outbox 和首次副作用启动事务；内部快脑步骤的作者、幂等和停止排序须联合定稿。TM-S7 目前仅在工作区有 design-review/Proposed AD-TM-13，不能视为已合入基线。 | TM-S2 既有验证继续；TM-S7 可推进架构审阅，代码须等 AD-TM-13 定案和 Story/OpenSpec 门禁，Document/Command 还受各自门禁。 |
| CU-S2、BU-S2 | Driver/ego-lite Bridge 保持模型无关；EX-S3 增加从 Observe 枚举可信候选、派发前复核与快脑来源，旧 Agent 逐步调用继续兼容。 | 既有 Driver/Bridge 验证可继续；快脑产品接线等 EX-S2/AD-EX-02。 |
| DO-S2、FI-S1、CM-S1 | 原有 OOXML 文件锁/身份、结构化命令与副作用围栏不变；EX-S4 只选择参数已完整的候选。 | 各自双平台 Spike/验证继续；Document/Command 产品 Gateway 仍受 FI-S1/CM-S1 门禁，EX-S4 未就绪。 |
| TM-S5、DS-S2、AG-S4 | TM-S5/DS-S2 需展示快脑执行、交回慢脑和结果待核实的最小事件/状态；AG-S4 Skill 在能力齐备后说明快慢脑入口，不建旁路。 | 现有时间线/任务菜单和 Skill 规格可继续，新增快脑展示/Skill 实施等 EX-S2 合约。 |

最先可启动的是 **EX-S1 的限时 Spike**：补齐并审阅统一样本/淘汰数值后创建独立 Spike Change，在 macOS 做隔离技术验证；Windows 暂缓时只形成子结论。与之并行可继续 AG-S1 的现有身份绑定，以及 TM-S7 的共享启动设计与 AD-TM-13 审阅。**EX-S2～S4 目前均不可开始产品实施**；AD-EX-02 双平台通过、TM-S7 与各执行层门禁是必经前置。

## 统一验收口径（待 EX-S1 冻结数值）

使用相同任务、初始计划、授权、设备和 Observe 条件对照「慢脑逐步决策」与「Jev 局部决策」。记录任务成功率、误动作/恢复、慢脑调用次数与 token、Jev 请求次数/成本、端到端时延、用户中断响应、`unknown`/交回率；将模型输入正文排除在持久日志之外。成功样本不能替代失败和敏感操作门禁。TypeSafe 与 browser-use 公开演示数据仅作路线参考，不是 Yonder 验收结论。
