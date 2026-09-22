当前归属 Story：DS-S1；规划：`docs/specs/epic-DS/story-DS-S1/README.md`。旧编号保留历史追溯。

# 任务

- [x] DS-S1 RUN-ASSET-01～04：正式宿主接入四状态完整素材，保留加载回退和旧边缘双眼
- [x] 独立verification-state-playback-macos.md：页面状态/冻结/回退与macOS原生证据；Windows暂缓
- [x] DS-S1 LISTEN-01～04：成功 Agent `task.create` 驱动瞬时收到请求动画，拒绝无动画、执行优先
- [x] 独立 verification-listening-agent-macos.md：Application信号、TaskHost优先级、正式本地Agent与页面播放证据
- [x] DS-S1 WAKE-STATE-01：隐藏态任一展示状态变化主动唤醒，相同值不重复

- [x] 建立 Epic、Story、OpenSpec 与 Verification Goal
- [x] 安装并记录 Rust/Tauri 开发工具链
- [x] 创建最小透明窗口与托盘应用
- [x] 接入透明小龙，按用户反馈缩小为 200×200，macOS Release 构建并重启
- [x] 呼吸、眨眼、点击反馈及拖动分流实现；ego-browser 页面验证通过（verification-animations.md）
- [ ] 动画 Windows/macOS 原生完整验证
  - [x] macOS 原生透明边缘、待命双爪、背景与眨眼子范围证据
  - [ ] 真实 running/等待/暂停完整原生链路、九态完整语义与 Windows 原生验证
- [x] 3 分钟闲置、最近四边探头及躲藏眨眼、点击回原位及原生任务门禁判断实现；ego-browser 页面回归通过
- [ ] 最终边缘休眠原生/双平台验证与正式任务事件接线（verification-edge-rest.md）
  - [x] 四边布局、边界、等距选择与浏览器行为回归
  - [x] macOS 真实三分钟右侧隐藏、顶部原生视觉/几何与正式 Agent 状态唤醒
  - [ ] macOS 其余三边完整原生交互、点击恢复与跨 Space 连续可见
  - [ ] Windows 对应原生验证
- [ ] 透明小龙双平台视觉、尺寸及拖动验证（verification-dragon.md）
  - [x] macOS 透明、200×200、原生完整形象与尺寸证据
  - [ ] macOS 人工原生拖动验证
  - [ ] Windows 视觉、尺寸与拖动验证
- [x] 补充桌宠原生拖动与最小权限，配置检查和 Windows Release 编译/启动通过
- [ ] Windows 人工拖动验证；macOS 按用户要求暂缓，不据此通过双平台门禁（见 verification-drag.md）
- [x] 创建 Local Socket echo 与 CLI 探针
- [x] macOS同进程宿主echo接线、宿主UDS所有权、错误请求后可用、退出端点清理及重启（verification-host-ipc-macos.md）
- [x] 新增IPC验证线程后的新版全应用资源读数复测（CPU0.170%、均值122.24MiB、峰值135.55MiB；同期视觉限制见verification-host-ipc-budget-macos.md）
- [x] macOS 真实 UDS 往返、无 TCP 套接字与服务端退出清理验证
- [x] macOS独立IPC Spike的错误版本与无效JSON拒绝、异常退出端点清理（不替代跨用户权限拒绝）
- [x] 原生托盘图标及显示/退出菜单实现，前端找回事件回归通过
- [x] macOS原生托盘AX菜单与当前宿主/WebKit退出清理（verification-exit-coalition-macos.md；Windows及未来Worker不在本项范围）
- [x] 完成 Linux 编译自检
- [ ] 完成 Windows 实机验证并保存证据（2026-09-13 用户暂缓，保留门禁）
- [ ] 完成 macOS 实机验证并保存证据
  - [x] 已保存托盘/退出、IPC、动画、资源、启动、右侧休眠与唤醒等子范围证据
  - [ ] 四边完整原生交互、跨用户拒绝、完整状态延迟与完整生命周期仍缺
- [x] 产出 AD-E0-01 Proposed，并复核 macOS 已通过子范围与 Windows 暂缓边界
- [ ] 评审并决定 AD-E0-01 是否 Accepted；接受前必须补齐双平台当前版本证据和剩余 macOS 缺口
- [ ] 完成 Verification Goal

- [ ] 用户确认的环绕图标菜单：规格已记录；按用户要求暂缓实现与验证

- [x] 修复 macOS 全屏 Space 缺少辅助窗口行为（本机清醒形态实拍通过）
- [x] 用真实全屏前后屏幕区域复验并保存独立记录

- [x] 系统资源分组归属的五分钟自然闲置与持续展开采样
- [x] 缩小运行时素材后完成 macOS 持续展开复验（均值117 MiB、峰值123 MiB；限制见 verification-runtime-assets-macos.md）

- [x] 生成三倍显示尺寸的运行时图片副本并保留原图
- [x] 验证素材透明度、原生完整形象及页面呼吸/摇尾/自然眨眼；其他原生交互门禁保留
- [x] 完成相同资源统计口径的五分钟持续展开对照，独立记录截图方式与缓存差异

- [x] macOS单次新进程渲染就绪与托盘几何恢复计时，保留原始日志和独立验证
- [ ] 完整可用状态、核心窗口状态更新延迟及Windows对应性能证据
  - [ ] macOS 完整可用状态与核心窗口状态更新延迟
  - [ ] Windows 当前版本性能证据
- [x] 正常LaunchServices启动渲染就绪功能：ready=true，4314.51ms；按2026-09-14用户变更功能PASS，性能优化后置（verification-launchservices-startup-macos.md）

- [x] macOS从托盘明确重置时刻测三分钟无任务闲置，180.23秒仍展开、180.50秒首次几何收起（verification-idle-timing-macos.md）
- [ ] 跨用户Socket权限拒绝实测（sudo -n缺授权，不能以父目录0700替代）

- [x] 修复并验证无指针语义click入口；macOS未激活时AXPress唤醒通过（verification-accessibility-macos.md）
- [x] macOS已聚焦双眼按钮的原生Enter/空格唤醒，定向PID事件及状态/尺寸后置检查通过（verification-keyboard-macos.md）

2026-09-14生命周期素材子范围：关联DS-S1三份设计ASSET-01～04、assets/mascot/互动动画生命周期-v1.md。Architecture Impact：none（只新增图片与声明式清单）；生成9种状态四帧图集并独立验证角色/alpha/边缘/清单。原图保留，不修改运行时状态/协议/持久化。完整UI生命周期与Windows仍未Done/Archive。

本批素材处理：已获用户明确Python/Pillow授权；原图保留，生成400×400透明帧和声明式播放清单，独立验证记录见verification-state-assets-v2.md；不修改运行时状态接线。
