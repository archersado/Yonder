# CU-S1 桌面 Driver 技术选型

Story: CU-S1
Epic: CU
Status: verifying
OpenSpec: e0-compare-cua-drivers

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

2026-09-18：唯一选型trycua 0.25.0已同步到正式`apps/desktop/cua`依赖清单与`crates/adapters`运行链路，旧`e0-compare-cua-drivers`混合Change冻结。CU-S1不再新增实现；正式宿主权限、停止、Observe和Windows后续证据由CU-S2/TM-S3承接。

SDK原生输入首批STOP-IN01–03已在macOS通过：当前测试宿主权限只读检查、唯一隔离AX输入/动作后Observe、动作间shutdown后拒绝新输入与750ms目标状态稳定。见[原生验证](../../../../openspec/changes/e0-compare-cua-drivers/verification-input-macos.md)。不是执行中动作中断或正式Yonder权限通过，完整Story仍未完成。

当前SDK-only边界见Accepted [AD-CU-01](../../../../_bmad-output/planning-artifacts/architecture/architecture-Yonder-2026-09-09/AD-CU-01-SDK-ONLY-INTEGRATION.md)。不附带上游App或可执行文件；独立构件分支撤回，历史拒绝证据保留。自管SDK子进程就绪/监督终止/宿主通道断开退出/新实例恢复STOP-SDK01–04在macOS通过，见[独立验证](../../../../openspec/changes/e0-compare-cua-drivers/verification-sdk-worker-macos.md)。后续为SDK隔离输入/停止/Observe与正式宿主权限验证，不继续下载或核验上游App。

Windows 选型已有证据；macOS 和 Office 延期，不改称全部完成。

2026-09-14补充：固定trycua0.25.0在原生macOS完成只读取消、关闭/重复关闭、关闭后拒绝调用和异常退出后新实例恢复，STOP-READ01–04独立验证通过。见[验证记录](../../../../openspec/changes/e0-compare-cua-drivers/verification-stop-macos.md)。执行中键鼠停止、原生Worker退出证明、Finder/Office动作后Observe仍未验证，不能据此启用接管或Recording。

## OpenSpec 与验证

openspec/changes/e0-compare-cua-drivers/

[Change](../../../../openspec/changes/e0-compare-cua-drivers/proposal.md)；独立验证在该 Change 内维护，记录存在不代表通过。

[原 Story 正文与历史验证](legacy-record.md)。旧编号仅作追溯，不用于新 PR。
