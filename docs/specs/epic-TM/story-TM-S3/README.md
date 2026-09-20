# TM-S3 逐任务暂停取消与接管

Story: TM-S3
Epic: TM
Status: implementing
OpenSpec: tm-s3-cancel-retain-data
Increment: [tm-s3-step-boundary-stop](../../../../openspec/changes/tm-s3-step-boundary-stop/proposal.md)
Control Increment: [tm-s3-control-request](../../../../openspec/changes/tm-s3-control-request/proposal.md)
Focus Increment: [tm-s3-takeover-work-focus](../../../../openspec/changes/tm-s3-takeover-work-focus/proposal.md)

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

控制协议、停止确认和 macOS 当前 Space 接管定位已通过增量验证；跨 Space/多显示器、Windows 与 Recording 仍待实施。

## OpenSpec 与验证

Accepted AD-TM-04审阅通过，首批仅created取消；openspec/changes/tm-s3-pending-cancel/。执行过的任务仍需TM-S2/CU停止前置，不以本子范围完成完整Story。

首批created取消与macOS原生按钮验证PASS，记录[独立Verification Goal](../../../../openspec/changes/tm-s3-pending-cancel/verification-pending-cancel.md)。完整执行中暂停/取消/接管仍待停止确认，Windows暂缓。

当前openspec/changes/tm-s3-cancel-retain-data/，Accepted AD-TM-06按用户明确变更保留数据。之前pending-cancel核心PASS保留历史，撤销删除清理。

取消保留数据/实验格式安全兼容首批PASS，见[独立验证](../../../../openspec/changes/tm-s3-cancel-retain-data/verification-retain-data.md)。真实两任务/说明/四条事件与Outbox保留；完整执行中停止/接管尚未完成。

2026-09-14：新增[执行身份/停止联合设计](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-TM-08-EXECUTION-IDENTITY-AND-STOP.md)与三份设计验收补齐，AD保持Proposed；当前产品实现状态不变。下一项完成AG步骤/动作去重、Port/事务/迁移及可信WorkRef原生复核Spike，不因设计文档补齐宣称接管已实现。

2026-09-14工作身份隔离Goal PASS，见[独立验证](../../../../openspec/changes/e0-compare-cua-drivers/verification-work-identity-macos.md)。关闭/替换/重启旧引用拒绝，正常/最小化及新Observe有效；仅技术子范围，下一项AG步骤/动作去重与Rust Port/控制事务/迁移定稿，产品接管尚未启用。

2026-09-16：依据 Accepted AD-TM-08 实施步骤边界停止事务。首批只允许 observed attempt 原子暂停/取消/接管为 paused，并在提交后释放 Permit；unknown 保持占用。定位与 Recording 继续后置。

内部步骤边界停止子范围PASS：schema9停止事务、observed/unknown身份门禁及Permit释放顺序通过，正式库迁移保留全部数据；见[独立 Verification Goal](../../../../openspec/changes/tm-s3-step-boundary-stop/verification-goal.md)。下一增量为外部控制协议与任务卡片接线。

外部控制请求子范围 PASS：协议1.6、schema10、Yonder CLI MCP 与运行中任务卡片已接通；pending 仅显示“正在停止”，执行器确认后才释放占用。见[独立 Verification Goal](../../../../openspec/changes/tm-s3-control-request/verification-goal.md)。下一增量为接管后的精确工作定位。

2026-09-17开始接管定位增量：按Accepted AD-TM-08定位子范围实施协议1.13、SQLite13、宿主WorkRef保留及停止后精确前置；Recording、交回和Windows继续后置。

2026-09-17接管定位增量PASS：真实UDS Agent经trycua SDK执行步骤，Yonda任务卡片接管后提交`paused/stopped/focused`并精确前置原生任务窗口，Recording未启动；见[独立 Verification Goal](../../../../openspec/changes/tm-s3-takeover-work-focus/verification-goal.md)。完整Story仍保留跨Space/多显示器、Windows及RC-S1 Recording前置。

2026-09-20接管定位可见性增量PASS：macOS Adapter 在 AX 焦点确认后复核原窗口处于 WindowServer 当前可见列表，避免跨 Space 不可见窗口被误报为已定位；见[独立 Verification Goal](../../../../openspec/changes/tm-s3-work-focus-visibility/verification-goal.md)。真实跨 Space/多显示器仍需单独原生样本，Windows与 Recording 继续保留。
