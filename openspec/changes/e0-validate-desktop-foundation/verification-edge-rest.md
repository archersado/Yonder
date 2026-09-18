当前归属 Story：DS-S1；规划：`docs/specs/epic-DS/story-DS-S1/README.md`。旧编号保留历史追溯。

# 独立 Verification Goal：E0-S1 边缘休眠

日期：2026-09-11；关联 E0-S1、AD-E0-01 跨桌面与边缘休眠决策、同 Change 的「桌面常驻与边缘休眠」。状态：实现与页面回归通过；macOS 已补真实三分钟右侧隐藏及正式 Agent 状态唤醒，四边完整原生与 Windows 仍未通过，不归档。

## 最终交互

- 默认 3 分钟没有与小龙互动后缩身；Space 切换、失焦与 WebView 隐藏不算互动，不重置计时。
- 使用专用透明 PNG：底部趴伏、侧边探头、顶部倒挂，露出头部、一双眼睛和小爪子。水平入口 112×56、侧边 56×112；顶部避开菜单栏，底部使用工作区下沿贴近 Dock。躲藏后每 3–5 秒眨眼，停止身体动作。旧单眼裁片为已放弃方案。
- 点击或键盘激活回到停靠前位置与尺寸；显示器移除时限制到可用工作区。位置仅驻内存，不跨重启保存。
- macOS 配置 visibleOnAllWorkspaces；不保证 Windows 虚拟桌面或 macOS 全屏 Space 已验证。

## 证据

1. `cargo test --release --offline --locked --manifest-path spikes/desktop-foundation/src-tauri/Cargo.toml`：1 个测试通过，覆盖 Some(0) 可停靠、Some(1)/None 禁止停靠、负坐标显示器与过小工作区限制。
2. ego-browser TaskSpace 6 使用同一个原始页面测试；测试注入将 180000 毫秒定时压缩为 900 毫秒，记录生产代码请求的延迟仍为 180000，未修改生产阈值。
3. 等待自动停靠，DOM 尺寸为 112×56；原图 naturalWidth=1774，sips 确認 alpha 存在。深色背景截图见 `spikes/desktop-foundation/evidence/animations-macos/edge-pair.png`，已人工查看：连着头部的一双眼睛和前爪，无底板。
4. 点击后命令顺序为 pet_dock → pet_wake，页面回到 awake。休眠时 document.getAnimations() 为 0。
5. 浏览器测试桩拒绝停靠时回到 awake；唤醒失败留在 docked，第二次成功请求恢复 awake。数据见同目录 `edge-checks.json`。这是前端失败处理测试，不冒充真实后台任务源。
6. 发现测试跨轮次重新加载会丢失 CDP 注入，造成按生产三分钟等待而超时；修正为同轮次注入和加载后通过。此超时不是通过结果。
7. macOS 启动日志确认原生 image decode、188×188 展开尺寸、样式与脚本初始化成功。发现 WKWebView 在未激活时 page_hidden=true；移除其对休眠计时的阻断和重置，并以浏览器 hidden=true 场景复核。
8. 初版原生停靠日志为 position=(3332,1528)、size=88×96，证明原生命令可改变窗口位置/尺寸；该证据属于已替换的单眼侧边版本，不算最终双眼底边坐标通过。

## 后台任务门禁

Rust 在停靠前检查正式 Application/SQLite 派生值，运行中或未知一律拒绝；前端没有修改计数的命令。2026-09-15 已用正式私有 stdio Agent 完成真实三分钟右侧隐藏、成功 `task.create` 主动唤醒和 `task.cancel` 保留数据，证据见 `verification-listening-agent-macos.md`。执行中禁止隐藏已有组合根测试；真实执行器长任务闭环仍随相应 Story 验证。

## 待验收

### 2026-09-11 原生顶部躲藏眨眼续验

屏幕录制和辅助功能权限均为 true。先前查询只使用 optionOnScreenOnly，返回空列表；改为读取目标进程全部窗口后定位到窗口 6648，再激活 Yonda，取得当前屏幕可见窗口。未将空列表直接认定为截图权限缺失。

真实顶部停靠为逻辑 (1098,34)、112×56，层级 5。native-edge-current.png 显示透明倒挂小龙，眼睛未被菜单栏遮住。native-top-blink.mov 为该原生窗口 12.032 秒录屏，20Hz 分帧检查得到三次闭眼区间：2.10–2.20、5.35–5.45、9.55–9.65 秒；相邻间隔 3.25 和 4.20 秒。查看 native-top-blink-frames/baseline.png 与 largest-change.png，确认变化来自双眼闭合，躲藏身体保持静止。采样帧间隔不等于精确闭眼持续时间。

原生日志记录右侧停靠与恢复，以及顶部恢复物理坐标 (2108,68)，对应停靠前逻辑 (1054,34)；恢复尺寸 400×402 物理像素。测试者第一次原生点击因窗口不在当前桌面中止，第二次点击时小龙已展开且位置改变，所以不把该点击算作唤醒操作证明。恢复日志单独作为几何结果证据。

结构化结果：animations-macos/native-edge-verification.json。结论：本轮顶部原生躲藏/眨眼视觉通过，右侧和顶部停靠/恢复几何有日志证据；四边完整原生交互、精确三分钟计时、Windows 与跨 Space 完整连续可见仍未通过。此前“本轮窗口查询空列表”的限制已在此补充突破，不表示所有原生项目通过。

### 2026-09-11 四边躲藏增量

- 原生 dock_layout 比较四边距离，等距按底/左/右/顶稳定选择；pet_dock 返回边缘，重复停靠不覆盖原位置。
- Rust Release 测试 2 项通过：门禁、四边、负坐标、2 倍缩放、等距选择与坐标限制。Release 构建通过（12.96 秒），新 .app 已启动 PID 99143。
- ego-browser TaskSpace 11：check-edge-rest.mjs 使用测试桩返回四个边缘，加速 180000ms 闲置为 900ms（不改生产值）。四边均观察到自然眨眼，水平 112×56、侧边 56×112，实际页面指针点击后恢复 awake。数据 four-edges-checks.json 和四边闭眼截图保存在 animations-macos，TaskSpace 已关闭。
- 闭眼参考生成图带棋盘背景，仅以眼区遮罩叠加，不替换透明全图。edge-mask-review.png 为 3 倍放大视觉检查，确认闭眼配准、无棋盘外露；强制闭眼仅用于截图，不作为自然时序证明。
- 复用探头图旋转形成三类姿势，左右方向相反，未生成四套独立角色素材。
- 本轮原生窗口查询返回空列表，无法取得有效桌面截图。因此浏览器或坐标单元测试不代表四边原生视觉通过；仍需真实三分钟停靠、躲藏眨眼、唤醒及 Windows 复验。

最终双眼版右侧原生三分钟停靠及状态变化恢复已通过；点击恢复、跨 Space 连续可见、全屏 Space、屏幕插拔、其他三边完整原生和 Windows 原生行为仍是总体验门禁。未达到这些门槛前不将 Story 标记 Done。
