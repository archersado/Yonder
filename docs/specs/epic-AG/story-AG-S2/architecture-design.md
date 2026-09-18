# AG-S2 架构设计

依据AD-AG-01入口边界、AD-OCT-05握手、AD-ST-01MVP存储、AD-TM-01元数据联合契约（技术字段仍Proposed）。该Story会新增协议/持久化，须先完成相应Architecture Decision再生成Proposal。

Agent→已认证连接→GatewaySession→Application创建→TaskStore事务。UI不是创建调用方；Adapter不互调；本地/云端复用同一用例。可信AuthContext绑定归属，不使用LocalUser绕过Agent限制。Rust协议是唯一字段模型来源，schema/TS由其派生。

创建使用有界输入、deadline、request_id、capability与idempotency_key；幂等范围按Agent隔离，内容不匹配拒绝。状态/事件/Outbox以及幂等结果的原子边界、任务ID发放/迁移尚待TM-S1联审，不能仅调用现有create(id)假装满足幂等。

created只表示受理记录；TM-S2执行准入、开始唤醒、RestPermit协调完成前不派发。副作用未知不自动重试。认证/字段/幂等门禁未明确时不开传输入口。验证覆盖真实双Agent创建/重复/冲突/回滚/撤权/断连与双平台原生证据；Windows暂缓不变。

## 边界与依赖

Gateway身份/握手归Application会话，TaskHost持唯一数据库；依赖方向不变。

## 状态与契约

协议来自Rust；查询复用既有契约，创建字段/幂等迁移在AG-S2定稿后实施。

## 失败与验证

认证、握手、归属、过期、不可用必须明确失败；真实文件测试与原生连接证据分别保存。

## 人工接管记录与交回（2026-09-14用户变更）

来源：本次用户明确执行中支持人工接管并记录用户行为，作为交回Agent的Observe依据；架构Recording与桌宠/任务恢复约束，Accepted AD-TM-03。仅Agent创建任务，人工接管不创建新任务。

验收映射：TAKE-01显式接管阻止Agent新动作，停止未确认不称已移交；TAKE-02本次手动接管开启可见、可停止的记录，只录user输入，密码/安全界面/排除应用不采；TAKE-03原始时间线不可变并关联原task_id；TAKE-04交回保存证据并执行新鲜Observe，向归属Agent交付轨迹/证据引用及当前状态；TAKE-05交接失败/配额缺口明确反馈、保持暂停，不自动续跑；TAKE-06Agent重新Observe并显式恢复，重新准入后才executing。

TM-S3管接管及停止确认，RC-S1管用户记录/不可变证据，TM-S5管任务时间线引用，TM-S4管交回及显式恢复，AG管归属Agent交接协议，DS-S2展示已有任务接管/记录中/交回。自动输入干预只暂停，不无提示开启Recording；显式接管作为用户手动开始。已有每设备Recording时明确冲突，不覆盖。

UI流程：任务面板“接管”→“正在停止Agent控制/记录中”→停止确认后人工控制；小龙按既有暂停表现并保留独立录制提示。用户可以停止记录，任务仍保持人工接管；交回时如果停止后有未记录操作标注证据缺口，最终Observe仍必须新采集。“交回Agent”不等于立即继续，不用点击桌宠自动恢复任务。

协议/持久化/Driver停止与授权/隐私字段未定稿，相关Story保持设计阶段，不提前实现采集或向外发送用户记录。Windows暂缓、真实双平台证据保留。

## 首批创建权限门禁

依据AD-AG-01，仅Application create用例的AuthContext::Agent允许进入TaskStore创建；LocalUser在存储调用前返回PermissionDenied，协议错误沿用-32003身份/权限拒绝，不新增RPC错误字段。该防线不实现Agent认证或task.create传输方法；调用方不得从请求构造Agent身份。真实SQLite测试证明本机用户创建不落任务；合法Agent内部创建仍正常。Architecture Impact：conforming，无迁移/新依赖。其余创建协议/幂等/传输仍需后续Architecture Decision，不能以本子范围完成代替完整AG-S2。

## 本地任务登记设计定稿（AD-AG-02）

首批创建协议/幂等字段按Accepted AD-AG-02定稿，先做独立本地stdio Agent联调核心。task.create只登记说明和created任务，不执行说明、不接受外部自报running；真实动作执行能力及生产认证IPC不属于首批。

协议1.1、task.create参数与返回字段、schema2→3原子幂等迁移、输入限额与冲突-32009按AD-AG-02；重复返回当前真实快照。同一Agent同key同内容重试不增任务/事件/Outbox，不同内容拒绝，两个Agent key隔离；LocalUser、未握手、1.0、伪造身份、过期均在写库前拒绝。不得把任务登记当执行动画验证。

本地测试Agent通过私有继承stdio管道发hello/create/list/get，测试宿主身份预绑定，SQLite使用独立目录。原生当前桌面进程/生产连接认证未接，不冒充已接通；现有任务面板/小龙仍只消费正式宿主真实状态。核心联调没有人工任务创建按钮。

## Agent命名与接管工作定位（2026-09-14用户变更）

当前产品边界见AD-TM-07：名称来自Agent，可信归属由Gateway绑定，Application校验并与状态/事件/Outbox/幂等同事务保存；Rust生成Schema/TS，不在UI或Adapter另造名称。名称作为独立元数据，不从说明自动生成。创建协议版本、旧客户端/旧库兼容、字数上限与名称幂等比较技术细节待定案，再生成OpenSpec实施。

## 名称子范围设计审阅通过（2026-09-14）

依据用户“任务名称根据对应任务让Agent创建”及Accepted AD-TM-07名称技术定稿；承接前节来源与NAME验收映射。名称字段、1.3兼容、schema5备份迁移与幂等行为按该AD实施，不扩大到尚未定稿的接管目标。菜单/详情显示Agent名称，历史记录显示“未命名历史任务”并保留任务ID；标题支持完整无障碍文本，不引入创建/改名入口。协议错误-32602、版本错误-32010、幂等冲突-32009可观察，任何错误不写任务/事件/Outbox。名称子范围ready，完整Story既有门禁不变。
