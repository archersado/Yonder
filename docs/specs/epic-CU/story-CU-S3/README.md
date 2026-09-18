# CU-S3 后台原生动作与显式前台切换

Story: CU-S3  
Epic: CU  
Status: design-review  
OpenSpec: cu-s3-background-native-action-spike

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

本 Story 记录 2026-09-18 用户确认的执行边界：应用公开的原生 API、App Intent 或 SDK 可作为后台原生动作；AX/UI 与键鼠动作仍由 CUA Driver 执行。两类动作必须进入同一任务生命周期和 Task Space 时间线，但不得在失败时互相隐式降级。

当前仅批准限时 macOS Spike。Windows 按用户决定暂缓；双平台证据和 ADR 接受前不得接入产品 Gateway。

## OpenSpec 与验证

[OpenSpec Change](../../../../openspec/changes/cu-s3-background-native-action-spike/)只验证技术边界，不新增产品协议、持久化或通用 Adapter。

2026-09-18：macOS `NSWorkspace`子范围独立Verification Goal PASS。后台fixture未抢前台、动作后Observe有效、未新增trycua Worker、不支持请求无隐式兜底，且输出可映射Task Space的顺序事实；临时进程正常退出。通用App Intent、真实第三方应用能力、产品接线与Windows仍未验证，Story保持design-review。

同日真实系统计算器对照PASS：应用此前未运行，非激活启动、前台保持、Observe、无CUA兜底及退出清理全部通过。该证据只把fixture结论扩展到真实应用后台启动，不证明应用内业务动作或App Intent。

App Intents公开SDK面探针确认计算器metadata可发现但不可被外部应用按标识通用执行；通用App Intent Adapter路线淘汰。后续后台业务动作只接受明确系统API、应用SDK或Yonder可编译链接的具体Intent类型。

EventKit真实副作用子范围PASS：首次权限获取会改变前台并按后台验收失败；权限已授权后，临时提醒创建、identifier Observe、删除清理和前台保持通过。产品必须先进入显式等待用户授权，授权完成后再由Agent提交新动作。

最终探针已拆成`authorize`与`background`两种模式：授权步骤不执行副作用；后台步骤启动时必须已full access，并输出可映射到Task Space的步骤声明、attempt、Observe和结果事实。后台内部请求权限的旧行为已淘汰。

2026-09-18门禁复核：macOS Spike范围已完成，当前不得进入产品接线。AD-CU-06明确要求双平台证据完成后才能开放Gateway，而Windows已按用户决定暂缓；本Story保持design-review，不创建NativeActionPort、协议或第二执行栈。
