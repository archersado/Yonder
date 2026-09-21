# AD-EX-01 有界执行快脑与外部慢脑分工

状态：Accepted（系统边界；技术接入仍由 AD-EX-02 验证）  
日期：2026-09-21  
关联：EX-S1～EX-S4、TM-S2/TM-S7、AG-S1、CU-S2、BU-S2、DO-S2、CM-S1

## 决策问题与来源

原产品简报「产品定义」「MVP 主干链路」「两条执行路径」要求外部 Agent 发起任务，Yonder 负责本地受控执行、Observe 和用户接管；补充材料「执行原则」及此前架构主干进一步禁止 Yonder 内置 Agent、Planner 或模型循环，Observe 后由外部 Agent 决策。2026-09-21 用户明确决定修正这条围栏：允许 Jev 作为位于 CUA/BUA/Office/Command Driver 之上的执行快脑，外部慢脑负责首次行动规划和需要时的 replan，以减少终端执行 token 并提高质量。本 ADR 接受**系统职责调整**，不等于 Jev 技术接入已验证或产品 Story 可实施。

用户已确认「Jev」指 TypeSafe AI 的结构化决策模型。其官方说明是预定义输出结构的概率决策，并非任意文本生成器：[产品说明](https://typesafe.ai/)、[发布说明](https://typesafe.ai/blog/introducing-system-one-models-and-jev)。参考 [browser-use/jev-ultrafast](https://github.com/browser-use/jev-ultrafast)：每次 Observe 建立动态编号元素表，Jev 同轮选择操作及兼容目标；只有输入文本时另用小模型，执行前复核目标新鲜度。该仓库是单浏览器演示，不证明 Yonder 跨 Driver 效果或可直接复用其 Python/Browser Harness。实际 API、数据驻留、授权、成本和可用平台均须 EX-S1 核实。

## 决定的边界

外部 Agent 仍是任务目标、首次计划、计划偏离后的语义 replan 和最终结果的决策者。Yonder **允许内置有界模型决策循环**：在一个已授权、已版本化、范围有界的计划片段内，每次 Observe 后基于动态枚举的受支持操作及可信目标，调用 Jev 选择操作和兼容目标、判断片段内继续/完成候选，或选择 `escalate`。快脑可以连续驱动多个动作，不必每步回到慢脑；执行前重新校验目标、计划版本和权限。计划目标变化、缺少关键参数、偏离预期或预算耗尽时交回慢脑 replan。Jev 不得自由生成命令、文档改写内容、接收人、支付对象、权限或新动作种类；自由文本或结构化写入参数应由初始计划提供，缺失时交回慢脑。参考仓库的小模型文本助手可作为后续独立设计建议，首版不引入第二模型。Jev 不是新的任务事实源、Driver、Gateway 或通用桌面 Agent。

慢脑接入链路保持现状：本地 Agent 经 MCP stdio/CLI 与 Local Socket、云端 Agent 经 Yonder 主动建立的 WSS，**都先进入同一个 Agent Gateway 和 Application 用例**。初始计划与 replan 必须携带既有可信 `agent_id`、`request_id`、`task_id`、能力和截止时间，并核对任务归属与计划版本。快脑要求 replan 时，只把有界 Observe/失败摘要写入现有任务事件与 Outbox，再通过 Gateway 对当前归属 Agent 可见；不得直连慢脑、创建第二个 Agent 通道，或让 Jev 自行调用外部 Agent。新计划仍须由归属 Agent 从 Gateway 提交，不能把一次快脑选择当作新计划。

现有 AG-S3 步骤声明和 CU-S2 连续步骤由 Agent 发起。接入快脑时，Application 可在已验证计划的作用域内使用**独立的可信内部来源**创建后续步骤/attempt，并在事件中保留归属 Agent 与快脑执行来源；不得借用 Agent 会话或伪造 `agent_id` 调用 Gateway。该内部来源的协议、事务、幂等和停止排序由 EX-S2 联合 AG-S3/TM-S2/TM-S7 定稿。Jev 的 `DONE` 仅是待验证的片段结果，不替代归属 Agent 的 `task.complete`/`task.fail` 或用户确认。

```text
外部慢脑 Agent ── 初始计划 / replan ──> Agent Gateway
                                         │
                               Application 执行协调
                               ├─ 策略/授权/资源准入
                               ├─ 有界候选 + Observe → 快脑决策 Port → Jev Adapter
                               ├─ 已选动作 → CUA / ego-lite BUA / Document / Command Adapter
                               └─ 任务当前状态 + 事件 + Outbox（同一 SQLite 事务）
                                         │
                              偏离、低置信、预算耗尽 → 外部慢脑
```

Application 持有执行循环与决策 Port；Adapters 只实现 Port 和具体 Driver，不互调。Rust 协议类型仍是唯一协议来源。BUA 只引用 ego-lite Task Space，不在 Yonder 复制浏览器任务空间。既有任务、步骤、attempt、sequence、事件、Outbox 和资源租约仍是执行事实；快脑决策只能关联这些身份，不能创建第二套状态机或重复记录完整上下文。

每次实际动作仍经现有授权、确认、资源准入、步骤/attempt 提交和 Driver；动作后强制 Observe，再决定是否继续有界片段或上报外部 Agent。初次观察、动作结果、步骤边界及 replan 请求须有可排序的最小事件；不得存正文、截图、完整命令输出或完整模型 Payload。低置信、候选无效、计划偏离、敏感操作、用户输入/接管、超时、断连、崩溃、`unknown` 或预算耗尽时冻结新动作并交回外部 Agent；`unknown` 必须先重新 Observe 且不得自动重试副作用。快脑不可绕开 Agent 显式恢复、取消与停止确认。

## 技术路线门禁

- Jev 是否能以动态受控候选集稳定选择真实 CUA/BUA/Document/Command 下一动作；哪些动作能由 Observe 枚举，哪些参数只能由慢脑给出。若仍需慢脑逐步生成候选，token 目标可能不成立。
- Jev 是远端服务还是可嵌入本机的运行时，以及网络延迟、隐私与离线行为；未证明前不得声称「放到 Yonder 里」等于本机模型部署。
- 与「慢脑每步决策」基线比较任务成功率、误操作率、慢脑 token、端到端时延和用户打断响应；阈值在统一样本运行前写入 Spike，不用厂商营销数据作验收。
- 跨平台能力、权限、数据传输最小化、供应商故障处理和费用。不得为远端 Jev 在本机开放 HTTP/TCP 服务；如需出站调用，应通过 Yonder 受控 Connector 并先定稿认证与数据边界。

EX-S1 为限时 Spike，统一样本和淘汰门槛见 Story 设计，技术选型结论单列 [AD-EX-02](AD-EX-02-JEV-INTEGRATION-ROUTE.md)。按现行研发围栏，双平台证据与 AD-EX-02 Accepted 前，EX-S2～S4 不生成实施 Proposal。Windows 当前按用户既有决定暂缓；本系统边界决定已经生效，但不能把 macOS 单平台结果记为技术路线通过。

## 与既有决策的关系

本决定撤销「Yonder 一律不内置模型循环」；改为禁止内置**通用 Agent、首次规划和跨计划语义 replan**，允许上述有界快脑模型循环。原产品简报和补充材料中的旧边界保留为历史来源，由本次用户变更及本 ADR 覆盖对应冲突；Driver 模型无关、任务事实源、Observe、停止/接管和安全围栏继续生效。实施前仍须复核 AD-CU-04/05、AD-BU-02、AD-TM-08/13 的 Observe 和控制契约。
