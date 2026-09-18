# AD-CU-03 接管工作定位的原生窗口控制

状态：Accepted（macOS生产 WorkRef Adapter；TM定位事务仍未授权）。日期2026-09-14，2026-09-16增量接受；Architecture Impact：architecture-change（CU原生窗口定位路线明确）；关联TM-S3、CU-S2、DS-S2、AD-TM-07/AD-CU-01/02。

## 来源

原产品简报Task Space逐任务状态和权限模型、架构每步Observe/单桌面租约/任务当前事实源；用户要求接管后前置正在操作的工作并切到工作所属屏幕，且只复用SDK、不附带上游App。名称子范围已独立通过，不替代此门禁。

## 路线决定

安装trycua0.25 SDK工具契约仅含窗口查询和frame调整，无精确窗口前置/恢复最小化。禁止用set_window_frame或移动到小龙屏幕冒充定位。动作执行仍复用SDK；CU同一Adapter内部用系统原生AX/CoreGraphics能力完成工作前置，不新建App、不调用AppleScript、不复制Browser Task Space、不引入第二CUA执行栈。临时Swift探针仅为技术验证，不作为产品Shell/CLI依赖打包。

macOS隔离基线：验证WindowServer窗口ID及PID所属和存活；按目标标题/几何与AX窗口唯一映射，恢复AXMinimized、设目标AXMain、AXRaise、设该进程AXFrontmost。完成必须读取新鲜AXFrontmost/AXFocusedWindow/最小化状态，并独立由原生夹具确认key/activeSpace；请求返回成功不等于实际完成，不依赖缓存isActive判断。原位置保持，不能改窗口frame换屏。

WindowServer可见不等于AX就绪：准入前可有界重新Observe/只读匹配，不重发副作用；映射不唯一、目标无可用AX、权限拒绝、超时、焦点失败保持任务暂停与明确定位失败，不自动改选同名窗口。关闭对象可能保留WindowServer ID，但只剩诱饵AX窗口；SDK会返回退化空树而非isError，不能当作本次AX目标有效。

## 产品实施尚需定稿

生产WorkRef必须由当前执行/Observe产生并绑定task/attempt/Worker与进程启动身份，WindowServer与AX两侧唯一映射及当前目标指纹均复核，不能只信Agent的PID/名称或仅按标题/几何长期缓存。原生副作用前再次复核，PID复用、窗口替换、不同Space/显示器和工作关闭均须覆盖。WorkRef字段、控制事务/事件、Port签名仍待TM/CU/AG联合技术设计，本文不授权绕过。

AD-CU-02停止确认必须在前；CU返回定位事实，TM持有任务/控制状态，UI只消费快照，Adapter不互调。定位通过也不自动Recording，RC显式开始/来源/隐私条件另验。正式宿主权限责任链与Windows、多显示器/Space证据尚未通过，完整Story不Done/Archive。

## 证据

spikes/cua-driver-comparison/evidence/focus-macos-20260914/catalog.json与native-readiness.json；独立Goal openspec/changes/e0-compare-cua-drivers/verification-focus-macos.md。正常前置/最小化恢复/几何保留/动作后SDKObserve/关闭后原生拒绝与同名诱饵不受影响全部通过。早期失败诊断原样保留；修正夹具AppKit启动完成时序、AX就绪判断、新鲜焦点观察及关闭后退化空树断言，不解释为SDK强制停止或所有目标路线通过。

## 2026-09-14 工作身份子范围证据

macOS隔离复核采用proc_pidinfo启动秒/微秒、保留AX对象并核对当前AX列表成员、WindowServer新鲜几何/标题双侧唯一映射；原生正常/最小化、关闭、同名同frame替换与进程重启拒绝全部通过。映射不唯一与同PID不同启动时间/权限拒绝的合约负样本通过，后两项不是系统权限切换或真实PID复用实测。只读AX就绪/几何匹配可有界等待，不重发副作用、不改选同名对象。

独立Goal：openspec/changes/e0-compare-cua-drivers/verification-work-identity-macos.md；证据spikes/cua-driver-comparison/evidence/work-identity-macos-20260914/fresh-mapping/result.json。限于原生身份路线隔离子范围，不接受整个AD-TM-08字段/Port/事务，也不外推正式宿主、多Space/显示器或Windows。产品接管/Recording仍未启用。

## 2026-09-16 生产 WorkRef Adapter 定稿

CU-S2可把隔离验证过的原生逻辑作为现有Rust Adapter的内嵌实现，不启动辅助App、脚本或常驻sidecar。Application定义`WorkRef`、稳定失败分类与`WorkFocusPort`；Adapter在有效后置Observe后，用可信`WorkTarget`捕获进程启动时间、WindowServer窗口与唯一AX窗口，并在宿主内保留AX对象。`work_ref_id`由当前attempt接受序号生成，完整绑定task/step/attempt/Worker/host；Agent与UI不能提交PID、窗口ID或引用。

定位前重新核对辅助功能权限、进程启动时间、WindowServer的PID/窗口ID/标题/几何、保留AX对象仍属于当前AX窗口列表且双侧映射唯一。通过后恢复最小化、设主窗口、Raise与进程Frontmost；返回前重新读取FocusedWindow、Frontmost、最小化和几何。任一步失败均返回稳定拒绝原因，不改选同名窗口、不移动frame、不自动重试。引用只在当前宿主内存有效，释放或重启后不可操作。

本增量只授权Port、macOS Adapter和隔离原生证据，不新增外部协议、任务状态、SQLite字段或UI成功提示。TM-S3必须在takeover停止确认后另建Proposal，原子记录定位结果并驱动“正在定位/失败/已前置”；未完成前按钮仍只能停在“正在停止”。Recording继续关闭，Windows按用户要求暂缓。
