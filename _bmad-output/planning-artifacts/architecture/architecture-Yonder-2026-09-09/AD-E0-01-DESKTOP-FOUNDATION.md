# AD-E0-01 桌面基础栈（临时结论）

- 状态：待 macOS 验证

2026-09-14研发依赖增量：按Accepted AD-DS-01，完整DS-S1不再作为macOS研发接线的循环前提；接线仍须逐项证明可信身份、真实SQLCipher源和恢复/准入条件。Spike本身不承载正式产品模块，不据此将本ADR转Accepted。

## 2026-09-14 用户变更：当前阶段以功能通过为准

用户明确要求适当放宽性能、以功能实现为目标，允许当前启动功能pass并继续下一项。当前研发阶段3秒启动、1% CPU、150MB内存及300ms事件延迟作为优化目标，不作为功能实施的阻断门槛；保留原始读数与失败历史，不伪造达标。发布前重新评审性能预算。已取得真实LaunchServices渲染ready=true的4.31秒样本，因此本机启动就绪功能通过；不将其扩大为完整交互、Windows或整项DS-S1通过。不因此解除协议、安全、事实源或尚未解决的正式接线依赖门禁。
- Story：E0-S1
- OpenSpec：`e0-validate-desktop-foundation`
- 日期：2026-09-10

## 当前决定

### 2026-09-13 宿主IPC验证边界澄清

既有OpenSpec design已要求“Rust Core同进程运行Local Socket echo服务”。补齐该项属于本Spike验证，不依赖正式Application任务接线，也不授权正式产品模块。复用独立IPC Spike的唯一Rust消息类型及传输实现为本地库，由Tauri宿主启动可停止的验证线程；正常退出先停止并等待该线程，释放测试Socket。CLI继续使用同一协议。仅固定版本echo，无任务创建、Gateway、数据库或凭据；测试请求最多64KiB，避免未终止输入无限增长。

本澄清不将ADR转为Accepted，不放宽双平台门禁，不解除正式任务接线的前置冲突。当前允许继续的范围只有已约定的宿主echo及退出验证；正式忙碌任务验收仍单独保留。

保留 Tauri 2 + Rust Core + `interprocess 2.x` 作为桌面基础栈候选。Local Socket 的传输代码保持统一，名称构造在 Adapter 内按平台分支：Windows `GenericNamespaced`，Unix/macOS `GenericFilePath`。

## 证据

Windows 11 上 Tauri 2.11.5 Release 可构建运行，Named Pipe 往返成功；五分钟空闲 CPU 0.109%，平均 RSS 约 69.84 MiB，窗口句柄建立 1.192 秒。Linux 编译与运行自检通过，但不属于目标平台门禁证据。

## 未决项

### 2026-09-11 未激活动画验证修复

原生窗口仍可见时，WKWebView 的 document.hidden 不能作为桌宠动画的唯一可见性来源。Spike 使用 Tauri 原生 is_visible/is_minimized 查询，并在 macOS 14+ 将 backgroundThrottling 设为 disabled，使未激活但可见的桌宠继续呼吸与眨眼；真正隐藏、最小化、边缘休眠和减少动态效果仍主动停动画。此策略需重新测量资源预算，不据此支持较旧 macOS 或 Windows；不引入额外依赖或任务状态所有者。

### 2026-09-11 跨桌面与边缘休眠 Spike 决策

在 E0-S1 中验证 Tauri 的 visibleOnAllWorkspaces，使 macOS 桌宠跨 Spaces 保持同一窗口及坐标；Tauri 明确不支持 Windows 的同名能力，本批不据此宣称 Windows 虚拟桌面可用，也不引入 Windows 私有实现。全屏 Space 的行为单独验证。

休眠仅属于窗口展示生命周期：Rust 宿主保存内存中的停靠前位置/尺寸，是恢复位置的唯一所有者，不写入数据库或用户配置；前端只维护动画阶段和本窗口的交互计时，不监控全局输入。闲置后把窗口缩成 112×56 逻辑像素的双眼探头入口，停在当前显示器工作区下沿；避免将窗口移出屏幕后落到相邻显示器。唤醒由 Rust 恢复原位置，显示器移除时限制在可用屏幕工作区。增加仅供 pet 本地窗口调用的 dock/wake 命令，不对 Agent Gateway 暴露，不改变任务状态。隐身不取消任务或开始录制。重启记忆位置不属于本增量。

### 2026-09-11 透明小龙窗口验证增量

在 E0-S1 Spike 内接入已生成的透明 PNG 小龙，保留原生拖动，关闭窗口阴影。macOS 按 Tauri 2 现有配置启用 `app.macOSPrivateApi`，并启用 `tauri/macos-private-api` Cargo feature，使 WebView 透明设置生效。该能力仅限候选桌面栈验证，不改变状态所有者、通信协议或模块边界；不据此定案产品发布路线。实际透明效果与后续分发约束仍需验证，ADR 保持待定。

