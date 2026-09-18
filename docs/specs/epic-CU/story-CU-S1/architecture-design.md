# CU-S1 架构设计

## 边界与依赖

当前SDK-only边界以Accepted AD-CU-01为准；上游App构件方案撤回，STOP-PKG段落仅历史。STOP-SDK01–04复用stop-macos-probe子进程，子进程仅导入SDK并metadata就绪，父进程经继承私有Node IPC监督，在就绪后SIGTERM并等待exit；另一个就绪子进程断开父通道后主动关闭SDK/退出。超时由宿主终止并判失败，不能忽略超时后称正常退出；新实例恢复沿用只读recover。Spike的Node IPC仅继承通道，不作产品协议或Agent认证入口，产品私有stdio字段仍待CU-S2定稿。

遵循 AD-E0-02，Windows 选 trycua；模型无关，不引入语义规划或第二执行栈。

## 状态与契约

SQLCipher 保持任务当前事实源；UI 仅持展示快照。传输类型从 Rust 派生。改变协议/持久化/边界前先补 ADR，不为本 Story 另建状态系统。

## 失败与验证

STOP-DRAIN01–03沿用同一输入窗/探针：可选延迟setter实际回调报告布尔/时间，SDK自管子进程自己取得token，父进程观察开始且未完成后发shutdown；关闭结束时间与原生应用时间比较，恢复实例Observe验证固定目标。2秒延迟只为隔离样本，不做产品超时阈值。若shutdown先于原生应用返回或无法证明开始，门禁不通过、不得安全移交；即使本场景通过也不外推其他SDK能力。

STOP-IN01首先调用已安装SDK currentMacOsPermissionStatus（当前导入宿主），只输出两权限布尔及SDK版本；不调用requestMacOsPermissions或系统设置。accessibility=false时停止输入阶段，不创建测试窗口或派发动作；结果不能代替正式Yonder责任链证据。STOP-IN02–03采用隔离AX目标、固定测试内容、无截图/无剪贴板/无前台兜底，动作后Observe与输入框状态相符才通过；未取得原生准入证据不得称执行中中断通过。

Windows 选型已有证据；macOS 和 Office 延期，不改称全部完成。
失败不得隐式重试未知副作用。验证覆盖正常、拒绝和中断路径；沿用架构依赖与关联检查。

## 架构影响

本文件为规划迁移，运行时无变化；具体实施按关联 ADR 和 Proposal 声明影响，draft/design-review 不授权实施。

## macOS接管停止补充Spike

来源：原产品两条执行路径/用户接管变更、架构CUA与任务恢复、AD-E0-02/AD-TM-03；关联spikes/cua-driver-comparison/TAKEOVER-STOP-PLAN.md。期限2026-09-14至2026-09-16，统一样本沿用已有fault/crash-recovery，唯一路线trycua0.25.0，Qwen已淘汰不再运行。STOP-READ01记录预取消只读调用真实结果；STOP-READ02记录关闭/关闭后调用/重复关闭；STOP-READ03异常退出后新实例只读恢复；STOP-READ04只保留布尔/错误分类/时间，不输出正文或应用列表。均对应原Driver故障验证与隐私/恢复约束。

只读生命周期探针不触发键鼠、截图或Recording。取消Promise/Node退出不是系统输入停止证据，未知结果保留占用；没有真实动作/worker树停止证据前CU-S2/TM-S3接管按钮继续禁用。Windows新增测试暂缓，完整双平台CU验收不通过/不Archive。

### 已提交调用与关闭竞态

STOP-PKG01–03沿用TAKEOVER-STOP-PLAN，固定官方Release归档与发布SHA-256；解包前拒绝绝对路径、父级穿越、链接及设备文件，签名与平台评估只读执行。任何安全门禁失败禁止运行构件，不执行远程安装脚本，不修改系统权限、服务或产品依赖。

裸二进制系统评估拒绝后，复用同版本官方macOS目录包核验CuaDriver.app，检查App完整签名、com.trycua.driver标识及主程序架构。旧拒绝证据保留，不安装到Applications；App评估同样失败则停止该路线验证，不自行修改分发身份。

STOP-SUB01–04沿用同一只读子进程探针；调用list_apps后通过setImmediate发送Abort，明确仅观测JS提交/完成边界，不宣称原生准入或动作中断。Abort后新读请求必须成功；另一次list_apps与shutdown并发，记录读调用结果，shutdown结束后调用必须拒绝。子进程超时仍失败，所有结果仅布尔/分类。已安装官方README说明shutdown关闭准入并等待已有操作完成，因此它是排空接口；是否可用于动作停止确认仍需场景级动作后Observe及Worker证据。
