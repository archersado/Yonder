# 独立Verification Goal：接管工作窗口定位

状态：PASS（macOS隔离原生基线）；环境2026-09-14 macOS，Windows暂缓。来源TM-S3 FOCUS-01～04、CU-S2与AD-TM-07/AD-CU-02；前置SDK-only由AD-CU-01约束。本Goal是技术路线Spike，不是产品接管已完成。

安装SDK catalog确认仅list_windows/get_window_state/set_window_frame，无精确前置/恢复最小化接口。使用临时原生AX/AppKit可执行夹具与定位探针，仅验证夹具PID/WindowServer ID；不打包额外App、不采集用户输入、不截屏、不操作用户工作。

断言：同名诱饵在前时准确前置目标，最小化恢复目标、原位置不变、原生key/activeSpace与SDK无截图Observe成立；关闭原目标后拒绝并保持同名诱饵不受定位动作影响，SDKObserve不得把失效目标当作有效AX目标。

附加Space/显示器、进程启动身份复用、用户真实工作窗口与正式Yonder责任链尚未验证，不以普通桌面通过外推。失败返回Spike实施，保存固定错误/布尔结果。停止确认、任务状态/租约、Recording与对外协议均不属本Goal。

## 实测结果

native-readiness.json全部断言通过：正常focus/geometry/SDKObserve，原生最小化确认，恢复focus/geometry/SDKObserve，关闭原目标后定位返回拒绝（AX仅剩同名诱饵但几何不匹配），诱饵不受定位动作影响，SDK返回退化空树且本次AX目标不可用。input_dispatched/Recording/screenshot请求均false，fixture自动退出，不操作真实任务。

SDK不存在精确focus工具的结论来自安装契约catalog.json；采用Accepted AD-CU-03原生路线。失败诊断native-result/native-diagnostic/native-validity/native-degraded/native-observed/native-ready/native-mapping/native-launched/native-ax-focus/native-ax-errors/native-closed-state保留。早期断言错误地要求关闭后SDK必须isError，实测退化空树应拒作有效AX目标；早期把夹具未激活/动画阶段当作窗口未就绪，已区分启动、可见、AX可观察和实际焦点。

证据目录：spikes/cua-driver-comparison/evidence/focus-macos-20260914/。可运行检查：编译input-fixture-macos.swift与focus-target-macos.swift，然后运行focus-macos-probe.mjs --native <fixture> <helper> <result>。Node语法与Swift编译通过，架构关联检查不代替原生目标验证。

限制：没有生产PID启动身份/当前WorkRef绑定、多Space/显示器、正式Yonder权限责任链或Windows新验证。完整任务接管/停止确认/Recording与完整Story不Done/Archive。
