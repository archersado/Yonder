# DS-S1 原生托盘退出与进程清理独立 Verification Goal

Story：DS-S1；OpenSpec：e0-validate-desktop-foundation；日期：2026-09-13。

依据：增量规格「启动验证程序」「托盘找回与退出」中正常退出不残留受管进程。状态：本机当前预览包的托盘退出及已归属宿主/WebKit清理通过；不代表Windows或未来Worker清理通过。

## 方法

check-exit-macos.py 复用 measure-app-macos.py 的 resource coalition 解析，按系统组名、ID和bundle ID确认宿主及WebKit归属；仅在宿主确为 yonder-desktop-spike 且存在同组WebContent时继续。awake-probe-macos.swift 增加quit模式，通过现有AX菜单“退出 Yonda”执行正常退出，不发送终止信号。

之后最多30秒、每500ms检查已确认进程编号是否全部消失。30秒是探针超时，不是新增产品性能要求；约0.54秒是一次检查观测上界，不是精确退出耗时。若PID被复用会保守判未清理，不因同名其他应用进程消失而误报成功。统计仅覆盖退出前已确认成员，不声称覆盖未来执行Worker。

## 实际结果

通过系统 `open -n …/Yonda.app` 启动PID61767。资源组28853，bundle ID com.yonder.e0-spike；确认宿主61767、GPU61770、Networking61771、WebContent61772。

`python3 spikes/desktop-foundation/check-exit-macos.py 61767 spikes/desktop-foundation/evidence/exit-coalition-20260913.json /private/tmp/yonda-awake-probe` 退出0。AX菜单调用返回0；0.540秒后的进程检查无上述四个PID残留。证据见命令中的JSON。工具真实成功运行同时验证资源组解析、菜单调用及清理断言。

随后通过正常.app入口重启，pgrep确认唯一宿主PID62176。未修改产品二进制。

## 被正确拒绝的前置与历史修正

首次用PID32717尝试检查时，系统资源组不属于独立Yonda bundle，工具在发出退出动作前以“非目标应用”停止。该进程来自上轮 measure-startup-macos.py 直接启动包内可执行文件，不能据此把继承组里的其他进程当作Yonda进程。

随后通过限定bundle的托盘探针正常退出32717，pgrep确认无宿主，改用LaunchServices启动再测。没有绕过资源组断言，也没有杀死同组其他应用。

这补充了 verification-latency-macos.md 的启动方法限制：1.990秒只代表直接可执行文件启动至诊断报告，不代表正常.app启动链路。该进程上人工眼睛唤醒反馈仍有效；自动化点击失败是否与启动上下文有关未证实，不作为本轮结论。

Windows由用户暂缓。DS-S1其余门禁及AD-E0-01评审仍保留，不Archive。
