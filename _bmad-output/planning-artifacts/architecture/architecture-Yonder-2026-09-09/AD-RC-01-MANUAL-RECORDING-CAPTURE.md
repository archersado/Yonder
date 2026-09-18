# AD-RC-01 手动 Recording 原生采集与证据边界

状态：Proposed（macOS Spike 可开始；产品持久化、回放和 Windows 未授权）；日期：2026-09-17。关联 RC-S1、TM-S3/S4/S5、CU-S2、CX-S1、DS-S2 与 Accepted AD-TM-03/08。Architecture Impact：architecture-change（原生采集 Port、Recording 事实、持久化与后续协议）。

## 来源与目标

产品简报“显式上下文录制”“MVP 主干链路 3–4”要求默认不采集、用户主动 Record 后记录操作时间线与授权上下文；架构主干“Recording 与桌宠”要求每设备最多一个、原始时间线不可变、仅 `user` 事件派生轨迹；2026-09-14 用户变更要求人工接管时记录用户行为，作为交回 Agent 的 Observe 依据。

本决策先验证原生事件来源、隐私排除和有界采集是否可靠。验证通过前不新增产品 Recording 表、外部协议或回放入口。

## 拟定边界

1. 用户点击任务“接管”只表达开始意图。只有当前任务已提交 `paused`、takeover 控制为 `stopped`，且原工作定位成功或用户随后显式重试 Record，RC Application 才可建立 Recording；普通用户输入、自动暂停和 Agent 请求都不能开始。
2. 每设备最多一个活动 Recording。记录对象绑定 `recording_id/task_id/control_id` 和宿主会话；冲突明确拒绝，不覆盖旧记录。
3. 原生 Adapter 只上报受限事件：单调时间、事件种类、来源分类、经隐私检查后的目标引用及必要动作参数。应用日志、Outbox 与 Spike 证据不得包含按键正文、截图、AX 文本、坐标明细或完整 Payload。
4. 密码框、Secure Event Input、系统安全界面和用户排除应用始终产生“已排除/证据缺口”事实，不产生可回放动作。队列溢出、Tap失效、权限撤销同样结束完整性保证并显式标记 gap，不静默丢失。
5. `agent_cua` 与 `replay` 不派生用户轨迹。Yonder 自身执行期间由桌面租约阻止 Recording；未来 Replay 必须在原生注入边界携带可信来源标记。不能仅凭窗口标题、进程名或客户端字段判断来源。
6. 原始时间线追加后不可修改；派生 Trajectory 单独版本化。停止 Recording 只关闭记录边界，任务仍保持人工接管/paused。交回的新鲜 Observe、证据引用交付和显式恢复归 TM-S4/AG，不由 RC 自动执行。
7. MVP 暂不加密不放宽采集范围和日志禁令。是否允许未加密持久化输入正文必须另行决策；本 Spike 不保存正文。

## 首个 Spike

macOS 候选使用系统 `CGEventTap` 的 listen-only 会话与 AX/系统安全状态做来源和隐私门禁，不引入第二个 App 或第三方录制运行时。统一样本包括：真实用户鼠标/键盘、程序注入事件、Secure Text Field、排除应用、Tap禁用与有界队列溢出。只输出计数和稳定分类。

淘汰门槛：无法稳定区分真实用户与 Yonder 注入来源；密码/安全界面事件可能进入有效动作；Tap失效或队列溢出无法显式形成 gap；停止后仍接收事件；需要持续高权限 Sidecar 或第二个 App。任一命中即不进入产品实现。

Windows 对应路线需使用受控低级输入钩子与 UI Automation 做同一样本。用户已决定暂缓 Windows，因此本 AD 在只有 macOS 证据时保持 Proposed，RC-S1 不 Archive。

## 2026-09-17 首轮来源验证

CGEventTap本身可用且无正文探针的容量/停止自检通过，但真实CUA REPL点击产生的51个事件全部呈现`source_pid=0/source_tag=0`，与物理用户候选不可区分。纯CGEvent来源字段路线按淘汰门槛失败，不得接产品。

下一候选仅增加系统原生IOHID输入值与CGEvent单调时间关联：只有同类硬件事件在可校准窗口内匹配才标记`user`；未匹配一律`external_unknown`并不派生轨迹。探针仍不保存usage/keycode/坐标正文。该候选需要Input Monitoring权限与真实硬件样本，通过前本AD保持Proposed。

首个负样本中Input Monitoring已授权；CUA REPL点击产生28个CGEvent但没有IOHID标记，20ms窗口内全部成为`external_unknown`，程序注入未误归用户。仍需真实物理输入正样本校准时间窗，并继续Secure Text与停止样本；负样本通过不构成路线验收。

真实物理正样本三轮分别使用20ms与100ms关联窗：首轮3个CGEvent/1个IOHID标记但0关联，后两轮仍未稳定获得可关联硬件标记。IOHID时间关联无法在当前交付环境稳定证明用户来源，按淘汰门槛失败。纯CGEvent与IOHID关联两条候选均不得接产品；本AD继续Proposed，需重新定义可验证来源契约后才能开展隐私/持久化实现。

## 2026-09-17 用户控制租约语义定稿

用户确认采用“用户控制租约”。`user`不再由不可靠的OS事件来源字段推断，而由Yonder控制平面的互斥边界定义：只有打包Task Space窗口调用专用本地`user_takeover`命令，宿主生成本次意图身份并在当前任务sequence上接受；Agent、CLI、Gateway及通用JSON查询不能声明该意图。takeover停止与定位完成后才可建立单设备`user_control`租约。

租约存在期间Admission必须拒绝Yonder的Agent CUA与Replay；因此RC只在该边界内接收允许事件并标记`user`。普通输入触发的自动暂停、点击桌宠、切屏、Agent发起takeover和历史控制重放都不建立租约。停止记录、交回、退出、权限撤销或不可恢复gap关闭租约。外部系统级注入无法由macOS稳定识别，后续规格必须明确归为`external_unknown`或用户控制范围内行为，不能伪造物理来源。

首个产品增量只收紧显式意图入口：专用Tauri命令生成takeover请求，通用`task_query`拒绝takeover；不建立Recording、不新增Schema。Recording租约及不可变时间线仍需后续Proposal。

## 验收映射

- REC-01：默认关闭且只有可信本地用户显式开始/停止。
- REC-02：单设备单 Recording，与当前 takeover 身份绑定。
- REC-03：只接受 `user`，拒绝 `agent_cua/replay`。
- REC-04：密码、安全界面和排除应用不采集，缺口可观察。
- REC-05：原始时间线不可变、有界且停止后无新事件。
- REC-06：记录状态由 Application 事实驱动，桌宠/任务菜单只展示。
- REC-07：交回前保存证据并另做新鲜 Observe；本决策不授权自动恢复。
