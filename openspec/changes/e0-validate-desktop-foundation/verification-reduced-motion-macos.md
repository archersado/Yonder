# DS-S1 减少动态效果原生 Verification Goal

Story：DS-S1；OpenSpec：e0-validate-desktop-foundation；日期：2026-09-13。

依据：基础动态交互与闲置停靠/唤醒的减少动态效果条件。状态：开启/关闭后的启动采样及2026-09-14同进程停动/恢复采样通过，原始设置已恢复；减少效果下过渡等余项尚未完成，完整 Goal 尚未通过。

## 当前证据

本机SDK NSAccessibility.h提供只读NSWorkspace.accessibilityDisplayShouldReduceMotion和变更通知；产品页面通过prefers-reduced-motion及change事件控制动画。

系统原生API实测当前值为false。`open -b com.apple.systempreferences`返回0，但NSRunningApplication没有可操作的系统设置主进程；pgrep仅见System Settings.app内csimporter插件。未将打开命令成功当作设置窗口可用，也未修改系统偏好。

已请用户在系统设置→辅助功能→显示开启“减弱动态效果”，准备后回复。原始值false已记录；因自动化无法操作设置页，验证后需用户关闭该开关，再用同一只读API确认false。不能声称工具已具备自动恢复能力。

## 待执行验证

1. 原生API确认true后，再检查当前WKWebView是否遵循系统偏好，不用CDP模拟值冒充原生设置。
2. 复用已有report_pet_motion诊断，使用正常LaunchServices启动并将固定数字/布尔诊断输出到验证日志。确认reduced=true、无呼吸变换及眨眼，结合原生画面；启动静态检查不能替代运行中切换响应。
3. 记录运行中切换后的停动、直接休眠/唤醒及恢复默认后的动画恢复；每项以实际观测为准，不因CSS/JS源码分支存在而通过。
4. 恢复原始false并用原生API复核。缺任一步只记录已完成子项。

此前前置准备轮次未重启桌宠、未改变设置。Windows仍由用户暂缓；本Goal与DS-S1总Goal均未通过。

## 用户开启后的原生验证

用户回复“已开启”后，`/private/tmp/yonda-settings-search` 在启动前、采样后均通过 NSWorkspace 读取到 `native_reduce_motion=true`。未使用浏览器偏好模拟。

通过托盘 AXPress 正常退出 PID 9322，确认旧进程退出后，使用 `open -n --stdout … --stderr … Yonda.app` 经 LaunchServices 启动当前应用，PID 26540。没有修改产品代码或重新编译。

二进制 SHA-256：`fa06f9f3c8e6159c116438d1eb59e4b409f0c313097d05f725644807f40b5e3f`。

证据：`spikes/desktop-foundation/evidence/reduced-motion-enabled-20260913.stderr.log`。

```text
桌宠渲染：ready=true, width=188, height=188, image_width=600, style_loaded=true, script_ready=true, page_hidden=false
桌宠动画采样（14秒）：breathing=false, blinks=0, reduced=true
```

通过范围：真实系统设置开启时，正常启动的 WKWebView 识别该设置，图片与脚本正常加载；14 秒内未观察到眨眼，呼吸元素采样首尾变换一致。呼吸诊断只比较首尾，不等同于逐帧检测；本次没有追加同步视频证据。

截至开启验证结束：运行中切换后的停动与恢复、减少动态效果下的直接停靠/唤醒过渡未覆盖；当时系统设置为 true，尚未恢复。后续结果如下。

## 用户关闭后的恢复验证

用户回复“已关闭”后，原生 NSWorkspace API 读取 `native_reduce_motion=false`，对照测试结束后再次确认 false，原始系统设置已恢复。

未重启的 PID 26540 经既有托盘入口唤醒，窗口 28796、200×201、onscreen=true、app_hidden=false。使用系统 screencapture 录制该窗口，证据为 `spikes/desktop-foundation/evidence/reduced-motion-restored-live-20260913.mov`。已有 AVFoundation 脚本 `/private/tmp/yonder-inspect-video.swift` 提取 219 个样本，视频时长 11.985 秒，首秒后各帧相对基准最大仅 18 个颜色通道差值超过 15；基准图显示完整小龙。帧与统计保存在同名 `.frames/` 目录。没有取得动画已恢复的证据，此项不通过；窗口视频可能存在采集限制，尚不能仅据此确认 WebKit 通知失效为根因。

随后正常退出该进程，经 LaunchServices 重启同一应用，未修改代码。原生日志 `spikes/desktop-foundation/evidence/reduced-motion-restored-startup-20260913.stderr.log` 显示渲染 ready=true，14 秒采样 `breathing=true, blinks=3, reduced=false`。因此关闭设置后的启动动画恢复通过；此结果不能替代不重启恢复通过。日志 page_hidden=true，既有实现使用原生窗口可见性；未据此误判窗口动画停止。

