# DS-S1 透明桌宠与桌面基础验证

Story: DS-S1
Epic: DS
Status: design-review
OpenSpec: e0-validate-desktop-foundation

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

2026-09-16：EXEC-PRESENT-01 macOS增量PASS。桌宠仅启动读取一次任务快照，随后由宿主`yonda-presentation`事件驱动；真实CUA调用尚未返回时已进入`executing`挥爪状态，调用结束后恢复SQLite/Admission派生展示。证据见[独立 Verification Goal](../../../../openspec/changes/ds-s2-task-overview/verification-event-driven-executing-macos.md)。Windows继续暂缓。

## 当前状态与前置条件

[完整验收审计](../../../../openspec/changes/e0-validate-desktop-foundation/verification-audit-20260913.md)汇总全部场景、现有证据、暂停项及正式接线的循环门禁；不按历史更新条目逐个推断总体验收状态。

补齐桌面基础栈与跨桌面证据；正式任务忙碌事件尚未接线。

2026-09-15：待命、执行中、等待用户、暂停四个已有真实来源状态的正式素材接线与 macOS 播放子目标完成；等待用户 v10 抬爪、呼吸、眨眼及录制预览的 `REC`/相机闪光验证通过。收到请求、等待外部响应、成功、失败、手动录制仍须等待所属模块提供真实事件来源，不以预览模拟接入产品。

## OpenSpec 与验证

openspec/changes/e0-validate-desktop-foundation/

[Change](../../../../openspec/changes/e0-validate-desktop-foundation/proposal.md)；独立验证在该 Change 内维护，记录存在不代表通过。

[原 Story 正文与历史验证](legacy-record.md)。旧编号仅作追溯，不用于新 PR。

2026-09-12 [真实原生菜单与窗口验证](../../../../openspec/changes/e0-validate-desktop-foundation/verification-tray-macos-20260912.md)：菜单项唤醒尺寸恢复、主进程退出和重启已验证；唤醒截图裁切尚待定位，继续 design-review，不解除基础栈门禁。

用户准备桌面后的续验：可见且激活状态下，原生菜单唤醒后 1 秒/3 秒截图均完整 200×200；休眠瞬间曾出现 onscreen=false，跨 Space 与无干预闭环仍未通过。详见上述独立验证追加记录。

用户随后确认右侧休眠时能看到眼睛；该轮 onscreen=false 与目视不一致，不据此认定窗口消失。原生标记异常与跨 Space 验证仍单独保留。

屏幕区域对照续验：不额外激活应用，原生托盘唤醒后 1 秒/3 秒屏幕区域与窗口截图均显示完整小龙。验证工具已停止把单点 onscreen=false 作为显示失败，跨 Space 和其他基础栈门禁仍保留。

[轻量插件与全屏辅助显示验证](../../../../openspec/changes/e0-validate-desktop-foundation/verification-fullscreen-overlay-macos.md)：新版 macOS 清醒形态在其他应用全屏时保持原位置可见，测试应用保持焦点；Yonda 不提供全屏主窗口。其余平台/生命周期门禁仍待完成。

[休眠全屏与返回唤醒验证](../../../../openspec/changes/e0-validate-desktop-foundation/verification-rest-fullscreen-macos.md)：本机右侧眼睛在其他应用全屏时可见，返回后托盘恢复完整小龙。普通桌面切换尝试未观察到成功通知，不计通过。

[普通桌面往返验证](../../../../openspec/changes/e0-validate-desktop-foundation/verification-normal-spaces-macos.md)：本机右侧休眠眼睛在两个普通桌面往返可见，窗口编号/位置/尺寸不变；临时桌面已清理并恢复原应用。尚未覆盖清醒普通桌面、Windows 等全部门禁。

[全应用资源采样](../../../../openspec/changes/e0-validate-desktop-foundation/verification-app-budget-macos.md)：2026-09-13 同系统资源组的宿主与三个 WebKit 进程完成五分钟自然闲置采样，CPU 0.012%，footprint 总和均值 108 MiB、峰值 138 MiB。未覆盖持续展开动画、系统共享渲染进程或 Windows，不能关闭整体性能门禁。

