# Verification Goal：TM-S3 CUA 规划与执行步骤展示

## 目标

在 macOS 正式 Yonder GUI 中，通过真实 Local Socket、真实 trycua CUA 计划片段验证顶部控制条：显示规划步骤和当前执行步骤、保持圈选工具条同屏顶部定位，并能由显式按钮在 Observe 边界接管。

## 通过条件

- UI 回归覆盖计划步骤、执行中→已完成推进、显式接管和失败态。
- Desktop 单元测试覆盖 Hub 的真实 step 投影且不含动作参数。
- 正式打包 GUI 的原生 AX 验证确认控制条可见、560×174、顶部居中，含“规划步骤”和首个计划标签；点击接管后任务在未执行完全部槽位前进入 `paused/stopped`。
- 普通输入零新增 `unknown/user-input`；Windows 仍按用户决定暂缓。

## 证据位置

2026-09-28 macOS PASS：

- `node apps/desktop/check-cua-control.mjs`：规划步骤、执行中→已完成推进、显式接管与失败态通过。
- `cargo test -p yonder-desktop --lib --locked`：13 项通过，包含 Hub 真实派发步骤投影与参数脱敏。
- 正式打包 `Yonda.app` 的 `python3 apps/desktop/check-cua-control-macos.py apps/desktop/evidence/tm-s3-cua-step-presentation-macos-20260928` 通过：控制条 560×174、顶部居中、规划标题和首个步骤可见，点击接管后 `paused/stopped`、未跑完全部槽位、零新增 `unknown/user-input`。

证据位于 `apps/desktop/evidence/tm-s3-cua-step-presentation-macos-20260928/`。Windows 按用户决定暂缓。
