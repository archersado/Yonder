# SDK原生输入权限独立 Verification Goal

2026-09-14；状态PASS（当前测试SDK宿主权限只读检查）。实现后独立建立，关联CU-S1 STOP-IN01、Accepted AD-CU-01/AD-E0-02及增量规格。原生命令`node spikes/cua-driver-comparison/stop-macos-probe.mjs permissions`退出0，accessibility=true、screen_recording=true，input_dispatched/recording_started/permission_requested/yonder_host_verified均false。没有请求权限/截图/输入/Recording。当前测试宿主可以进入隔离AX样本，但不能叫作原生输入或正式Yonder权限通过。STOP-IN02–03由verification-input-macos独立验证，Windows暂停。