[持续展开资源验证](../../../../openspec/changes/e0-validate-desktop-foundation/verification-animated-budget-macos.md)：CPU 0.175%；四进程 footprint 总和均值164 MiB、峰值183 MiB，内存预算未通过，需返回 Spike 性能修复。不能用此前自然闲置通过替代持续动画验收。

[运行时素材优化验证](../../../../openspec/changes/e0-validate-desktop-foundation/verification-runtime-assets-macos.md)：保留原图并切换小尺寸副本，macOS五分钟持续展开 footprint 求和均值117.22 MiB、峰值122.85 MiB，CPU 0.155%。本轮读数达标，原生截图及页面动画检查通过；Windows和完整生命周期门禁仍保留。

[新素材原生生命周期验证](../../../../openspec/changes/e0-validate-desktop-foundation/verification-runtime-lifecycle-macos.md)：本机右侧自然收起、双眼可见、托盘唤醒后原位置完整恢复通过；未将观察器等待时长误作精确三分钟计时证据，其余边缘/点击唤醒/Windows门禁仍保留。

[启动与托盘恢复延迟](../../../../openspec/changes/e0-validate-desktop-foundation/verification-latency-macos.md)：单次新进程至渲染报告1.990秒，托盘至窗口几何恢复51.858ms；含测量边界，不关闭冷启动/完整可用/状态更新及双平台门禁。

[右侧眼睛点击验证](../../../../openspec/changes/e0-validate-desktop-foundation/verification-eye-click-macos.md)：未通过，保留点击未恢复、原生坐标超屏和发送前停止记录；需在稳定坐标下继续定位事件分发与未激活首击语义。Windows按2026-09-13用户要求暂缓，门禁保留。

[IPC拒绝路径](../../../../openspec/changes/e0-validate-desktop-foundation/verification-ipc-rejection-macos.md)：独立Spike错误版本/无效JSON拒绝及异常退出端点清理通过；宿主接线及跨用户权限拒绝仍待验证。

2026-09-13 用户手动确认当前眼睛点击可以唤醒；该人工验收追加至右侧眼睛点击验证，停止根据自动化输入失败推定产品缺陷。

[托盘退出与进程清理](../../../../openspec/changes/e0-validate-desktop-foundation/verification-exit-coalition-macos.md)：正常.app启动后，托盘退出使宿主和三个同资源组WebKit进程全部结束，已重新启动。直接包内可执行文件启动与正常.app启动的资源归属差异单独记录。

[三分钟闲置计时](../../../../openspec/changes/e0-validate-desktop-foundation/verification-idle-timing-macos.md)：托盘明确重置后180.23秒仍展开、180.50秒几何收起，本机单次无任务计时通过；忙碌重置等门禁仍保留。

[同进程宿主IPC验证](../../../../openspec/changes/e0-validate-desktop-foundation/verification-host-ipc-macos.md)：macOS桌宠实际持有UDS，错误拒绝后CLI可继续往返，托盘退出清理Socket和同组进程，重启可重建。没有正式任务/Gateway接线；新版资源预算待复测。

[IPC版本资源复测](../../../../openspec/changes/e0-validate-desktop-foundation/verification-host-ipc-budget-macos.md)：五分钟读数CPU0.1695%、内存均值122.24MiB、峰值135.55MiB，低于预算；同期视觉证据缺失，保留范围限制。

[无障碍语义激活](../../../../openspec/changes/e0-validate-desktop-foundation/verification-accessibility-macos.md)：补齐detail=0 click入口，页面合约与macOS未激活AXPress唤醒通过；当前PID9322，Windows及实体键盘门禁仍保留。

[原生键盘唤醒](../../../../openspec/changes/e0-validate-desktop-foundation/verification-keyboard-macos.md)：已聚焦双眼按钮的Enter/空格路径通过，采用定向PID原生事件且核对实际恢复；未扩大为键盘导航或Windows完成。

2026-09-14：有任务才悬停、任务状态表现与同桌面面板交互已接线；独立验证见ds-s2-task-overview/verification-task-state-hover-macos.md。空库及原生面板操作PASS，真实运行任务全链路/Windows/完整姿态仍待验证，不Done。
