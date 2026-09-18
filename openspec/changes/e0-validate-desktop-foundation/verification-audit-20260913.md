# DS-S1 完成条件审计

日期：2026-09-13。Story：DS-S1；OpenSpec：e0-validate-desktop-foundation。

结论：**完成未被证明，不能Archive或Done**。本表按当前产品需求、增量规格全部场景、Verification Goal及架构门禁盘点；不以已经做出的实现重新定义范围。

## 验收映射

下列verification文件均在本Change目录，原始JSON/截图按各文件引用检查。已通过项的范围仅限对应平台及样本。

| 原规格场景/门禁 | 当前证据与判定 | 还缺什么 |
|---|---|---|
| 启动验证程序、托盘找回与退出 | macOS原生截图、托盘AX找回及同组四进程清理；verification-exit-coalition-macos.md | 当前Windows版本；普通鼠标菜单路径不能被AX调用完全覆盖 |
| 透明小龙形象、打包样式加载 | macOS完整透明形象、600像素副本、原生style/script ready；verification-runtime-assets-macos.md、verification-latency-macos.md | Windows当前素材/打包视觉；拖动另列 |
| 呼吸眨眼、未激活仍活动 | macOS原生14秒诊断有呼吸及3次眨眼；真实不同姿态截图；页面测试通过 | 当前原生完整周期/摇尾、真正隐藏/最小化后的停动与恢复、减少动态效果 |
| 点击与拖动分流 | 页面测试通过；未把用户眼睛唤醒反馈扩大为清醒点头通过 | 清醒单击/快速连点/拖动分流的原生闭环；macOS拖动在历史verification-drag.md中仍为用户暂缓 |
| 普通桌面切换 | 右侧休眠形态原生A→B→A可见且位置稳定；verification-normal-spaces-macos.md | 清醒形态普通桌面往返；其他边缘/显示环境不能由该样本替代 |
| 其他应用全屏进出 | 清醒及右侧休眠原生全屏辅助悬浮通过；verification-fullscreen-overlay-macos.md、verification-rest-fullscreen-macos.md | 保留平台范围，不要求Yonda成为全屏应用 |
| 三分钟闲置停靠 | 从托盘重置开始180.23秒仍展开、180.50秒收起；verification-idle-timing-macos.md | 有交互/按住期间不收起、完整缩身时序及不同状态重置 |
| 最近四边、三类躲藏形态、休眠眨眼 | 四边/负坐标/缩放/等距Rust测试；页面四边；历史顶部原生眨眼与右侧可见 | 当前四边原生完整闭环、Dock/菜单栏边界、多屏/不同缩放实机。单元/页面不能替代 |
| 点击唤醒 | 用户2026-09-13明确手测可唤醒；托盘恢复原坐标/尺寸实拍 | Enter/空格、重复点击、减少动态效果、原生命令失败重试、显示器移除回退；合成点击失败单列为工具问题 |
| 后台任务禁止隐藏、结束后重计时 | can_dock拒绝busy/unknown单元测试；TM-S6核心协调独立实现 | **未接线**：PetWindowState初始化Some(0)，当前桌宠没有执行器，不能用假任务数冒充正式接线 |
| CLI echo、本地Socket不监听TCP | 独立IPC Spike同用户往返、无TCP、版本/JSON拒绝及清理通过；verification-ipc-rejection-macos.md | 桌宠宿主IPC接线；真正不同用户拒绝。sudo -n要求密码，探针未执行到连接 |
| 性能与启动/窗口变化状态更新 | macOS四进程持续展开均值117.22MiB/峰值122.85MiB，CPU0.155%；直接可执行文件就绪1.99秒、托盘几何恢复约52ms | 正常.app启动到可用的端到端证据、300ms状态更新链路。直接启动无独立资源组；几何恢复不是状态事件延迟 |
| 环绕菜单的两项场景 | 已有规格；用户明确暂缓 | 保持暂停，不能当作已实现或从原规格删除 |
| Windows各自原生证据 | 旧版本编译/Named Pipe及资源证据；用户2026-09-13暂缓当前版本验证 | 不再安排Windows执行，完成门禁继续存在 |
| AD-E0-01评审与独立Goal/Archive | AD仍未Accepted，独立分项验证存在 | 完整Goal未通过，禁止Archive/Done |
| Story/OpenSpec/PR及CI | 四份Story设计与Change关联存在；本机结构检查通过 | Story仍design-review，现有CI不检查全部原生证据真实性或前端依赖；无本轮独立PR完成证据 |

