# SDK隔离原生AX输入与动作间停止独立 Verification Goal

2026-09-14；状态PASS（SDK隔离原生输入/Observe与动作间停止）。实现后建立，关联CU-S1 STOP-IN02–03、AD-CU-01/AD-E0-02与TAKEOVER-STOP-PLAN。使用临时Swift可执行输入窗及input-macos-probe，不打包上游或额外产品App。

验收：SDK权限足够且唯一PID/窗口/AX字段确认；后台固定标记输入，SDK无截图Observe与目标匹配；SDK shutdown后第二输入拒绝，新SDK Observe原标记仍在，750ms内目标匹配/长度稳定。结果不含原始内容或AX正文，未调用Recording/剪贴板/像素兜底。不满足即返回实施阶段。只验证动作间关闭，不代表执行中原生动作中断、正式Yonder宿主权限或Windows完成，完整Story不Archive。

首轮失败保留在spikes/cua-driver-comparison/evidence/sdk-input-macos-20260914/result.json：临时Swift测试窗首次编译尚未完成时启动探针，fixture_spawn_error=true、input_dispatched=false。回到实施阶段，待构建明确退出0后才重新进入验证，不把启动失败改写为通过。

## 构建与原生结果

首次新缓存构建在编译系统Swift模块；取样证实仍在模块构建，随后找到项目既有/private/tmp/yonder-swift-module-cache，停止重复构建进程并复用该缓存。`swiftc spikes/cua-driver-comparison/input-fixture-macos.swift -module-cache-path /private/tmp/yonder-swift-module-cache -o /private/tmp/yonda-sdk-input-fixture-ready`明确退出0后才重跑。

原生命令：`node spikes/cua-driver-comparison/input-macos-probe.mjs /private/tmp/yonda-sdk-input-fixture-ready spikes/cua-driver-comparison/evidence/sdk-input-macos-ready-20260914/result.json`，退出0。

- STOP-IN02：target_unique=true、input_error=false、sdk_observe_matches=true、fixture_matches=true，固定测试标记长度18；先确认唯一PID/窗口/AX token，后台请求输入后立即无截图Observe，两条独立状态证据相符。
- STOP-IN03：after_shutdown_input_rejected=true、stopped_sdk_observe_matches=true、stopped_fixture_stable=true，750ms观察期目标长度/匹配稳定，passed=true。
- 记录约束：recording_started=false、screenshot_requested=false；结果未保存AX正文/输入文本，无显式剪贴板工具或前台像素兜底。临时测试窗已关闭，真实任务/数据库未修改。

native_inflight_stop_verified=false、yonder_host_verified=false；测试在动作完成后关闭SDK，不能证明执行中动作中断或正式Yonder权限责任链。Windows暂停，完整CU-S1/CU-S2/TM-S3不Done/Archive。稳定仅为本样本750ms观察期证据，不推断无限时长无输入。
