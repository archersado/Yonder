# DS-S1 启动与托盘恢复延迟独立 Verification Goal

Story：DS-S1；OpenSpec：e0-validate-desktop-foundation；日期：2026-09-13。

依据：ARCHITECTURE-SPINE「非功能与发布」、增量规格「性能证据」「托盘找回与退出」。本轮只增加测量工具，不修改桌宠实现。

状态：取得本机单次启动和托盘几何恢复证据；不视为完整性能 Goal 通过。

## 环境

macOS 26.5.1（25F80），arm64。沿用 runtime-assets 验证的 Release 预览包，二进制 SHA-256：`d7b4d19302a3668420e051860e00693687dd425998938856414f7db4ab1d0ba1`。未清理系统缓存，未重启操作系统。

## 托盘恢复

check-wake-visible.swift 在 AXPress“显示小龙”前读取单调时钟，每10ms读取原生窗口几何，最多等待3秒；捕获200×200后记录耗时。之后再等待1秒/2秒取得窗口与屏幕区域截图。原工具 awake-1s/awake-3s 文件名保留，本轮实际等待起点为几何恢复，不是 AXPress。

命令：`/private/tmp/yonda-check-wake-visible 15225 /Users/archersado/workspace/Yonder/spikes/desktop-foundation/evidence/tray-latency-20260913`。

工具退出0。右侧休眠56×112经托盘恢复200×200，单次几何恢复 **51.858ms**，同一窗口23237。人工核对两张屏幕区域截图，小龙完整、姿态变化。该指标包含 AX 调用与窗口查询开销，受10ms轮询精度限制；不是精确首帧时间，也不是架构所指“窗口变化至状态更新”的核心事件延迟，不能据此关闭300ms状态更新门禁。

## 新进程启动

先结束 PID 15225，再由 measure-startup-macos.py 启动预览包内可执行文件，保持新 PID 32717 运行。脚本拒绝覆盖证据或在已有同名进程时启动副本；使用单调时钟及20ms轮询，10秒内未就绪或进程提前退出即失败。

命令：`python3 spikes/desktop-foundation/measure-startup-macos.py spikes/desktop-foundation/evidence/startup-20260913.json`。

从进程创建前到既有渲染诊断报告 **1989.999ms**。报告 ready/style_loaded/script_ready 均为true，image_width=600；14秒原生诊断显示 breathing=true、blinks=3、reduced=false，即使 page_hidden=true 仍有活动。证据在 startup-20260913.json 和同名 .log。报告字段后续改为精确分隔匹配，并使用该真实日志及 ready=false/script_ready=true 反例检查，防止将 script_ready 子串当作图片 ready。

随后独立原生探针确认新窗口24408为200×200、应用未隐藏、onscreen=true，截图 startup-20260913.png 为400×400像素；人工查看透明小龙完整。该截图发生于测量之后，不作为1.99秒时已经呈现首帧的证据。

既有页面诊断主动等待300ms，并等待图片decode，因此这是到诊断报告的上界，不扣除等待推测首帧。包内可执行文件直接启动也不等于用户通过 LaunchServices 点击启动。仅一次新进程样本，不是冷缓存启动，也不声称p95或稳态分布。

## 结论与缺口

本轮取得可复现工具及原始计时，未用固定1秒截图等待冒充恢复延迟。托盘几何恢复、启动渲染就绪和事后视觉检查均成功。架构“托盘至可用低于3秒”及“窗口变化至状态更新低于300ms”的完整语义仍需相应端到端观测，不能以本轮两个较小数字直接判定通过。

Windows当前构建、冷启动/重复样本、宿主实际状态事件链路以及其他既定门禁仍缺证据。DS-S1 保持 design-review，AD-E0-01不转Accepted，不 Archive。

后续退出验证发现PID32717未归属独立Yonda系统资源组，正常.app启动后才取得该归属，详见 verification-exit-coalition-macos.md。1.990秒记录仍仅表示直接可执行文件启动，不能替代正常LaunchServices启动证据。
