# AD-TM-04 未开始任务取消

状态：Accepted（2026-09-28扩展为全部非终态任务直接取消）；日期：2026-09-14，修订：2026-09-28。Architecture Impact：architecture-change（协议控制入口与运行态取消语义）；关联TM-S3、AG-S1/S2、DS-S2、AD-TM-08。

来源：产品简报Task Space与权限模型逐任务取消、架构任务生命周期、用户允许操作已有任务。2026-09-28用户明确“所有未开始执行的排队任务都可以直接取消”，并进一步决定进行中的任务取消目前不关心副作用回收，直接停止任务并变更状态。该决定覆盖本文件首批仅created及AD-TM-08“运行取消必须先确认Driver安全停止”的限制；pause/takeover的安全边界要求不变。

Rust协议1.2的task.cancel线格式不变。可信LocalUser可取消任意非终态任务，Agent仅可取消所属非终态任务，越权与不存在同为-32004。created、尚无attempt的running排队任务以及已有attempt的running/paused/waiting_for_user/interrupted均直接以CAS进入cancelled；completed/failed保持终态并拒绝变更。请求ID仍JSON-RPC id，不产生第二套模型。

Application校验身份、归属与expected_sequence，通过原有CAS状态/事件/Outbox同事务提交Cancel。已cancelled仅expected_sequence等于当前或当前减1时返回既有快照，不新增事件；其他旧序号-32011冲突。取消成功立即冻结后续Application派发；组合根释放Yonder持有的任务准入与会话投影，但外部副作用是否已经发生、能否回滚不再作为取消成功前置，也不自动重试或伪报回滚。所有请求仍由同一TaskHost串行边界处理：已进入同步Driver的调用先返回，随后取消落库；排队任务立即落库。并发Start与Cancel由同一CAS序号保护，创建幂等重试返回真实cancelled，不复活任务。

本机控制沿现有固定LocalUser桌面query入口使用task.cancel；Agent只走已握手Gateway。UI对全部非终态任务显示“取消任务”，不再把running分流到task.control；提交中禁用，成功刷新进行中列表，失败保留并提示刷新，不自动重试。终态在全部中保留事件和快照；取消不删除数据，不调用系统删除操作。

验证：实际SQLite身份隔离、版本拒绝、序号冲突、重复取消单事件、Outbox失败回滚、Start竞态、创建重试不复活；macOS真实按钮取消一条本地测试Agent任务，另一条保留，全部可见取消态。Windows暂停，完整暂停/接管/执行中取消仍待停止确认与双平台证据，不Archive完整TM-S3。