## 必须先解决的前置冲突

1. DS-S1产品需求和增量规格要求忙碌任务/未知禁止隐藏、全部结束后重新计时，并明确正式任务源接线前不可通过。
2. DS-S1架构设计、AD-TM-02及规划入口又规定DS正式接线受桌面Spike门禁约束；当前Proposal明确不实现正式产品模块。
3. 如果把全部DS-S1验收作为桌面Spike准入条件，就出现“通过DS-S1才能接线，而接线后才能通过DS-S1”的循环。当前材料没有独立的阶段准入决策解除它。

按AGENTS与DEVELOPMENT-AND-CHANGE-MODE，此处停止依赖该冲突的实施；本次只记录冲突，不自行修改事实源或放宽门禁。下一步应先形成架构决策，明确基础栈阶段准入与完整Story验收之间的关系，再相应更新Story/OpenSpec；忙碌接线验收仍保留，不能通过缩小Story范围使其“通过”。Windows暂缓同样不能被工程决定变成已验证。

## 当前执行顺序

2026-09-14后续：诊断版同一PID 56576实测开启停动、关闭后边缘双眼恢复眨眼、托盘展开后恢复呼吸/眨眼通过，原始false已复核恢复。通过范围限定该路径；减少效果下停靠/唤醒过渡与关闭时仍展开等未覆盖，历史静帧样本根因未定。

2026-09-14更新：减少动态效果原始false已由原生API复核恢复；开启/关闭后的启动采样通过。运行中恢复尚未通过；正常录屏对照可捕捉动作，已补有界运行中采样以定位。详见verification-reduced-motion-macos.md，不将诊断增量视为缺陷修复。

1. 不重测已有通过的内存、三分钟无任务计时、托盘退出；用户眼睛点击确认已留档。
2. 可独立补macOS清醒普通Space、键盘、减少动态效果/隐藏恢复、四边原生证据；实施或系统设置变更前遵守已有暂停与权限约束。减少动态效果启动采样随后通过（原生true，14秒无眨眼、呼吸首尾变换一致），运行中切换、停靠/唤醒和恢复原始false仍待验证，详见verification-reduced-motion-macos.md。
3. 跨用户拒绝等待可用的既有不同用户执行权限，不在工具中收集密码、不新建账户。
4. 正式任务/宿主接线先解决上述架构前置冲突；Windows、菜单及历史拖动暂停须按用户指令处理。

## 本轮检查

已读取当前spec.md全部场景、proposal.md、三份Story设计、历史AC、任务表、AD-DEV-01和AD-TM-02接线限制，核对当前main.rs独立UI及pet_window.rs的Some(0)任务观察值。

`python3 scripts/check_architecture.py`首次因PATH找不到cargo失败；使用已安装工具链 `PATH=/Users/archersado/.cargo/bin:$PATH python3 scripts/check_architecture.py` 后通过。脚本只证明结构/关联，不代表Story验证完成，也不覆盖隔离Spike的全部架构约束。

清醒普通桌面续验见verification-awake-spaces-macos.md：未取得合格样本，系统入口及前置状态不稳定；创建临时桌面的两轮均清理成功。停止连续创建桌面重试，需稳定原生环境再继续。

宿主IPC增量已完成macOS受限echo接线并独立验证，见verification-host-ipc-macos.md；上表“宿主IPC接线”缺口由该证据更新，跨用户/Windows和正式任务接线仍缺。AD-E0-01明确已有同进程echo属于Spike，不是正式任务模块，不需借此放宽正式接线门禁。新增IPC线程后的当前二进制需重新测资源预算。

该新版资源复测随后完成：verification-host-ipc-budget-macos.md记录四进程均值122.24MiB、峰值135.55MiB、CPU0.1695%。同期只有窗口几何检查，事后探头图不能作为同期视觉证据，不能将本项读数通过扩大为全部动画验收。

无障碍增量：verification-accessibility-macos.md记录语义click缺失的修复及原生AXPress成功。此更新补充辅助功能按钮激活范围；实体键盘、重复点击、减少动态效果等上表缺口仍保留。

键盘增量：verification-keyboard-macos.md通过macOS已聚焦双眼按钮的原生Enter与空格输入路径；上表该两种唤醒输入缺口已补齐，不把它扩大为Tab导航、长按repeat或Windows。此前“实体键盘”表述不作为追加人工物理键盘门槛。
