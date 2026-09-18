# macOS接管停止补充Spike

当前路线（2026-09-14用户变更，Accepted AD-CU-01）：仅SDK，不附带上游App/可执行文件。下述STOP-PKG分支撤回，仅作历史；完整包虽已下载，不解包/评估/运行。STOP-SDK01–04转为自管SDK子进程就绪、宿主监督终止并确认退出、通道断开关闭退出、新实例只读恢复，期限/唯一trycua/隐私/原生输入门禁不变。macOS独立Goal已通过只读范围，下一样本为SDK隔离原生输入/停止/Observe与正式Yonder权限责任链，Windows继续暂停。

SDK原生输入首批STOP-IN01：在实际SDK导入宿主检查currentMacOsPermissionStatus，仅返回accessibility/screenRecording布尔，不请求权限或打开设置。缺少Accessibility时不创建输入样本、不派发输入，记录阻断与宿主归属；屏幕录制缺失不阻断无截图AX样本。STOP-IN02后续仅在隔离原生测试输入框唯一PID/窗口/AX token确认后写固定测试标记，立即SDK Observe与测试目标状态比对；不做前台像素/剪贴板兜底。STOP-IN03先验证动作间停止，无新的准入及停止后稳定Observe；不能用动作间停止证明执行中原生动作中断。真实输入样本与执行中停止仍须后续实机证据，Windows暂停。

首批输入样本实现：临时Swift/AppKit可执行测试窗（不打包成App、不进入产品分发）仅一个NSTextField；只输出PID/窗口号/字段长度/固定标记匹配布尔。SDK按该PID/窗口及唯一AX token后台type_text，动作后get_window_state无截图，SDK字段内容与测试窗匹配布尔同时成立才通过；shutdown后向同SDK提交第二标记必须拒绝，新SDK Observe确认原标记仍在，测试窗750ms内长度/匹配稳定。拒绝、权限丢失或目标不唯一不做兜底；自动关闭仅本次测试窗，不影响真实任务或其他应用。

STOP-DRAIN01–03：测试窗可选延迟AX setter模式，实际收到固定标记写请求时先输出native_write_started与时间，再等待2秒并应用，输出native_write_applied与时间。SDK由自管子进程加载且自行取得AX token，父进程必须看到原生setter开始、SDK调用仍未返回才请求shutdown。记录关闭是否等待写入完成，并由新SDK无截图Observe确认最终状态；没有原生setter开始证据则样本失败，不把JS提交当准入。目标仅用延迟模拟应用处理在途请求，不声称真实应用都有相同时延；结论仅此AX排空场景，不覆盖强制中断、像素拖动或正式Yonder宿主权限。

内置NSTextField实际AX桥绕过公开setter后，延迟模式采用NSView显式实现NSAccessibility textField role/value/setValue入口，仍通过真实系统AX调用；普通NSTextField基线不变。只允许固定测试标记，原生回调提供开始证据，不能使用自制SDK替代或伪造调用成功。

关联CU-S1、CU-S2、TM-S3、RC-S1与AD-E0-02/AD-TM-03；用户继续开发接管。技术验证期限：2026-09-14至2026-09-16；Windows新增验证按用户暂缓，不覆盖既有Windows选型证据，不以macOS只读测试完成CU/Recording Story。

统一生命周期样本复用fault-probe/crash-recovery：未知工具、调用前取消、Driver关闭、重复关闭、关闭后拒绝、异常退出与新实例恢复。仅执行Driver元数据/只读调用，输出布尔及错误分类，不输出应用、窗口、正文或完整Payload；不记录键鼠、截图或开始Recording。

淘汰/门禁：调用前取消不能当成已停止，shutdown返回不能当成系统动作已停止，进程退出不能当成原生输入worker已停止。任何实际行为未知则fail closed，保留占用，要求Driver原生停止证据。首批只读取消/退出通过仅支撑后续停止契约设计，不能授权CUA动作派发或接管记录。

