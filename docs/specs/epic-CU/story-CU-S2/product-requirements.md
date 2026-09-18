# CU-S2 产品需求

## 问题与目标

每一步后 Observe，用户输入暂停，崩溃超时结果 unknown。

## 范围与非目标

本 Story 仅负责“受监管桌面执行与 Observe”。Worker 停止/Observe 契约与平台证据未齐。

## 验收条件

- 目标行为有可复现成功样本，失败不得伪报成功。
- 每一步后 Observe，用户输入暂停，崩溃超时结果 unknown。
- 平台限制与未实现项明确；涉及 UI/Driver/权限时分别提交 Windows/macOS 原生证据。
- DISPATCH-01：仅接受绑定当前 task/step/attempt/Worker/host 的受监管动作结果。
- OBSERVE-01：后台 AX 输入返回后必须重新 Observe；后置 Observe 无效不得报告成功。
- UNKNOWN-01：Worker 超时、崩溃、断连或非法响应返回 unknown，不自动重试。
- APP-FOCUS-01：`launch_app`保持SDK后台启动语义；Yonder不得把单个SDK动作隐式改写为动作序列。
- APP-FOCUS-02：同一任务随后显式调用`bring_to_front`时，Yonder只可注入该SDK会话最近一次启动结果中的可信应用/窗口身份，不接受Agent提交PID或窗口号。
- APP-FOCUS-03：跨Space前置必须经后置Observe证明用户当前可见；SDK拒绝时不得报告成功，可由Agent显式组合SDK公布的Dock键盘动作后再次Observe。
- APP-FOCUS-04：不得使用硬编码Dock坐标、AppleScript或第二套桌面动作实现。

AD-TM-08 已接受尝试准备子范围；本轮受监管派发与后置 Observe 以 [AD-CU-04](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-04-SUPERVISED-DISPATCH-OBSERVE.md) 为实施决定。后文保留的 `Proposed` 是 2026-09-14 历史审阅状态。

## 待决事项

Worker 停止/Observe 契约与平台证据未齐。

## Agent Gateway增量（2026-09-16）

来源为产品简报“两条执行路径”、统一任务/权限/事件模型及用户要求集成CUA/BUA能力。验收CGW-01 Agent只提交任务、序号、动作种类和动作正文，不提交PID/窗口/元素/Worker路径；CGW-02 Yonder只操作请求时最前方唯一工作窗口，每步强制Observe；CGW-03真实用户输入立即中断Agent输入，不重试或自动恢复；CGW-04最后步骤经Agent显式完成并释放资源。首批仍仅支持已验证的后台AX文本输入。

## 需求来源

- 产品依据：[产品简报](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/brief.md)、[补充材料](../../../../_bmad-output/planning-artifacts/briefs/brief-Yonder-2026-09-09/addendum.md)，对应章节：两条执行路径；补充材料 执行原则、验证方案。
- 架构约束：[架构主干](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/ARCHITECTURE-SPINE.md)，对应章节：CUA 与 BUA。具体 ADR 以架构设计列出的状态和平台范围为准。
- 本 Story 是上述需求的模块内分解，不代表整条产品链路完成；跨模块与遗漏项见 [覆盖映射](../../REQUIREMENTS-TRACEABILITY.md)。具体阈值、字段和布局若无原文依据，应作为设计建议审阅，不能称为用户要求。

## MVP步骤边界控制（AD-CU-02，2026-09-14）

来源为原产品执行原则/逐任务控制、架构每步Observe及用户SDK-only人工接管/工作定位变更。冻结新动作→等待当前步骤返回并Observe→结果已知且attempt/Worker一致后确认停止→前置对应工作→满足隐私及显式记录条件后接管。边界等待显示正在停止，不提前显示已接管。超时/断连/Observe失败保持unknown与占用，不自动重试或录制；旧身份回调不释放新租约。合成延迟控件测试未成立，不作为额外产品前置；具体控制协议/持久化/目标及Recording仍待联合定稿，真实步骤竞态/停止/Observe和两平台证据门禁保留，Windows暂缓。

## 工作定位原生路线审阅输入（2026-09-14）

来源：本Story既有产品简报Task Space/逐任务权限映射、用户接管时前置正在操作工作与SDK-only变更、Accepted AD-CU-02/AD-TM-07/AD-CU-03。验收FOCUS-01停止确认后才定位；FOCUS-02可信当前任务目标唯一前置并保持工作屏幕原位置；FOCUS-03关闭/失效/不唯一保持暂停，提示“未能定位任务工作，请手动打开”，不前置同名工作；FOCUS-04小龙不移动，BUA仍外部引用。

macOS隔离正常前置、最小化恢复与关闭拒绝已验证；SDK本身无精确focus接口，产品沿CU内部原生AX/CoreGraphics路线。WindowServer可见和AX就绪分开；界面显示“正在定位工作”直至实际焦点/可见状态确认，返回成功不能提前显示已移交。定位失败保持暂停，显式记录条件仍按RC门禁；UI不接受PID/路径/任意Agent命令。

独立verification-focus-macos.md仅技术基线通过。生产当前WorkRef/attempt/Worker/启动身份、双侧唯一映射、事务事件和多Space/显示器及权限责任链尚未完成，不据此把控制协议设计标ready或启用接管。Windows用户暂缓，完整Story保持原状态。

## 受监管步骤与工作引用验收补齐

字段级设计见[AD-TM-08](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-08-EXECUTION-IDENTITY-AND-STOP.md)（Proposed）。产品简报「两条执行路径」及补充材料「执行原则」映射STOP-02每步Observe；架构执行恢复映射EXEC-ID01身份隔离、STOP-01冻结后不新增动作；用户对应工作/SDK-only变更映射FOCUS-05可信精确目标及所属Space/显示器核验；用户手动接管记录映射TAKE-07，Driver停止或定位不自动开启Recording。原需求完整保留，字段与技术细节属于设计建议。

2026-09-16验收补充：WORKREF-01仅有效后置Observe可捕获当前完整身份引用；WORKREF-02定位前复核进程启动身份与WindowServer/AX唯一映射；WORKREF-03关闭、替换、不唯一、权限或焦点核验失败均拒绝且无替代目标副作用；WORKREF-04保持原几何，不启动新App或Recording。来源为产品“对应工作”、用户SDK-only要求及Accepted AD-CU-03。
2026-09-16后续用户变更：Yonder不重复定义CUA SDK动作；Agent通过通用桥接调用SDK公布的动作，Yonder只监管目标、权限、生命周期、Observe和持久化。验收映射：SDK动作变化无需修改Yonder协议枚举；受保护目标字段必须拒绝；每次调用仍须Observe。

2026-09-16连续动作补充：同一任务的相邻CUA步骤不得因Yonder内部实现切换动作执行器或逐步重建trycua Driver；正常序列复用受监管trycua会话。每步仍有独立attempt与Observe；用户输入、超时、崩溃等unknown边界必须终止会话，不能为保持连续而自动重试。原生目标解析和输入监听属于监管信号，不构成第二动作执行栈。

2026-09-16调用链精简补充：Agent每个CUA动作只调用一次`computer.step`，由Yonder内部完成步骤声明、执行、Observe和边界推进；不要求Agent重复搬运中间sequence。响应须提供紧凑动作结论与可读取的有界Observe证据，使本地Agent无需调用`osascript`或`screencapture`确认界面。截图不进入日志、事件、Outbox或长期上下文，任务结束即清理。
