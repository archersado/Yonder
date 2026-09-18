# CU-S3 后台原生动作 macOS 独立 Verification Goal

日期：2026-09-18  
状态：PASS（仅macOS `NSWorkspace` 后台启动子范围）  
关联：CU-S3、AD-CU-06、`cu-s3-background-native-action-spike`

## 验证目标

隔离验证公开系统API能够在不激活目标应用、不启动trycua的情况下执行后台动作；动作后Observe有效，不支持动作没有隐式CUA/Command/脚本兜底，并能形成Task Space统一时间线所需的最小事实。

## 结果

执行`spikes/background-native-action/run-macos.sh evidence/macos-20260918/result.json`退出0。结构化证据显示：

- `foreground_unchanged=true`、`activates=false`；目标未抢占当前前台。
- `observe_ready=true`；fixture就绪标记提供动作后Observe。
- `new_trycua_workers=0`、`unsupported_no_fallback=true`；没有启动CUA或其他兜底。
- `task_space_facts`按1～4依次包含步骤声明、attempt开始、Observe和结果，可映射到统一任务时间线。
- `cleanup_terminated=true`、`cleanup_forced=false`；独立复核时无fixture残留。

随后使用同一探针验证真实系统应用`/System/Applications/Calculator.app`，结果位于`spikes/background-native-action/evidence/macos-real-app-20260918/result.json`：应用此前未运行，`target_kind=real-application`、前台保持、Observe、零新增trycua Worker、无隐式兜底和正常清理全部通过。fixture回归随后再次通过，两类进程均无残留。

App Intents公开SDK面探针`inspect-app-intents-macos.py`退出0，证据位于`evidence/app-intents-macos-20260918/result.json`：计算器metadata动作数为1，具体类型`perform/callAsFunction`和Donation API存在，跨应用通用执行入口数为0；Shortcuts CLI只提供Command入口。结论为通用App Intent Adapter路线淘汰，不影响具体系统API或应用SDK候选。

EventKit探针保留失败与最终证据：直接执行bundle内二进制无法建立授权会话且没有保存；旧探针在后台步骤内部请求权限的结果已标记`result-permission-request-inside-background-rejected.json`，即使前台保持也不接受。最终同一临时App先运行`authorize`：`not-determined→full-access`、`saved=false`，输出permission-required/permission-observed两条事实；再运行`background`：启动时full access，创建、identifier Observe、删除清理与前台保持通过，输出步骤声明、attempt开始、Observe和结果四条事实。证据位于`spikes/background-native-action/evidence/eventkit-reminder-macos-20260918/`，不含用户提醒或测试项内容，复核时无探针进程残留。

既有[SDK隔离AX输入Verification Goal](../e0-compare-cua-drivers/verification-input-macos.md)证明AX动作仍由trycua执行并在动作后Observe，本Spike未复制该执行链。

## 失败保留

首次fixture未初始化`NSApplication`，进程已写Observe标记但LaunchServices完成回调未返回，见`result-initial-fixture-lifecycle-fail.json`。修正生命周期后的首次Goal复核又发现只调用`terminate`未等待退出，见`cleanup-initial-fail.json`；回到实现阶段增加退出确认后重跑通过。

## 边界

本Goal已淘汰通用App Intent桥接并接受已授权EventKit临时提醒子范围，但不验证可链接的自有Intent、其他应用内业务动作、真实第三方应用SDK、产品协议/SQLite/Task Space UI或Windows。AD-CU-06保持Proposed，CU-S3不Archive，产品Gateway不得据此开放后台原生动作。
