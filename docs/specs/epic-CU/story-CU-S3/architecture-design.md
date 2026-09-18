# CU-S3 架构设计

## 边界与依赖

执行请求必须显式声明一种已注册能力：后台原生动作、CUA动作或Command。Yonder只做能力校验、准入、执行编排和结果提交，不根据自然语言或运行失败选择另一条路线。

- 后台原生动作：调用目标应用公开且经验证的系统API、应用SDK，或Yonder可编译链接的具体App Intent类型；不进入trycua会话，不取得`Resource::Desktop`。
- CUA动作：AX/UI及键鼠统一经Accepted AD-CU-05的受监管trycua链路；连续CUA会话中不得插入原生执行器。
- Command：保持AD-CM-01的结构化进程语义，不作为原生动作兜底。

原生动作不因绕过CUA Driver而绕过Application。它仍使用现有任务、步骤、attempt、Admission和事务提交路径；当前状态、同sequence事件与Outbox同事务写入。Task Space只读取该统一事件链。

## 状态与契约

SQLite保持任务当前状态唯一事实源。后台原生动作、CUA和Command共用既有任务生命周期；执行类别只描述已声明的执行路线，不允许UI或Adapter据此改写任务状态。协议类型仍从Rust单一来源派生。

## Task Space 可观测性

每个后台原生动作至少产生可排序的步骤声明、开始和结果事实，并关联动作后Observe。时间线展示稳定的能力名称、`background-native`执行类别、阶段、结果分类及有界证据摘要。正文、截图、完整Payload和底层原生对象不进入SQLite事件、Outbox或日志。

桌宠的“后台执行”图标是同进程事件驱动投影；SQLite任务状态仍是事实源。窗口初始化允许读取一次快照，之后不得轮询数据库驱动动画。

## 前台切换与失败语义

后台动作发现必须操作界面时返回稳定的`unsupported`或`foreground-required`结果，不自行启动CUA。Agent Observe后可显式声明下一CUA步骤；该步骤重新执行资源准入并遵守用户输入暂停。副作用动作超时、断连或Observe失败为unknown，不自动重试或改道。

系统权限是后台准入前置条件。`not-determined`、`denied`或`restricted`不得在后台动作内部调用授权界面；Application提交`permission-required`事实并转为等待用户。用户通过显式权限入口完成系统交互后，Agent重新Observe权限并提交新动作。权限界面不是CUA步骤，也不能显示为后台忙碌。

## 失败与验证

原生能力不可用、权限拒绝、目标不存在、超时、断连和Observe失败必须稳定分类；不能证明副作用结果时保持unknown。验证覆盖后台成功、前台不变、不支持无兜底及与CUA对照，单平台证据不外推完整Story。

## 限时Spike

2026-09-18至2026-09-20在macOS隔离fixture验证：后台打开/调用不改变前台应用；一个公开原生动作产生可核验结果；一个AX动作仍经过trycua并Observe；不支持场景不隐式改道。Spike只输出布尔、时间和安全枚举，不记录内容。

本机AppIntents公开SDK面只允许具体`AppIntent`类型执行或donation，没有按外部应用metadata标识调用的公共入口；计算器虽发布`Metadata.appintents`，Yonder不能据此获得可链接类型。通用App Intent桥接路线淘汰；`shortcuts run`属于Command，不进入原生Adapter。Windows按用户决定暂缓，不能据此接受跨平台路线。

EventKit真实副作用样本确认：首次提醒事项授权会改变前台，不能归入后台动作；已授权后，临时提醒创建、按identifier Observe和删除清理可在前台不变的情况下完成。证据不保存用户提醒正文、测试标题、列表名或对象标识。

## 架构影响

关联[AD-CU-06](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-06-BACKGROUND-NATIVE-ACTION-ROUTING.md)为Proposed。Spike不新增产品Port、协议字段、数据库迁移、依赖或常驻Worker；至少一个具体真实能力通过且双平台路线明确后，再决定最小产品接线。
