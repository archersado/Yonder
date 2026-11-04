当前归属 Story：DS-S1；规划：`docs/specs/epic-DS/story-DS-S1/README.md`。旧编号保留历史追溯。

# Verification Goal：E0-S1

当前完整完成条件与缺口以 [2026-09-13验收审计](verification-audit-20260913.md) 为入口。下文日期记录保留追溯，不能把分项通过扩大为总Goal通过。

## 目标

独立确认 E0-S1 的全部验收条件与 OpenSpec 场景在 Windows/macOS 实机成立，并审查其符合架构围栏。

## 必需证据

- 两个平台的构建日志、系统版本与硬件说明
- 窗口和托盘截图或短视频
- IPC 请求/响应及无 TCP 监听证明
- 五分钟空闲 CPU/内存、启动时间、窗口事件延迟原始记录
- 进程退出与权限验证记录
- AD-E0-01

## 判定

2026-09-11 macOS 基础能力续验见 `verification-native-macos.md`：真实 UDS 往返、无 TCP 套接字及退出清理通过；新增托盘找回/退出实现，原生菜单点击与全进程资源预算仍缺证据。

跨桌面与边缘休眠见 `verification-edge-rest.md`：用户选择 3 分钟、双眼趴边，后台任务禁止隐藏；正式任务源和原生完整闭环仍待验证。

四边增量原生续验：顶部倒挂透明画面及 12 秒内三次眨眼已通过真实 macOS 窗口录屏确认，右侧/顶部停靠和恢复几何有日志证据；不扩大为四边完整、Windows 或跨 Space 验收通过。

小龙形象增量见 `verification-dragon.md`：已构建并重启，原生视觉与双平台证据仍待补充。

基础动态交互增量见 `verification-animations.md`：ego-browser 页面与视觉回归通过；原生双平台门禁仍未通过。

当前状态：部分验证，Goal 未通过。任何目标平台缺失、以模拟环境替代或未达到门槛，均不得标记完成。

## 2026-09-13 当前证据汇总

以上日期记录保留历史语境，当前增量如下：

- macOS 清醒/右侧休眠的其他应用全屏覆盖、右侧休眠普通桌面往返已有实拍，分别见 verification-fullscreen-overlay-macos.md、verification-rest-fullscreen-macos.md、verification-normal-spaces-macos.md；不扩大为所有形态和平台通过。
- 素材优化后的四进程持续展开资源读数达到本机预算，见 verification-runtime-assets-macos.md；保留旧版本超预算记录以及采样方法限制。
- 新素材右侧自动收起、托盘恢复原位置及完整画面通过，见 verification-runtime-lifecycle-macos.md；精确180秒计时、点击双眼、其他边缘及忙碌任务实际接线仍缺完整原生证据。
- 后续优先补齐原生启动/事件延迟和 Windows 当前版本证据；正式任务接线仍受 E0/任务前置门禁约束。菜单与密钥保持暂停。

2026-09-15：四个已有真实来源状态的正式素材接线与 macOS 播放子目标已完成，见 `verification-state-playback-macos.md` 和 `verification-waiting-recording-v10.md`。其余五态只完成独立动画预览，必须等待所属任务/录制事件契约后接线；Windows继续按用户要求暂缓。因此总 Goal 保持未完成。

总 Goal 仍未通过，不 Archive。

2026-10-25：Windows 自动空闲停靠子范围修复并经独立 Goal 复核通过。两个隔离无输入样本分别在181.401秒与181.210秒进入底边112×56和左边56×112；一次真实拖动样本按交互重置规则排除。详见 `verification-windows-idle-dock-20261025.md`。顶部/右侧、恢复交互、九态完整原生链路、性能及 macOS 剩余门禁仍保留，因此总 Goal 状态不变。

2026-10-29：Windows 四边休眠的剩余原生子范围经独立 Goal 通过。原生点击后170.013秒仍展开、180.355秒顶部停靠；顶部可见眼睛点击恢复；右侧179.839秒停靠并由Enter恢复。方法用窗口句柄反查排除DPI注入误判，详见 `verification-windows-edges-wake-20261029.md`。结合2026-10-25证据，Windows四边停靠尺寸及点击/键盘恢复、交互重置已覆盖；人工拖动、九态完整原生链路、性能与macOS剩余门禁仍保留，总 Goal 状态不变。

