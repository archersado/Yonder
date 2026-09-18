# AD-TM-04 未开始任务取消

状态：Accepted（首批created取消）；日期：2026-09-14。Architecture Impact：architecture-change（协议控制入口）；关联TM-S3、AG-S1/S2、DS-S2。

来源：产品简报Task Space与权限模型逐任务取消、架构任务生命周期、用户允许操作已有任务。仅取消created任务，不派发动作、不释放执行租约，不启动Recording。TM-S2/CU停止前置仍约束所有执行过的任务；首批未开始分支不依赖Driver停止技术路线。

Rust协议1.2新增task.cancel，capability=task.cancel，参数agent_id/deadline/task_id/expected_sequence（沿用序号字符串）。Gateway协商minor>=2才能调用；1.0/1.1行为不变。响应复用snapshot。可信LocalUser可取消任意created任务，Agent仅所属，越权与不存在同为-32004。请求ID仍JSON-RPC id，不产生第二套模型。

Application校验身份、归属与expected_sequence，再检查created并通过原有CAS状态/事件/Outbox同事务提交Cancel。已cancelled仅expected_sequence等于当前或当前减1时返回既有快照，不新增事件；其他旧序号-32011冲突。非created/非cancelled返回-32012需要执行停止确认（包括paused/interrupted/终态），不得把取消请求当停止证据。并发Start与Cancel由同一CAS序号保护，Start提交失败不派发并释放本次准入，已有未知占用不清除。创建幂等重试返回真实cancelled，不复活任务。

本机控制沿现有固定LocalUser桌面query入口，仅增加task.cancel分支；Agent只走已握手Gateway。UI选择created详情后显示“取消任务”，提交中禁用，成功刷新进行中列表；失败保留并提示刷新，不自动重试，不新增人工创建入口。终态在全部中保留事件和快照；取消不删除数据，不调用系统删除操作。普通点击小龙仍不打开菜单。

验证：实际SQLite身份隔离、版本拒绝、序号冲突、重复取消单事件、Outbox失败回滚、Start竞态、创建重试不复活；macOS真实按钮取消一条本地测试Agent任务，另一条保留，全部可见取消态。Windows暂停，完整暂停/接管/执行中取消仍待停止确认与双平台证据，不Archive完整TM-S3。
