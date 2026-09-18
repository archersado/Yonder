当前归属 Story：DS-S1；规划：`docs/specs/epic-DS/story-DS-S1/README.md`。旧编号保留历史追溯。

# 独立 Verification Goal：E0-S1 基础动态交互

日期：2026-09-11。关联 E0-S1、同 Change「基础动态交互」、AD-E0-01。状态：ego-browser 页面验证通过；Windows 与 macOS 原生完整交互验证尚未通过，不归档。

## 实现范围

内置 UI 的 3.6 秒轻呼吸、3–5 秒间隔/130 毫秒闭眼、400 毫秒点击点头；增加轻微歪头、偶尔起伏和每 8 秒一轮的短暂摇尾巴。图片变换不移动 OS 窗口；拖动超过 4 像素才调用既有 Tauri 窗口命令。无任务、录音或录制接线，未加入用户素材包脚本。

## 2026-09-11 原生静止问题与动作增量

WKWebView 在窗口可见但未激活时报告 document.hidden=true，而且 CSS 动画时间线冻结。原生 14 秒旧版采样为 breathing=false、blinks=2。改用原生窗口可见性查询、禁用后台节流，并用 50 毫秒间隔更新变换；隐藏、休眠或减少动态效果时清除计时器。新版本原生采样 breathing=true、blinks=3、reduced=false，图片与样式加载成功。

尾巴采用原图 CSS 分层与尾根剪切变换，无新图片和依赖。ego-browser TaskSpace 7 验证未激活时身体与尾巴变换持续变化，原生可见性返回 false 后两者停止；结果 moving=true、wagging=true、stopped=true。同一检查保留为 check-motion.mjs，供后续 ego-browser 页面调用。

macOS Release 离线构建通过并重启 PID 86877。实际窗口截图 native-tail-window.png、12 秒窗口录屏 native-tail-motion.mov 已保存到 animations-macos 证据目录。录屏前段为空，不能使用相对首帧的最大差异数量证明动画。检查 9、9.5、10.5、11.5 秒非空帧，身体倾角与尾尖位置持续变化，尾根未见明显断裂；分帧见 native-tail-samples/sample-180.png、sample-190.png、sample-210.png、sample-230.png。短片后段证明原生动作显示，但不替代完整生命周期验收。Windows、完整交互和五分钟资源预算尚未复验，Goal 保持未通过。

## ego-browser 验证

在同一个 TaskSpace 3 中直接打开仓库 `ui/index.html`，未创建本地 HTTP 服务。使用真实页面指针点击、移动及浏览器原生媒体偏好仿真，保存证据到 `spikes/desktop-foundation/evidence/animations-macos/`。

1. 两张图片加载成功，原图为透明 PNG，body 计算背景为 rgba(0,0,0,0)。闭眼参考图无 alpha，仅显示面部眼区，不作为全身帧替换。
2. 实际点击进入 responding，400 毫秒后退出。快速连点后回到 active，没有队列积压；修复旧计时器可能提前终止新一次反馈的问题。
3. 采样呼吸变换从 matrix(1.00001,0,0,1.00001,0,0) 变为 matrix(1.00412,0,0,1.00619,0,0)，自然循环有效。
4. 等待自然眨眼事件，确认 blinking 出现后自行撤下，不通过手动强制状态替代此时序检查。
5. 真实鼠标按下并移动超过阈值，测试桩收到一次 `plugin:window|start_dragging`，label=pet；松开后没有 responding。此项仅证明前端分流及命令参数，不等于原生窗口已被拖动。
6. 仿真 prefers-reduced-motion=reduce 后 active 撤下，document.getAnimations() 为 0，点击不触发点头；恢复偏好后重新 active。
7. 派发 blur/focus 生命周期事件验证停止/恢复，blur 后动画数为 0。这是页面事件验证，不是 OS 遮挡/最小化验证。
8. 使用 ego-browser 自带 Node 运行 `check-drag.mjs`：最小权限与前端入口检查通过，无新增 Node 安装。

结果数据见 `browser-checks.json`；睁眼截图 `open.png`，最终闭眼截图 `closed-fixed.png`，深色背景截图 `closed-dark.png`。

## 发现并修复的问题

第一版把眼部遮罩绑定到可变宽高比容器，非正方形视口下眼睛错位并露出参考图背景（历史失败截图 `closed.png`）。改为不超过 200×200 的正方形画布，再次检查白色、深色背景，闭眼位置及角色轮廓正确。最终画布留 6 像素安全边距，轻呼吸/点头不挤出窗口。

## 仍需原生验证

### 空框问题：原生 CSS 被 CSP 拒绝

用户报告只看到框后，在原生页面加载完成时添加仅输出数值和布尔值的诊断。修复前：图片 decode 成功、naturalWidth=1254，但实际布局 1254×1254，style_loaded=false。旧 CSP `style-src 'unsafe-inline'` 不允许外置 pet.css；file 页面没有该 CSP，因此先前 ego-browser 测试未发现此问题。

修复为 `style-src 'self'`，保留同源限制并去掉方框式焦点描边。最终 Release 构建 11.94 秒通过，重新封装 `.app`、关闭旧实例并启动后，WKWebView 实测：ready=true、width=188、height=188、image_width=1254、style_loaded=true，页面加载 Finished。这是实际原生布局证据，已证明样式加载与原图裁切故障被修复；不等于已完成所有动画和双平台验收。`check-drag.mjs` 补充外置样式与同源 CSP 回归检查。

### 不可见启动修复续验

用户多次反馈看不到后，改用 `package-macos-preview.py` 生成的本机 `.app`，由 LaunchServices 独立启动。新增仅包含窗口几何、可见性及页面阶段的诊断日志。初次 center 得到物理坐标 (1512,-1674)；修复为明确使用 primary_monitor 计算中心后，新实例 PID 61708、父进程 1，日志为 visible=true、位置 (1510,856)、内部尺寸 400×402 物理像素，页面 Started/Finished。Release 再次构建通过，耗时 11.46 秒。逻辑窗口配置仍为 200×200。

系统辅助功能拒绝外部定位窗口；CoreGraphics 查询仍返回空列表，因此不把这些 API 当作截图或视觉成功证据。未要求用户为本次修复开放辅助功能权限，窗口定位在 Tauri 内部完成。`.app` 为本机预览包装，不是签名发布产物。

最终 `cargo build --release --offline --locked --manifest-path spikes/desktop-foundation/src-tauri/Cargo.toml` 通过，耗时 12.29 秒；直接启动 macOS GUI 后确认 PID 58221 仍在运行，启动阶段未输出错误。`git diff --check` 通过。进程存在不代替下述原生交互验收。

ego-browser 使用 Chromium，实际 macOS 桌宠使用 WKWebView；两者不等同。尚需分别在 Windows/macOS 验证原生拖动后继续点击、窗口切换、真实透明背景及减少动态效果。Windows 本批未编译或运行，不能宣称双平台完美。视觉检查通过也不证明全部设备和缩放比例无缺陷。
