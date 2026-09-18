# SDK隔离AX执行中排空独立 Verification Goal

当前状态：FAILED（未取得原生开始/唯一可编辑目标证据），未通过、不Archive。AD-CU-02定稿MVP步骤边界控制后，该合成目标排空样本不再作为额外产品前置；下述verifying为历史。没有实测证明通用执行中原生排空或强制中断。

2026-09-14；状态verifying。实现后建立，关联CU-S1 STOP-DRAIN01–03、AD-CU-01/AD-E0-02及TAKEOVER-STOP-PLAN。原生输入框实际AX setter收到固定标记后报告开始并延迟2秒，SDK自管子进程输入尚未返回时宿主请求shutdown，核实关闭等待应用及新SDK Observe最终状态。

样本须有真实setter开始，shutdown请求早于应用、关闭确认不早于应用、Worker正常退出、输入结果成功、Observe/目标匹配。缺任一证据不通过并回实施，不宣称执行中排空。结果仅布尔、时间/安全枚举，无原文/截图/Recording。时间为同一主机短样本墙钟证据，不作为产品超时阈值或协议。只验证后台AX graceful drain，不覆盖强制中断、其他输入能力、正式宿主权限或Windows。

首轮未取得原生setter开始，结果失败保留evidence/sdk-drain-macos-20260914/result.json，不追认排空通过。返回实施：将观察入口从NSAccessibility属性设置方法改为NSTextField实际stringValue setter，且失败时保留安全布尔/枚举诊断。修订后重新构建并验证，结果待核实。

第二轮evidence/sdk-drain-macos-setter-20260914/result.json在子进程就绪前超时，input_dispatched=false，未取得原生开始。再次回实施：子进程初始化沿已通过基线metadata/list_windows/get_window_state顺序；补齐目标失败/退出布尔诊断并放宽样本等待至30秒、测试窗总生命周期至90秒，均为测试上限，不是产品性能门禁。新版构建后独立重跑，不改写失败记录。

第三轮evidence/sdk-drain-macos-v3-20260914/result.json证明输入成功、route=accessibility、effect=confirmed且目标匹配，但内置控件绕过公开setter，native_write_started=false，样本仍失败。返回实施：延迟模式改为显式实现原生NSAccessibility写接口的控件，普通基线不变；构建后重新验证，旧失败全部保留。

显式AX控件首轮evidence/sdk-drain-macos-explicit-ax-20260914/result.json未取得唯一可编辑token，input_dispatched=false。回实施，在控件初始化设置原生Accessibility标志/role/identifier，并记录仅角色/启用/token存在布尔诊断，不包含AX正文或名称。修订后独立重跑；控件尚未暴露不能解释为SDK排空失败或通过。

最终evidence/sdk-drain-macos-ax-flags-20260914/result.json仍ax_roles为空、未派发输入，样本未成立。停止该合成样本研发，原生写入基线已有route=accessibility/effect=confirmed证据（sdk-drain-macos-v3），无需把测试控件调试当作用户功能。真正步骤边界停止/竞态/Observe及未知占用须在CU-S2/TM-S3独立验证。
