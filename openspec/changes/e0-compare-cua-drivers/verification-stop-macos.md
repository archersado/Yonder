# macOS Driver停止补充独立 Verification Goal

2026-09-14，关联CU-S1 STOP-READ01–04、AD-E0-02/AD-TM-03及TAKEOVER-STOP-PLAN。实现后独立验证，只读生命周期目标PASS；实际输入停止目标未通过验证，完整Story不Done、不Archive。目标为固定trycua0.25.0只读生命周期样本及异常退出恢复；结果只含分类/布尔，不输出应用、窗口、正文或完整Payload。Windows新增验证暂缓。

首次npm依赖安装因沙箱连接代理EPERM失败，按锁定依赖原生重新安装成功。首次生命周期探针三组均初始化失败；模块导入成功，metadata初始化在沙箱内NSPasteboard对象返回NULL，报DriverError.Protocol。保留spikes/cua-driver-comparison/evidence/stop-macos-20260914/result.json首次失败，不将沙箱故障改称Driver停止通过。

原生环境重跑通过，新增语义断言后再次验证退出码0，readonly_lifecycle_passed=true。最终证据：spikes/cua-driver-comparison/evidence/stop-macos-native-asserted-20260914/result.json；此前原生结果保留在stop-macos-native-20260914/result.json。

- STOP-READ01：未知工具返回is_error=true；预取消只读调用拒绝，未把Promise完成当成动作停止。
- STOP-READ02：shutdown成功；关闭后调用拒绝；重复shutdown成功。
- STOP-READ03：异常子进程按预期退出23，新实例list_apps成功。
- STOP-READ04：结果仅布尔/错误分类；input_dispatched=false、recording_started=false、native_input_stop_verified=false。

可复跑：`node spikes/cua-driver-comparison/stop-macos-probe.mjs <结构化结果路径>`（需要原生macOS系统访问）。探针检查上述语义，任一不满足退出非零。未调用clipboard_read、键鼠或Recording；Driver内部初始化系统组件不作为读取用户内容的授权。只读生命周期通过不是原生动作停止证明，CU-S2/TM-S3/RC-S1接管与Recording门禁继续保留。
