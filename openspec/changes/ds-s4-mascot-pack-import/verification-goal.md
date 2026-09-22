# DS-S4 资源包导入子范围独立 Verification Goal

日期：2026-09-21。关联 DS-S4 PACK-01～05 的当前实现子范围与 `openspec/changes/ds-s4-mascot-pack-import/`。

## 结果

PASS（离线导入子范围与 macOS 原生拒绝证据均完成）。`pet_pack` 校验 v1 manifest、九个固定状态、每状态帧集、PNG/WebP 完整解码、路径安全、重复/未知文件、源大小、解压总量、单帧大小、单帧尺寸和总像素；通过后写入应用数据目录受限暂存并执行带回滚的目录切换。托盘提供本机 ZIP 选择，桌宠通过已校验的字节资产重载，减少原内置局部动作遗留，并显示无障碍导入状态。2026-09-22 已补齐真实托盘触发、原生打开面板、无效包拒绝反馈和旧包保留证据。

工作区离线回归 PASS：`cargo test -p yonder-adapters -p yonder-desktop` 共 36 项通过，其中新增测试覆盖完整 PNG/WebP、缺失状态保留旧包、路径穿越与损坏帧。`node --check apps/desktop/ui/pet.js`、`python3 scripts/check_architecture.py` 和 `git diff --check` 通过；OpenSpec planning artifacts 4/4 complete。

## 完成边界

本 Goal 不宣称 Windows 平台或完整 Story 通过；Hatch Pet 生成委托按用户决定仅保留设计。macOS 原生拒绝证据见 `apps/desktop/evidence/ds-s4-mascot-pack-import-macos-20260922/`。DS-S4 保持 `verifying`，不 Archive。
