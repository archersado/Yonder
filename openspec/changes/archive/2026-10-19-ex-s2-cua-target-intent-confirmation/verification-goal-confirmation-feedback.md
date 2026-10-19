# 独立 Verification Goal：发送确认触点与失效反馈

日期：2026-10-07。关联 EX-S2 TARGET-ACTION-04、视觉交互设计“拒绝、过期或任务状态变化时显示最小原因”、本 Change。架构影响 conforming：仅修复顶部 UI 投影，不新增任务状态或协议。

## 失败证据

- 正式 macOS GUI 任务 `task_2b04936bc487ce52b1cc0757c5957d80` 的首条确认已到期；原生 AX 仍显示“等待用户在顶部浮窗确认发送”，只有接管按钮，没有发送/拒绝入口。
- 原 UI 合约 fixture 未包含发送确认 DOM，直接因缺失按钮发生异常；补齐 fixture 后新增过期用例在原实现失败。

## 验证与边界

- 正式 Gateway 在同一任务提交 v2 片段并执行：前三槽位成功，到发送槽位返回 `awaiting-confirmation`，sequence 38；没有派发发送。
- macOS 原生 AX 实测出现“取消发送”“确认发送”两个 button，证明有效确认具有本地可交互触点。检查未点击任何确认。
- 修复版 UI 合约回放：有效预览、过期清除/禁用、不可用反馈、批准/拒绝反馈保持、任务隔离；全部通过。
- `node --check apps/desktop/ui/cua-control.js`、`git diff --check` 通过。
- 修复版未重启正式 GUI，避免使当前待批准引用失效；真实发送和修复版原生过期提示仍待后续验证。不得据此标记 Story Done 或 Archive。Windows 按既有安排暂缓。

结论：确认触点已在当前正式 GUI 恢复；失效反馈代码与合约验证通过，原生新版反馈验证待办。发送任务仍等待用户本人批准。