选型复用已Accepted的trycua0.25.0，不重启被淘汰Qwen选型、不在产品增加双执行栈。补充证据关联原e0-compare-cua-drivers change及独立verification-stop-macos.md；结论回写ADR macOS限制和TM/RC设计前置，完整接管还需Driver停止确认、隐私采集、持久化及交回Observe联合设计。

同期限增加STOP-SUB01–04：只读调用提交后setImmediate Abort、取消后新读、未等待只读调用时shutdown及关闭后拒绝。JS提交不等于原生准入；竞争失败或调用提前完成如实分类，不强制伪造长动作。已安装README说明shutdown为关闭准入并等待已准入操作结束；不把等待排空改称强制中止。createPrivateWorker所需独立可执行文件未随npm分发，本次不拿同进程SDK或自制假Worker代替原生Worker验证。

STOP-PKG01–03补充供应链样本：从官方cua-driver-rs-v0.25.0 Release取得darwin-universal-binary归档，核对发布SHA-256，先检查归档路径再解包到忽略的evidence目录；验证Mach-O平台/架构、codesign签名身份与Gatekeeper结果，失败不运行、不移除隔离属性、不自行重签。不执行上游安装脚本，不注册Daemon/服务、不复制到Applications或产品包。STOP-PKG01校验与安全解包；02签名/系统评估；03原始证据与平台限制。通过后才允许只读CLI版本/帮助检查；真实输入仍须隔离样本和独立Goal。

裸二进制Gatekeeper拒绝后返回实施阶段，按同版本官方辅助安装脚本静态审查确认macOS正式载体为darwin-universal目录包内CuaDriver.app。允许核验该固定同版本归档的发布哈希、App签名/系统评估及其主程序；不执行安装脚本，不写Applications，不把裸二进制拒绝改成通过，App同样拒绝则保留阻断。

官方来源：[0.25.0 Release](https://github.com/trycua/cua/releases/tag/cua-driver-rs-v0.25.0)。静态审查脚本哈希与Release一致：install.sh=317ba3a49fdba10f2a7f1b9f392c1bc1b7657f3aae85e1e2e43684cf17a1bf3b；_install-rust.sh=0fe64971708a951a0b275b900ec71fb1fa5d7e4bfe0b87c2118c835d5be0e5b4。脚本仅作为布局/身份审查材料，不运行。

## 工作定位SDK补充样本（2026-09-14）

期限仍为2026-09-14至16，唯一trycua0.25 SDK-only，不创建附加App。关联TM-S3 FOCUS-01～04、CU-S2、AD-TM-07与AD-CU-02。先读取安装SDK工具契约，仅保存窗口控制工具名和schema，不输出窗口/应用内容。FOCUS-SDK01精确PID/window_id目标，正常前置后原生active/key/activeSpace反馈与SDK无截图Observe；02同一最小化目标恢复后前置；03关闭目标拒绝且不前置同名新窗；04额外Space/显示器保持工作原位置，缺实机环境则明确暂缓，禁止用同桌面结果外推。

临时原生Swift夹具仅测试窗口，固定stdin控制最小化/后台/关闭；输出身份与active/key/minimized/activeSpace布尔，无用户行为采集。失败门槛：SDK没有精确窗口选择、恢复不可验证、失效后模糊兜底均不进入产品；失败如实写独立Goal，方案改用既有原生平台能力必须先ADR。窗口前置只验证工作定位，不是停止确认或Recording通过；真实任务仍不可派发，Windows按用户暂缓。

SDK catalog实测仅list_windows/get_window_state/set_window_frame，无前置或最小化恢复工具。禁止用改变窗口位置冒充前置。进入原生AX/AppKit隔离路线验证，不修改产品依赖或启用任务接管。原生目标先验证PID/window_id的WindowServer所属/存活，再与目标进程AX窗口按标题和原始几何做唯一映射，多个候选拒绝；标题仅辅助身份映射，不是按名称打开工作。测试含同名诱饵窗口，关闭真实目标后不得前置诱饵。生产设计还必须携带进程启动身份/窗口指纹并复核，避免PID复用；Spike固定自身PID无此生产授权。
