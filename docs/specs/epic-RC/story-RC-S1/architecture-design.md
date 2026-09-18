# RC-S1 架构设计

## 边界与依赖

原始时间线不可变，不重录 agent_cua/replay；每设备单录制，依赖 CU-S2、CX-S1、ST-S1。

## 状态与契约

SQLCipher 保持任务当前事实源；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

## 失败与验证

产品轨迹协议和回放确认归RC-S2，不得默认开启录制。首个技术路线按 Proposed AD-RC-01 进入无正文原生采集 Spike，不修改产品状态或数据库。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本文件为规划迁移，运行时无变化；具体实施按关联 ADR 和 Proposal 声明影响，draft/design-review 不授权实施。

## 人工接管记录与交回（2026-09-14用户变更）

来源：本次用户明确执行中支持人工接管并记录用户行为，作为交回Agent的Observe依据；架构Recording与桌宠/任务恢复约束，Accepted AD-TM-03。仅Agent创建任务，人工接管不创建新任务。

验收映射：TAKE-01显式接管阻止Agent新动作，停止未确认不称已移交；TAKE-02本次手动接管开启可见、可停止的记录，只录user输入，密码/安全界面/排除应用不采；TAKE-03原始时间线不可变并关联原task_id；TAKE-04交回保存证据并执行新鲜Observe，向归属Agent交付轨迹/证据引用及当前状态；TAKE-05交接失败/配额缺口明确反馈、保持暂停，不自动续跑；TAKE-06Agent重新Observe并显式恢复，重新准入后才executing。

TM-S3管接管及停止确认，RC-S1管用户记录/不可变证据，TM-S5管任务时间线引用，TM-S4管交回及显式恢复，AG管归属Agent交接协议，DS-S2展示已有任务接管/记录中/交回。自动输入干预只暂停，不无提示开启Recording；显式接管作为用户手动开始。已有每设备Recording时明确冲突，不覆盖。

UI流程：任务面板“接管”→“正在停止Agent控制/记录中”→停止确认后人工控制；小龙按既有暂停表现并保留独立录制提示。用户可以停止记录，任务仍保持人工接管；交回时如果停止后有未记录操作标注证据缺口，最终Observe仍必须新采集。“交回Agent”不等于立即继续，不用点击桌宠自动恢复任务。

协议/持久化/Driver停止与授权/隐私字段未定稿，相关Story保持设计阶段，不提前实现采集或向外发送用户记录。Windows暂缓、真实双平台证据保留。

## 2026-09-17 技术路线门禁

RC Application未来拥有Recording用例与单设备活动约束；原生Adapter只采受限事件，TM继续拥有任务/控制状态，DS只展示，AG只交付已提交证据引用。不得让React、CGEventTap回调或Node Worker成为Recording状态所有者。

首个Spike使用macOS系统listen-only Event Tap和AX/系统安全状态，不增加第三方依赖或第二个App。回调只写固定容量队列，回调外做隐私与来源分类；停止、Tap失效、权限撤销、队列溢出必须可观察。来源、隐私和停止样本全部通过后，才能另建产品Schema/协议Proposal。

MVP暂不加密使输入正文落盘仍是未决架构问题；Spike证据不得包含正文。Windows同样本按用户决定暂缓，因此AD保持Proposed。

用户已确认采用控制平面来源：只有打包Task Space专用`user_takeover`本地命令生成显式意图；takeover停止/定位完成后才允许建立单设备用户控制租约。租约期间Yonder的Agent CUA与Replay互斥，普通输入、Agent控制和通用JSON查询不能开启。首个增量只收紧入口，不启动Recording。
