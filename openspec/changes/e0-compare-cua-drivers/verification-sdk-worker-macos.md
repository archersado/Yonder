# 仅SDK自管Worker独立 Verification Goal

2026-09-14；状态PASS（只读SDK子进程生命周期）。实现后独立建立，关联CU-S1 STOP-SDK01–04、Accepted AD-CU-01/AD-E0-02与增量规格。探针只导入trycua0.25.0 SDK，无上游App或Driver可执行文件。

检查真实macOS子进程metadata就绪→宿主SIGTERM→确认exit；第二子进程宿主继承IPC断开→SDK shutdown→退出0；随后新实例list_apps成功。任一超时、错误、退出不符合或恢复失败，退出非零并返回实施阶段。以前只读生命周期/Abort回归同时运行，不采集正文、输入、截图或Recording。

Goal仅证明SDK自管子进程生命周期；原生动作停止、动作后Observe、宿主崩溃残留、生产私有stdio协议、Node打包/宿主权限与Windows仍待验证。接管按钮仍禁用，不Archive完整Story。

## 原生结果

命令：`node spikes/cua-driver-comparison/stop-macos-probe.mjs spikes/cua-driver-comparison/evidence/stop-macos-sdk-worker-20260914/result.json`。退出0，sdk_worker_lifecycle_passed=true，既有readonly_lifecycle_passed/submitted_lifecycle_passed也均true。

- STOP-SDK01：两子进程均metadata就绪，ready=true。
- STOP-SDK02：监督终止子进程确认exit_signal=SIGTERM，completed=true、timeout=false。
- STOP-SDK03：停止/断连后新SDK实例list_apps成功且is_error=false。
- STOP-SDK04：宿主私有IPC断开后，子进程SDK shutdown并退出0，completed=true、timeout=false。

本次并发shutdown样本的读调用成功，与此前读拒绝均为有效竞争结果；探针不强求某种竞态顺序。进程停止通过不能代表执行中原生输入停止，native_input_stop_verified=false，input_dispatched=false、recording_started=false。监督使用测试Node父进程，不冒充正式Yonder权限责任链已验证。
