# Proposal：TM-S3 CUA 规划与执行步骤展示

## Why

用户要求在 Yonder 控制电脑的过程中，直接看到本轮规划步骤以及正在执行的步骤。现有顶部控制条仅显示一个当前步骤摘要，无法让用户判断连续 CUA 是否正在按受限计划推进。

## What Changes

- 控制条展示已验证计划片段的有界步骤列表及当前真实派发步骤。
- 已完成、执行中、待执行状态可辨；单步任务和不可用读取均有诚实降级文案。
- 展示由宿主从 SQLite 计划片段和 prepared attempt 派生的只读内存投影提供，不新增协议、持久化字段或 UI 状态所有权。
- 不显示动作参数、输入正文、截图、AX 树或完整 Agent Payload。

## Impact

影响 `apps/desktop` 的 CUA 控制条投影与 UI，引用 Accepted AD-CU-07；不改变 Application 执行、Jev 选择、Adapter Driver、Gateway 协议或 SQLite。macOS 验证本增量；Windows 仍按既有决定暂缓。
