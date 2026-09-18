# CU-S2 原生 WorkRef Adapter 独立 Verification Goal

日期：2026-09-16。关联 Story CU-S2、Accepted AD-CU-03 与 `openspec/changes/cu-s2-native-work-focus/`。Windows按用户要求暂缓。

## 结果

PASS（macOS生产 WorkRef Adapter 子范围）。Application已定义完整执行身份WorkRef、稳定失败分类与WorkFocusPort；捕获只接受observed attempt，定位调用只接受当前takeover控制已stopped且完整attempt/Worker/host身份一致。工作区36项Rust测试通过，包含旧Worker引用拒绝与停止后takeover门禁；架构关联检查和`git diff --check`通过。

macOS实现作为静态原生对象直接链接进Rust Adapter，不启动辅助App、脚本或常驻sidecar。真实隔离窗口验证正常前置、最小化恢复、原几何保持；同名同frame诱饵产生双侧不唯一时拒绝且诱饵保持前台；目标关闭后拒绝；引用释放后拒绝。引用绑定真实进程启动秒/微秒，定位前重新核对权限、进程、WindowServer与保留AX对象，成功后重新读取Frontmost、FocusedWindow、最小化及几何。[结构化证据](../../../apps/desktop/evidence/work-focus-adapter-20260916/result.json)。

## 完成边界

本Goal只证明CU Adapter可安全捕获和前置工作引用。TM-S3尚未持久化“正在定位/成功/失败”事务，也未把停止确认后的控制编排到该Port；任务卡片仍只显示“正在停止”。Recording未启动，Windows未验证，CU-S2保持verifying，不Archive。