下一步需定位运行中恢复的可见性、媒体查询值及通知链路，并排除窗口录制冻结。减少动态效果下直接停靠/唤醒、运行中开启时停止等余项仍保留；不 Archive。

## 2026-09-14 对照与诊断增量

同一旧二进制的正常启动进程 PID 38990 经托盘唤醒后，使用相同 `screencapture -x -v -V 12 -l` 方法录制窗口 29298。证据 `spikes/desktop-foundation/evidence/reduced-motion-control-20260914.mov` 及同名 `.frames/`：11.9917 秒、219 样本，最大变化 104548 个通道；抽帧可见完整小龙姿态变化。正常对照能录到动作，昨日仅 18 通道变化的样本不能作为动画已恢复的证据；仍不据此断言具体根因。

按 Story 设计补充已有诊断的运行中触发能力：启动、托盘显示及媒体查询 change 启动 14 秒观察，重复触发合并，结束断开；观察期间 transform 变化，不再仅比较首尾。只返回既有 breathing/blinks/reduced，不改变动画控制，不新增权限或协议。采样跨越偏好切换时 reduced 仅代表结束时状态，不能将整段都视为该偏好下的稳定样本。

自动检查 `node spikes/desktop-foundation/check-motion-diagnostic.mjs` 通过，覆盖首尾回归、重复触发合并、静态减少效果样本、观察器清理。Release 离线锁定构建及架构关联检查通过。新验证包 SHA-256：`9ee83d06b71615125aac0b208559ed64a3ce8d9ca8db787df2badd7574988402`，已正常 LaunchServices 启动，原生日志 `spikes/desktop-foundation/evidence/reduced-motion-diagnostic-20260914.stderr.log`。

本次未修改系统设置。运行中开关仍需用户在真实系统设置中操作后复核；新采样器检查通过不代表恢复缺陷已修复。Windows继续暂缓，完整Goal仍未通过。

## 2026-09-14 运行中开启实测

用户再次开启真实系统设置后，原生 API 在采样前后均确认 true。验证包 PID 56576 未重启；既有启动采样 `breathing=true, blinks=3, reduced=false` 后，日志新增媒体查询变化触发的 `breathing=false, blinks=0, reduced=true`。

随后通过既有托盘入口显示同一进程的小龙（窗口 30899，200×200，app_hidden=false、onscreen=true），触发独立 14 秒采样，结果仍为 `breathing=false, blinks=0, reduced=true`。原始日志继续保存在 `spikes/desktop-foundation/evidence/reduced-motion-diagnostic-20260914.stderr.log`；单帧 `spikes/desktop-foundation/evidence/reduced-motion-enabled-live-20260914.png` 可见完整展开小龙，只作为形态证据，不代替动画观察。

通过范围：本轮运行中开启偏好已传递至页面，并在展开状态停止呼吸样式变化与眨眼；未测量从系统切换到停动的精确延迟。关闭后的同进程恢复仍待验证；当前设置为 true，需用户恢复原始 false 后继续读取变化采样，不重启替代恢复测试。直接停靠/唤醒过渡与 Windows 门禁仍保留。

## 2026-09-14 同进程关闭与恢复实测

用户随后回复“已关闭”。原生 API 在本轮开始与展开采样结束后均返回 `native_reduce_motion=false`，原始设置已恢复。PID 始终为 56576，未重启、未重新编译、未修改动画控制。

读取既有运行中日志可见右侧停靠后新增 `breathing=false, blinks=3, reduced=false`：关闭偏好已传入页面，边缘双眼恢复眨眼；停靠时不应呼吸，因此此处 breathing=false 符合设计，不判为恢复失败。

通过已有托盘入口展开同一窗口 30899（200×200，app_hidden=false、onscreen=true），下一轮独立14秒诊断为 `breathing=true, blinks=3, reduced=false`。通过本轮“运行中关闭后停靠双眼恢复眨眼、再唤醒后恢复呼吸与眨眼”路径；不能扩大为关闭时仍展开的路径或所有平台均通过。

原始日志：`spikes/desktop-foundation/evidence/reduced-motion-diagnostic-20260914.stderr.log`。展开后的窗口录屏：`spikes/desktop-foundation/evidence/reduced-motion-restored-live-20260914.mov`。此前2026-09-13未通过样本保留，不声称已定位或修复其根因；当前成功样本来自增加诊断后的验证版，动画控制逻辑没有变更。减少动态效果下直接停靠/唤醒过渡仍待验证，不Archive。

录屏核对完成：12.0667秒、221样本，最大变化109929个颜色通道，抽帧可见完整小龙的身体倾斜与尾部姿态变化；统计及基准/最大变化帧保存在同名`.frames/`目录。真实窗口画面支持展开后动作恢复，不只依赖DOM采样。原生采样与视频结合通过本轮路径，不推导精确切换延迟或所有动作质量均已验收。
