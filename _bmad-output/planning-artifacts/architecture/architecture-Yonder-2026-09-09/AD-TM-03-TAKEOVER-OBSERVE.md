# AD-TM-03 人工接管记录与交回Observe

状态：Accepted（用户接管/记录/交回产品边界）；控制确认、Recording协议及持久化字段仍待TM/RC/CU/AG联合设计，不据此直接实施Driver或采集。关联TM-S3/S4/S5、RC-S1、AG-S2、DS-S2。Architecture Impact：architecture-change（人工控制与证据交接边界）。

来源：2026-09-14用户明确执行中支持人工接管，接管过程记录用户行为，作为之后交回Agent的Observe依据。与AD-AG-01只允许Agent创建任务并行成立：接管是操作已有任务，不是人工创建。

用户显式点击“接管”作为本次手动记录开始，界面先显示记录范围/进行中提示与停止入口；默认Recording仍关闭。普通用户输入引发自动暂停不自动启动记录，必须有显式接管。每设备仍最多一个Recording，已有手动记录占用时明确冲突，不覆盖旧记录或自动启动第二个。

接管请求立即阻止新增Agent动作，并向执行Driver请求停止，停止未确认时显示接管处理中，不称已安全移交；未知动作保留unknown/占用，不自动重试。采集只记录明确接管时间边界内user来源；agent_cua/replay始终不派生用户轨迹。Recording原始时间线不可变，绑定task_id与交接边界；TM拥有控制/任务状态，RC拥有Recording证据，DS只展示。隐私/密码/排除应用永不采集。

交回流程：用户显式交回→关闭本次记录边界并保存→采集新鲜且经过隐私过滤的当前Observe→把Observe及用户轨迹/证据引用经Gateway交给归属Agent。轨迹是发生事实，不能代替新鲜当前Observe；因配额/权限导致缺口必须标注，不伪称完整。交接失败任务保持暂停/待处理，不丢未同步证据或自动继续。Agent重新Observe并显式决定后续；真实重新准入前不恢复executing，Yonder不做语义replan。

元数据、输入范围/脱敏、证据协议、日志/持久化字段、用户记录停止后的任务状态、撤权与幂等交回需联合定稿，再进入OpenSpec。MVP未加密不放宽隐私或正文日志禁令。

2026-09-14停止前置复核：CU-S1原生macOS只读生命周期Goal已通过，但native_input_stop_verified=false。TM/CU联合设计必须明确停止确认绑定当前执行attempt及Worker实例，先关闭新动作准入，再等待当前原生动作与Worker停止的可信证据；预取消Promise拒绝、SDK shutdown或Node退出不可替代该证据。停止超时/断连保留unknown与租约，不启动接管Recording、不宣称已移交。RC只有在停止确认、隐私过滤、user/agent_cua/replay来源判定及显式记录提示具备后才能开始；这些协议和字段仍未定案。该复核不授权新的执行或采集实现。