同日 Windows 原生合成输入拖动技术链路经独立 Goal 通过：窗口实际位移168×112逻辑像素，尺寸保持200×200，释放后继续移动光标不再带走窗口。详见 `verification-windows-drag-20261029.md`。该证据不冒充用户人工拖动；Windows原生视觉、人工确认、九态完整链路、性能与macOS剩余门禁不变。

随后Windows当前素材原生视觉与透明边缘经独立Goal通过：200×200逻辑窗口在150%缩放下对应300×300物理截图，四角和61,492个透明区域像素透出受控背景，28,508个角色/UI像素可见；人工查看完整小龙未裁切且无矩形底。详见 `verification-windows-visual-20261029.md`。Windows用户亲手拖动、九态完整链路、性能/生命周期与macOS剩余门禁仍保留。

同日 Windows 真实 BUA 展示子范围经独立 Goal 通过：Gateway 1.19 创建任务后进入 listening，两次真实 `browser.execute` 调用尚未返回时均进入 executing，`ego:3` 正常 finish 后任务完成并展示完整举爪 success，约2秒后恢复 idle。修复了正式清单缺少 success/failed 导致终态回退 idle 的缺陷；仅接入完整主体和闭眼层，未恢复已否决的局部切片。详见 `verification-windows-bua-presentation-20261029.md`。本轮未执行 CUA，也未伪造 failed；等待/暂停、thinking/recording、真实失败、九态完整链路与双平台总门禁继续保留。

随后补齐 Windows 当前调试构建五分钟自然闲置性能样本：304.625秒、55点，完整后代进程树工作集平均863.84MiB、峰值917.57MiB；单核口径CPU平均21.9259%、峰值42.0%，24逻辑处理器归一平均0.9136%。详见 `verification-windows-performance-20261029.md`。因此“当前版本证据”缺口关闭，但 `<1%单核CPU / <150MB` 优化目标未达；按Accepted AD-E0-01暂不阻断功能，发布前仍须优化和复审，总Goal不变。

同日修复任务总览增量覆盖 DS-S1 基础托盘入口的回归，并通过 Windows 原生托盘独立 Goal：真实菜单保留七项且首尾为“显示小龙 / 退出 Yonda”；系统语义激活可从最小化找回当前 pet 窗口，退出码0，退出前记录的宿主及11个WebView2进程全部消失、无TCP、未强制清理。详见 `verification-windows-tray-exit-20261029.md`。菜单变化后的macOS当前版本回归、Windows其余门禁和总Goal仍保留。

2026-10-31：Windows 正式 Gateway `task.create → listening` 核心展示状态延迟子范围经独立 Goal 通过。从请求写入至 WebView `applyPresentation` 同步配置后首次白名单诊断的观测上界为15.829ms，低于300ms当前优化目标；任务随后经正式 `task.cancel` 进入 `cancelled`。详见 `verification-windows-presentation-latency-20261031.md`。启动基线因既有 `interrupted` 任务真实为 `paused`，未伪造固定 `idle`；executing、等待、暂停、终态/恢复、完整可用状态和macOS同口径仍保留，总Goal状态不变。

2026-11-01：修复 Desktop 在多版本安装中误选未激活 ego Runtime 后，Windows 真实 BUA `browser.execute(create) → executing` 状态应用延迟子范围通过，观测上界14.225ms且请求仍在途；真实 `finish` 完成并关闭 `ego:7`，任务最终 `completed`，随后依次展示success并恢复既有paused基线。详见 `verification-windows-bua-latency-20261101.md`。success计时包含外部Task Space完成、恢复计时包含终态脉冲，均未冒充300ms纯UI目标；waiting、真实pause/failed、完整可用状态与macOS同口径仍保留，总Goal不变。

启动/延迟增量见 verification-latency-macos.md：本机单次新进程到渲染报告1.990秒，托盘到原生几何恢复51.858ms；14秒原生诊断出现3次眨眼及呼吸。未将这些观测替代冷缓存首帧、完整可用状态或300ms核心事件链路验收。

