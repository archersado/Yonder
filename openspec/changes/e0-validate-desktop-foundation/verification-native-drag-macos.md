# DS-S1 原生拖动独立验证记录

Story: DS-S1
OpenSpec: e0-validate-desktop-foundation
日期：2026-09-23

## 验证目标

在真实 macOS 进程上按住小龙左键拖动，窗口位移不少于 50 点，仍位于当前屏幕工作区内；拖动不打开任务总览，且窗口 ID 保持不变。Windows 证据按用户要求暂缓。

## 本轮结果

命令：`swift apps/desktop/check-dragon-native-drag-macos.swift <Yonda-PID> <证据目录>`。

本轮在执行前读取 `CGSessionCopyCurrentDictionary()`，`CGSSessionScreenIsLocked=1`。验证工具因此写出 `native-result.json` 的 `blocked_by:"screen_locked"`、`passed:false` 并以 9 退出，不发送鼠标事件，也不把位移 0 判定为产品失败。

结构化证据：[`native-result.json`](../../../apps/desktop/evidence/dragon-native-drag-20260923-locked/native-result.json)。

阻塞检查结束后，测试专用 Yonda 进程已退出，未留下本 Story 的常驻进程。

此前多组 `movement_points:0` 证据是在同一锁屏状态下产生的，只能证明验证环境不可用，不能证明或否定原生拖动实现。

## 静态检查

- `/Users/archersado/.cargo/bin/cargo test -p yonder-desktop --lib`：4 通过。
- `/Users/archersado/.cargo/bin/cargo test -p yonder-desktop --bin yonder-desktop`：2 通过；四边停靠目标、停靠尺寸与恢复位置为精确断言。
- `/Users/archersado/.cargo/bin/cargo test --workspace`：55 通过。
- `openspec validate e0-validate-desktop-foundation`：通过。
- `node --check apps/desktop/ui/pet.js`：通过。
- `python3 -m json.tool apps/desktop/tauri.conf.json`：通过。
- `swift -frontend -parse apps/desktop/check-dragon-native-drag-macos.swift`：通过。

## 剩余门禁

解锁后重新启动新的 Yonda 进程并运行同一脚本，保存前后截图和通过 JSON；只有 `native_drag_moved_window`、`window_within_screen`、`task_menu_not_opened`、`pet_window_visible` 同时为真才关闭本项。Windows 证据继续保留。

## 2026-09-25 验证工具修订与复验

复核发现旧脚本无条件使用 `NSScreen.main.visibleFrame` 计算目标；当 Yonda 位于副屏或负坐标显示器时，会把跨屏移动误当成同屏拖动验证，而且验证结束后不会恢复用户窗口。工具现改为从 `CGGetActiveDisplayList` 取得与原窗口交叠面积最大的实际显示器，在同一 Quartz 坐标系内选择安全目标；拖动序列使用 `.leftMouseDragged` 和 HID system event source，取证后执行第二次真实拖动恢复原位，并要求窗口 ID 不变且原点误差不超过 2 点。

`--self-test` 的负坐标双屏样本通过，证明选择左侧实际显示器且目标仍在该显示器安全区内。本机随后对正式 Yonder PID 执行复验时，`CGSSessionScreenIsLocked` 仍为 1；脚本在构造事件源和发送任何输入前以 9 退出。当前结果只证明验证工具正确拒绝锁屏环境，不构成原生拖动通过证据。结构化记录见 `apps/desktop/evidence/dragon-native-drag-harness-20260925/result.json`。
