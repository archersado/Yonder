# 独立 Verification Goal：DS-S1 Windows 原生拖动技术链路

日期：2026-10-29
Story：DS-S1
Goal：`01a15eda-f12a-7e32-bd6c-2d3336e17351`
结论：PASS（Windows 原生合成输入拖动技术子范围）

## 方法

验证器只启动并控制自己的 `yonder-desktop.exe`；发现同一路径已有进程时拒绝运行。窗口先无激活移到 `(640,400)`，再在200×200窗口的中心按下 Win32 主键，以15步移动光标。起点通过 `WindowFromPoint` 反查，必须归属本测试进程的 Yonda 根窗口。验证器观测到窗口已在按住期间发生位移后才发送一次释放，随后在不按键的情况下继续移动光标。

该方法真实经过 WebView 指针分流、`plugin:window|start_dragging` 和 Windows 原生窗口移动循环，但输入由 Win32 验证器生成，不冒充用户亲手操作，也不调用 Yonder CUA Driver。

## 结果

- 拖动前：`(640,400)`、200×200。
- 释放后：`(808,512)`、200×200；实际位移 `(168,112)`，窗口尺寸保持不变。
- 释放后把光标继续移动360×300，窗口仍为 `(808,512)`、200×200，证明释放停止了窗口跟随。
- 进程未提前退出；验证器只终止自身启动的精确 PID，并清理精确临时目录。
- 结构化结果：`apps/desktop/evidence/ds-s1-windows-20261029/native-drag-result.json`。
- 可重复脚本：`apps/desktop/evidence/ds-s1-windows-20261029/verify-native-drag.ps1`。

## 验证工具失败与修正

第一次样本已产生窗口位移，但验证器可能在 WebView 异步 `start_dragging` 真正进入 Windows 移动循环前发送释放，随后断言失败；原始记录保留在 `native-drag-release-race-failure-result.json`。最终脚本先观测按住期间窗口位置确已改变，再发送释放，消除验证器先释放、原生命令后启动的竞态。该首次结果判为验证方法失败，不判为产品失败。

## 判定边界

本证据通过 Windows 原生拖动技术链路、尺寸保持和释放停止三个子条件，但不把合成输入写成“用户人工拖动”。`verification-drag.md` 的用户亲手拖动确认、Windows 当前素材的原生视觉截图、macOS拖动以及DS-S1其他门禁仍未完成；Change不 Archive。本轮未执行或修改CUA。
