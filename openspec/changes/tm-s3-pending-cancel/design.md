# AD-TM-04 未开始任务取消

状态：Accepted（首批created取消）；日期：2026-09-14。Architecture Impact：architecture-change（协议控制入口）；关联TM-S3、AG-S1/S2、DS-S2。

来源：产品简报Task Space与权限模型逐任务取消、架构任务生命周期、用户允许操作已有任务。仅取消created任务，不派发动作、不释放执行租约，不启动Recording。TM-S2/CU停止前置仍约束所有执行过的任务；首批未开始分支不依赖Driver停止技术路线。

Rust协议1.2的task.cancel线格式不变。可信LocalUser可取消任意非终态任务，Agent仅所属；created、running、paused、waiting_for_user、interrupted统一CAS进入cancelled。

Application通过原有CAS状态/事件/Outbox同事务提交Cancel。已cancelled保持幂等，completed/failed拒绝。TaskHost串行化当前调用，成功后释放本机准入与执行投影；外部副作用回滚不作为取消前置。

本机控制和Agent Gateway均使用task.cancel。UI对全部非终态任务走同一取消入口，提交中禁用，成功刷新，失败可见且不自动重试。

验证：实际SQLite身份隔离、版本拒绝、序号冲突、重复取消单事件、Outbox失败回滚、Start竞态、创建重试不复活；macOS真实按钮取消一条本地测试Agent任务，另一条保留，全部可见取消态。Windows暂停，完整暂停/接管/执行中取消仍待停止确认与双平台证据，不Archive完整TM-S3。
