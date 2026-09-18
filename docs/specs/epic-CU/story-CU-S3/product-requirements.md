# CU-S3 产品需求

## 问题与目标

部分应用能力可通过系统原生 API、App Intent 或应用 SDK 在后台完成，无需占用前台桌面。Yonder需要保留这种能力，同时让用户在 Task Space 中看到动作过程；需要界面操作时再由 Agent 显式提交 CUA 步骤。

## 需求来源与分类

- 原始需求：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)的“产品定义”“MVP 主干链路”“两条执行路径”，要求任务可见、可控并与用户工作共存；[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)的“执行原则”要求步骤可观察。
- 后续用户变更（2026-09-18）：应用公开的原生 API、App Intent 或 SDK 动作不经过 CUA Driver；AX/UI 与键鼠动作继续走 CUA Driver；后台执行以桌宠小图标或状态标记呈现，仅在确需界面时进入前台；所有动作必须被 Task Space 观测。Spike进一步确认：只有Yonder可编译链接的具体Intent类型或Apple未来提供的公开跨应用调用API才能归入原生动作，已安装应用的`Metadata.appintents`本身不是执行接口。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)的“CUA 与 BUA”“任务、状态与恢复”“Command、File 与 Document”；Accepted AD-CU-05 禁止在连续 CUA 会话中插入第二套执行器，Accepted AD-OCT-06 规定资源准入，AD-DS-02 规定桌宠展示由宿主事件驱动。
- 待审设计建议：使用显式执行类别区分后台原生动作和 CUA；先验证一个具体动作，再决定是否需要产品 Port。此项不是原始产品范围。

## 验收条件

- NAT-01：Agent显式提交受支持的后台原生动作时，Yonder不启动trycua、不取得前台桌面租约，也不激活目标应用。
- NAT-02：后台原生动作必须绑定现有`task_id`、步骤和attempt；步骤声明、开始、动作后Observe及结果进入同一任务事件链，Task Space按sequence显示动作类别、阶段、结果和有界证据摘要。
- NAT-03：Task Space不得显示正文、截图、完整命令输出或完整Agent Payload；失败、超时及不支持必须稳定可区分。
- NAT-04：后台动作完成后必须Observe预期条件；Observe失败或副作用结果不明时记为unknown，不自动重试。
- NAT-05：需要AX/UI或键鼠时，Agent必须显式提交新的CUA步骤。Yonder不得因原生动作不支持或失败而自动降级到CUA。
- NAT-06：AX/UI与键鼠步骤继续使用CUA Driver；键鼠步骤取得唯一前台桌面租约，用户输入立即暂停，每步后Observe。
- NAT-07：Command只执行显式`program + args + cwd + env`，不得成为调用Apple API或绕过CUA的通用桥。
- NAT-08：桌宠通过宿主事件显示“后台执行”；确需前台桌面时显示“需要桌面操作”或既有接管状态，不从UI状态反推任务事实。
- NAT-09：未注册的原生能力返回不可用；Yonder不按自然语言、应用名或失败原因进行语义路由。
- NAT-10：Yonder不得从任意应用的`Metadata.appintents`生成通用调用器；`shortcuts run`若未来显式支持，必须作为Command能力审阅，不能伪装成后台原生Adapter。
- NAT-11：后台原生动作只能在所需系统权限已经授权时准入；首次授权、权限恢复或系统确认必须转为显式等待用户流程，不能在“后台执行”状态中弹出并抢占前台。

## 范围与非目标

本 Story 不建立通用自动化规划器，不复制 trycua 动作，不引入 AppleScript/osascript，不创建第二个常驻应用或服务，也不承诺任意第三方应用的 App Intent 可由Yonder直接调用。Windows 技术对照暂缓，完整 Story 不得 Archive。

## 验收映射

| 来源 | 验收 |
|---|---|
| 产品简报：任务可见、可控 | NAT-02、NAT-03、NAT-08 |
| 补充材料：执行原则、动作后Observe | NAT-02、NAT-04、NAT-06 |
| 2026-09-18 用户变更 | NAT-01、NAT-05、NAT-08、NAT-09 |
| macOS AppIntents公开SDK面Spike | NAT-09、NAT-10 |
| macOS EventKit权限与副作用Spike | NAT-02、NAT-04、NAT-08、NAT-11 |
| 架构主干与既有ADR | NAT-03、NAT-04、NAT-06、NAT-07 |
