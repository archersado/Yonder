# Windows 输入钩子只读诊断（2026-11-05）

## 实现与运行

隔离 Spike Host 新增专用线程安装 `WH_MOUSE_LL` 和 `WH_KEYBOARD_LL`；回调只累计 `generation`、未标记注入事件计数和非注入事件计数，不记录键值、文本、坐标或目标应用。输入桌面变化、线程退出和计数溢出会令诊断线程状态失效。原生 Worker 和产品 Adapter 未接入此诊断，未发送桌面输入。

Windows x86_64/MSVC 上多次运行 `native-input-observer-probe -- 1000`，线程安装与消息循环在各自 1 秒观察窗内均正常返回，generation 不回退。早期聚合输出观察到 11、14 个非 injected 事件，收紧诊断字段后的另一轮输出为 0；各轮 injected 计数均为 0。没有独立的人类输入 oracle，不能把这些计数记为 `INPUT-01` 通过。

## 关键限制

[微软的鼠标低级钩子文档](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelmouseproc)与[键盘低级钩子文档](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)说明，钩子处理超时后系统可静默移除钩子，应用无法获知。因此 `diagnostic_thread_running=true` 仅表示线程仍在运行，不能证明两个钩子仍覆盖输入。探针明确输出 `admission_ready=false`、`hook_health_proven=false`、`driver_attribution_proven=false`、`human_input_sample_proven=false`；不得把它接入派发准入或作为用户输入零漏分证据。

现有固定上游 `SendInput` 路径也没有 attempt/Worker 归属标记，故 `INPUT-02` 仍未验证。原生 dispatch 继续 `unsupported/not_delivered`。
