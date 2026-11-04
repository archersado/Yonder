# 独立 Verification Goal：DS-S1 Windows 四边休眠剩余链路

日期：2026-10-29
Story：DS-S1
Goal：`01a15eda-f12a-7e32-bd6c-2d3336e17351`
结论：PASS（Windows 顶部/右侧停靠、点击/键盘恢复与交互重置子范围）

## 方法

验证脚本只启动并控制自己的 `yonder-desktop.exe` 进程；若发现同一路径已有进程则拒绝运行。脚本在主屏工作区移动该测试窗口，不激活移动操作；交互使用 Win32 原生主键点击和 Enter 键。每次鼠标注入前通过 `WindowFromPoint` 和根窗口句柄反查，必须命中本测试进程的 Yonda 窗口，否则立即失败。整个过程未调用 Yonder CUA Driver，也未运行 CUA Story。

本机显示缩放为150%。PowerShell 的窗口几何为 DPI 虚拟化后的逻辑像素，Rust 诊断为物理像素；两套记录分别保留，不相互混写：112×56对应168×84，56×112对应84×168，200×200对应300×300。

## 独立证据

- 清醒态原生点击后170.013秒，窗口仍为200×200；这排除了沿用启动时旧计时并提前停靠。点击后180.355秒进入顶部112×56，证明交互会重新开始三分钟计时。
- 顶部探头位于 `(1226,0)`、尺寸112×56；对旋转后可见眼睛区域派发一次原生主键点击，原生窗口恢复至原位置 `(1182,0)`、尺寸200×200。Rust同时记录物理尺寸从168×84恢复到300×300。
- 随后把恢复窗口移到主屏右边；179.839秒后进入右侧 `(2504,640)`、56×112，右边界精确等于工作区右边界2560。
- 对已聚焦的右侧探头派发一次原生 Enter，窗口恢复到 `(2360,596)`、200×200。Rust同时记录物理尺寸从84×168恢复到300×300。
- 进程未提前退出；脚本只终止自己启动的精确 PID，并清理精确临时目录。
- Windows `cargo test -p yonder-desktop --locked`：库8项、桌面入口4项，共12项通过，0项失败；仅有既有未使用导入和OpenSSL PDB链接警告。
- 架构与Story/OpenSpec关联检查通过；`openspec validate e0-validate-desktop-foundation --strict`、PowerShell语法检查和相关文件 `git diff --check` 均通过。
- 结构化结果：`apps/desktop/evidence/ds-s1-windows-20261029/edges-wake-result.json`。
- 可重复脚本：`apps/desktop/evidence/ds-s1-windows-20261029/verify-edges-wake.ps1`。

## 验证工具失败与修正

第一次样本把 DPI 虚拟窗口矩形中心直接作为顶部透明画面的点击点，停靠与计时已通过，但点击没有命中可见眼睛；记录保留在 `dpi-injection-failure-result.json`。第二次错误地假定 `SetCursorPos` 消费未虚拟化物理坐标，点击没有命中窗口，记录保留在 `dpi-scaling-assumption-failure-result.json`。

最终脚本保持 `GetWindowRect` 与 `SetCursorPos` 的同一 user32 虚拟坐标系，点击前用 `WindowFromPoint` 验证根窗口归属，并选择顶部旋转后可见眼睛区域。上述两次是验证方法失败，不是产品失败，也不作为通过样本。

## 判定边界

结合2026-10-25底边和左边证据，Windows 四边自动停靠尺寸与本批恢复入口已覆盖。本证据不替代 Windows 人工拖动、九态完整原生视觉/事件链路、性能、托盘完整生命周期或 macOS 剩余门禁；DS-S1 总 Verification Goal 仍未完成，不 Archive。本轮未执行或修改 CUA。
