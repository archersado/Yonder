# AD-CU-06 后台原生动作路由

状态：Proposed  
日期：2026-09-18  
关联：CU-S3、TM-S5、AD-CU-05、AD-CM-01、AD-OCT-06、AD-DS-02

## 决策问题

Yonder如何在不抢占用户前台桌面的情况下使用应用公开的原生能力，同时保持CUA单一执行链、任务恢复语义和Task Space可观测性。

## 候选决策

执行分为三种显式类别：后台原生动作、CUA动作和Command。后台原生动作调用经验证的系统API、应用SDK，或Yonder可编译链接的具体App Intent类型，不进入trycua Driver、不取得前台桌面租约；AX/UI与键鼠动作仍由trycua Driver执行；Command只执行显式结构化进程。Yonder不根据自然语言、目标应用或失败结果推断类别，也不做隐藏降级。

后台原生动作必须进入Application既有任务、步骤、attempt和事务边界。每次状态更新、对应事件与Outbox同事务提交，每任务sequence严格递增；步骤声明、开始、动作后Observe和结果因此形成同一可排序事件链。Task Space从该事件链展示动作类别、阶段、结果和有界证据摘要。日志、事件和Outbox不得保存正文、截图、完整命令输出、完整Agent Payload或原生对象。

原生动作返回`foreground-required`或不支持时停止；Agent可在Observe后显式提交新的CUA步骤。该步骤重新进行资源准入，不是原生动作的内部兜底。副作用结果不明为unknown，不自动重试。桌宠状态是宿主事件驱动的展示投影，不成为任务事实源。

所需系统权限必须在后台动作准入前已经授权。未决定、拒绝或受限时返回`permission-required`并进入显式等待用户流程；不得在后台动作内部弹出授权框。用户授权后重新Observe权限，由Agent显式提交新步骤，原副作用不自动重试。

## 与既有决定的关系

本决定不修改AD-CU-05的连续CUA会话：同一trycua会话内仍禁止插入第二套原生执行器。后台原生动作是独立声明的任务步骤，不是假装成CUA工具。它也不把Command改造成Apple API桥接，不允许AppleScript/osascript路线。

## Spike门禁

先以macOS隔离fixture验证后台动作不改变前台、动作后Observe、Task Space事件映射所需事实及不支持时无隐式改道。Spike不新增产品Port、协议或持久化。Windows按用户决定暂缓；双平台证据和至少一个具体真实应用能力通过前，本决定保持Proposed且不得开放产品Gateway。

## macOS子范围证据

2026-09-18隔离fixture通过`NSWorkspace.OpenConfiguration(activates=false)`后台启动，前台应用身份保持、动作后就绪Observe有效、未新增trycua Worker，不支持请求未调用CUA/Command/脚本，并输出步骤声明、attempt开始、Observe和结果四类顺序事实。独立Goal同时确认fixture正常退出。该结果只接受系统后台启动子范围，不证明通用App Intent、真实第三方应用SDK或产品Task Space接线；Windows暂缓，状态保持Proposed。

同一探针随后对真实系统计算器验证通过：应用原本未运行，后台启动未改变前台，运行身份Observe有效，验证后正常退出且无进程残留。这证明`NSWorkspace`路线不只对自有fixture成立；仍不证明应用内业务动作、通用App Intent或第三方SDK。

## App Intent通用路线结论

本机macOS SDK与计算器metadata只读探针显示：AppIntents公开执行面要求调用方持有具体`AppIntent` Swift类型，Donation接口也只接受具体类型；计算器存在一个metadata动作，但框架没有按外部应用metadata标识执行Intent的公共入口。通用跨应用App Intent Adapter路线淘汰。`/usr/bin/shortcuts run`是结构化Command候选，若未来支持须单独审阅参数、确认与副作用语义，不得在原生动作失败后隐式调用。

## EventKit业务动作证据

提醒事项权限最初为`not-determined`。直接执行bundle内二进制无法建立授权会话且未写数据；经LaunchServices首次授权后，创建、identifier Observe和删除清理成功，但系统授权界面改变前台，因此该轮按后台验收失败。权限已为`full-access`后重跑，三项副作用及清理全部通过且前台保持。产品据此必须把权限获取与后台动作拆成两个显式步骤；该证据不授权通用原生Adapter或其他EventKit业务能力。

最终探针按该决定改为同一临时App的两个模式：`authorize`只提交permission-required/permission-observed事实，不保存提醒；随后`background`启动时已为full access，才提交步骤声明与attempt，完成创建、identifier Observe、删除清理并提交Observed结果。旧版在后台内部请求权限后继续执行的结果即使未改变前台也明确拒绝。
