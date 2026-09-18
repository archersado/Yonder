# DS-S2 任务状态与条件悬停独立 Verification Goal

日期：2026-09-14；Story：DS-S2（关联DS-S1）；Change：ds-s2-task-overview；AD：AD-DS-01、AD-ST-01。状态：本机空库、面板操作和分层回归PASS；真实运行任务全链路、完整生命周期姿态与Windows未通过，不Archive。

## 验收映射

依据两Story产品需求最新用户变更与DS-S2 AC2/AC5/AC6：有未结束任务才自动悬停显示、真实执行映射既有executing、无任务不显示；菜单与小龙同桌面，移入可操作、移出隐藏。原生窗口集合行为同时应用两窗口；去除失焦直接隐藏，避免操作面板时错误关闭，改由原生区域观察控制。托盘手动入口在首次移入前保持，首次进入后移出隐藏；自动入口跨间隙350毫秒宽限。

TaskHost派生展示只读取既有SQLite和唯一Admission，不新增任务持久化状态。执行优先，否则以首个未结束任务作代表；waiting_for_user/paused/idle按生命周期名称切换，未知保留未知。原生pet_task_state身份固定本地pet窗口，用blocking worker避免SQLite阻塞UI。task_menu_show在Rust再次检查未结束任务，前端不能自报任务数打开空面板；托盘可主动查看空库/历史。

## 核心与浏览器证据

cargo test --offline --locked -p yonder-desktop首次5项通过；补充等待/暂停转换后--lib三项通过。同一实际SQLite样本经过中断恢复→paused、Resume→executing、WaitForUser→waiting_for_user、Resume/Pause→paused、Complete/Cancel→无任务idle。应用身份伪造仍拒绝，重开仍无未结束任务。

ego-browser TaskSpace6，check-task-space.mjs回归emptyHoverHidden、tasksArriveWhileHovering、executingAnimation、menuHoverKeepsOpen、leaveHides、waitingAndPaused、clickDoesNotOpen、keyboardAndDrag、unknownOnFailure均true。只注入明确UI测试值，不写产品库；已finish。执行爪层动态属性采样变化；task-state-executing-browser-20260914.png人工查看为原有透明角色，没有棋盘背景。

生成四帧及去背景尝试均无真实alpha，未进入仓库运行资产。按生命周期文档补充设计使用原有透明PNG爪部局部遮罩分层、交替微转1.4秒循环；身体变换仅呼吸/前倾，不代替局部动作。原图保留。完整求助举爪、暂停收翼的姿态素材仍待补，不称其完整动画验收通过。成功/失败需要事件sequence去重，不能因快照刷新重播，尚未实现。

## 原生证据与失败历史

check-task-menu-conditional-macos.swift使用真实鼠标移动，空库悬停3秒不显示；托盘主动打开后鼠标移入菜单，AXPress刷新仍保持且真实内容为空；移出后有界等待隐藏。它不把辅助功能点击当物理鼠标按钮验证，也不伪造产品任务。

早期conditional及ready退出7，kept=false；中间trace构建通过，最终第一次复验仍退出7。发现共享退出检测错误给托盘手动入口套用悬停跨间隙超时，返回Apply修复。最终构建完成后正常退出/替换/LaunchServices启动，conditional-manual-20260914退出0：PID2707，empty_hover_hidden、panel_hover_keeps_open、refresh_operable、leave_hides、real_empty_state、same_visible_desktop全部true。证据native-panel-operation.png/result.json。工具等定位短暂稳定后才读取实际可见窗口坐标，不用旧AX坐标。same_visible_desktop仅证明本次两窗口同时在当前可见桌面，全部Space/多屏/全屏切换仍保留验收。

## 剩余范围

正式库当前为空，没有外部Agent任务创建/执行接入；Codex会话里的工作不会自动成为Yonda任务。真实running任务触发本机波形/悬停的完整原生E2E待接入后验证，不能用分层样本代替。Windows按用户要求暂缓；任务暂停/取消等执行按钮尚未接通，现有可操作按钮为详情、刷新、筛选和分页。完整RestPermit协调仍是开放执行前门禁。Story保持实施中，不Done/Archive。