2026-09-13 用户要求 Windows 先暂缓；停止安排该平台复测，仍保留双平台完成门禁。继续推进 macOS 已授权范围，不能将暂缓记为已通过。

右侧眼睛真实点击未通过，见 verification-eye-click-macos.md；原生坐标异常及事件序列尚待定位，不能用托盘恢复记录替代该门禁。

IPC拒绝路径见 verification-ipc-rejection-macos.md：本机独立Spike的错误版本/无效JSON拒绝及端点清理通过，非跨用户权限拒绝或桌宠宿主接线验证。

用户随后真实桌面手测确认“能被唤醒，我测试了”：当前眼睛点击可唤醒记录为人工确认，自动化合成点击仍失败；不把工具失败判为产品失效，不扩大为全部边缘/精确延迟已验收。

本机正常.app启动后的托盘退出及四个已归属进程清理通过，见 verification-exit-coalition-macos.md；已重新启动小龙。未扩大为Windows、未来Worker或全部Story完成。

三分钟无任务计时原生观测通过，见 verification-idle-timing-macos.md：明确托盘重置后180.23秒仍展开，180.50秒收起。跨用户Socket尝试因sudo缺授权未执行到连接，不计权限拒绝通过。

清醒普通桌面续验见verification-awake-spaces-macos.md：未取得合格样本，不改变已有休眠形态通过的范围；本轮临时新增桌面已按日志清理。

macOS同进程echo接线及原生宿主往返/退出清理通过，见verification-host-ipc-macos.md。正式任务接线不在本次范围，新增线程后的资源预算需复测。

新增IPC线程版本资源读数复测已完成，见verification-host-ipc-budget-macos.md：CPU0.1695%、均值122.24MiB、峰值135.55MiB。同期几何检查完整，截图取得时间晚于采样，未据此通过同期视觉验收。

无障碍入口缺少语义click已修复；页面回归及macOS未激活AXPress唤醒通过，见verification-accessibility-macos.md。标签/几何恢复约108ms，不代替完整出现动画时长或实体键盘验证。

原生Enter/空格唤醒随后通过，见verification-keyboard-macos.md：定向Yonda进程的真实原生按键事件，经已聚焦双眼按钮触发标签/几何恢复。未调用AXPress；键盘导航、repeat、减少动态效果及Windows仍分开处理。

减少动态效果的原生前置见verification-reduced-motion-macos.md：原始设置false，系统设置自动化入口不可用，已请求用户开启以继续验证；尚未变更设置或宣称通过。

用户开启后，减少动态效果启动采样通过：原生API前后均为true，正常.app内14秒诊断reduced=true、breathing=false、blinks=0，渲染正常。见verification-reduced-motion-macos.md与原始stderr日志；首尾呼吸采样不代表逐帧验证。运行中切换、直接停靠/唤醒与原始false恢复仍待验证，不关闭总Goal。

用户关闭后原始false已由原生API前后复核。未重启窗口录像11.985秒基本静止，运行中恢复未通过，根因尚待排除采集限制并定位。正常重启后的14秒采样breathing=true、blinks=3、reduced=false，启动恢复通过；应用已重新运行。见verification-reduced-motion-macos.md，不关闭总Goal。

2026-09-14诊断版运行中开启实测通过：用户开启后同一PID 56576的媒体查询变化采样与托盘显示后的独立14秒采样均为breathing=false、blinks=0、reduced=true，原生API前后true，展开截图正常。设置当前true，等待恢复原始false并验证同进程恢复；不宣称关闭恢复已修复，详见verification-reduced-motion-macos.md。

随后用户关闭，同一PID 56576停靠时恢复3次眨眼，托盘展开后独立14秒采样breathing=true、blinks=3、reduced=false，原生API前后false确认恢复原始设置。该路径恢复通过，无重启与动画控制改动；历史失败根因未定，关闭时仍展开及减少效果下过渡等条件不据此通过，详见verification-reduced-motion-macos.md。

正常LaunchServices启动计时新增失败样本：渲染ready=true，但至报告4314.51ms超过3秒，见verification-launchservices-startup-macos.md。修正的是验证脚本，未改产品；新PID 61042保持运行。报告含诊断等待与查询开销，不代表精确首帧，后续需分离测量开销并定位耗时，不关闭总Goal。