2026-09-11 macOS 独立 IPC Spike 已完成真实 UDS 往返、测试进程无 TCP 套接字、服务端正常退出与端点清理。托盘新增原生图标及显示/退出菜单，尚未取得真实菜单点击证据。宿主五分钟采样不含 WebKit 辅助进程，不能作为整体资源预算通过依据。详见同 Change 的 verification-native-macos.md。

macOS 的完整透明置顶/托盘交互、宿主 IPC 接线、权限拒绝、完整退出、启动/事件延迟及全进程资源预算仍待验证；在其完成前，本 ADR 不得转为 Accepted，E0-S1 不得完成。


### 2026-09-11 最近边缘与躲藏眨眼增量

Rust 在停靠时使用当前显示器及窗口几何选择最近边缘，仍唯一保存恢复位置。左右使用显示器边界；底部使用工作区下沿贴近 Dock/任务栏，顶部使用可见边缘避开菜单栏以免眼睛被遮挡。距离相同按底、左、右、顶稳定排序。pet_dock 返回 Rust 计算的边缘名称，前端据此旋转探头姿势：底部趴伏、侧边探头、顶部倒挂；不复制任务协议或保存新配置。水平入口 112×56、侧向 56×112，缩放以当前显示器为准。休眠时停止身体动作但保留 3–5 秒眨眼；真正隐藏/最小化或减少动态效果时仍停。任务门禁、三分钟闲置和原位置恢复不变。此增量在候选 Spike 中验证，不使 ADR 定案。

### 2026-09-12 全屏 Space 可见性修复验证

真实纯色 AppKit 窗口进入全屏后，休眠小龙在原坐标的屏幕区域消失，退出后恢复。稳定 4 秒采样仍缺失，不是单点 onscreen 误判。已核对锁定 Tao 0.35.3 实现：visible_on_all_workspaces 只设置 CanJoinAllSpaces。

在候选 Spike 的主线程 setup 中，使用 macOS 公共 NSWindowCollectionBehavior 补充 FullScreenAuxiliary，并移除互斥的 FullScreenPrimary/FullScreenNone。普通桌面标记继续由现有 Tauri 配置设置，保留其他窗口行为。复用锁文件已有 objc2-app-kit 0.3.2，只增加 macOS 直接依赖及 NSWindow feature，不引入新技术栈或状态所有者。

这仅是 DS-S1/E0 验证修复，不将 ADR 转 Accepted，不绕过 Windows 及其余门禁。先用同一全屏对照测试验证原坐标实际屏幕内容；若仍失败，回到诊断，不据标记设置成功宣称通过。

用户明确补充：Yonda 不应有全屏应用形态，只做轻量桌面插件。对应 macOS 候选宿主使用 Accessory 激活策略及 LSUIElement 预览包属性，提供桌宠与托盘而非 Dock 普通应用入口；窗口禁止全屏平铺，仍可作为其他应用的辅助悬浮窗口。单独 FullScreenAuxiliary 的实测未通过，继续验证 Accessory 组合，不追认为已完成。

进一步核对本机 Apple SDK NSWindow.h：CanJoinAllApplications 自 macOS 13 可用，明确用于悬浮窗口/系统覆盖层加入其他应用的全屏空间，且与 Primary/Auxiliary 互斥。候选实现使用 objc2::available! 运行时判断后设置该位并移除两种冲突角色；低于 macOS 13 不设置，不能声明旧版本全屏支持。objc2 0.6.4 已在锁文件，仅增加直接引用以使用系统版本检查；不引入私有 Space API。Accessory 形态是用户定位要求，不能与全屏可见性测试结论混为一项。

### 2026-09-13 运行时素材尺寸验证

持续展开时四进程 footprint 求和均值164 MiB、峰值183 MiB，预算未通过。发现主体/闭眼原图1254×1254、探头/闭眼原图1774×887，明显大于200逻辑像素宿主的显示需求。用户明确授权使用系统图片工具生成小尺寸运行时副本并保留原图。

先做单变量实验：主体/闭眼副本600×600，探头副本336×168，覆盖现有尺寸3倍缩放；原图不改动。HTML仅替换到副本，不同时修改动画频率、图层或视觉交互。使用系统sips离线生成，不添加运行时图像处理或技术依赖。默认屏幕缩放高于3倍或未来桌宠放大时须重新生成对应尺寸，不能宣称无限缩放无损。是否降低实际内存由同口径五分钟原生复测决定，不能以理论解码像素数代替验收。
