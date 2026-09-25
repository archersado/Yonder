# TM-S5 执行尝试开始历史投影 Verification Goal

性质：独立验证记录；日期：2026-09-25。Story：TM-S5；Change：`tm-s5-historical-attempt-start`；依据：Accepted AD-TM-21、TM5-AC01/03/07/08。

## 目标

- 只把 `task_attempts.accepted_sequence` 对应的原 Start 事件投影为 `attempt_started`，不从当前 phase、running 状态或最终结果补造。
- 协议 1.24 隐藏新增字段，1.25 返回完整不可变 step/attempt/worker/host 身份；非归属 Agent 不可读取。
- Task Space 只显示“执行尝试已准备”及 step/attempt，不显示 Worker/Host，也不暗示派发、成功、Observe 或安全停止。

## 结果

- Adapter/Application/Gateway 回归覆盖开始事实与最终结果分属 `#3/#4`、1.24/1.25 隔离、非归属 Agent `-32004`、旧 running 任务零回填及损坏身份明确失败。全仓 104 项 Rust 测试通过；Rust 派生 TypeScript 与 JSON Schema 已更新。
- Task Space 前端回归验证准备文案，并确认 `worker_instance_id/host_session_id` 不进入界面；macOS WebKit 隔离证据位于 `apps/desktop/evidence/tm-s5-historical-attempt-start-webkit-20260925/`。
- 独立 bundle identifier `com.yonder.attempt-start.fixture` 的正式 Tauri 宿主从专用 SQLite 读取真实步骤、尝试与结果事务。私有 Unix Socket 下，1.24 返回结果但不返回开始事实；1.25 精确返回 `#3 / step-one / attempt-one / worker-one / host-one`，最终结果仍位于 `#4`。证据位于 `apps/desktop/evidence/tm-s5-historical-attempt-start-native-host-20260925/`。
- 正式原生 Task Space 分别显示“执行尝试已准备（步骤 step-one · 尝试 attempt-one）”与“动作已观察：成功”，截图位于 `apps/desktop/evidence/tm-s5-historical-attempt-start-native-ui-20260925/`；没有使用正式用户任务库。
- 本增量无 schema 迁移；协议发布契约同步到 1.25。74 项 OpenSpec 校验、19 项架构检查、Task Space 回归和 `git diff --check` 通过；隔离宿主已关闭，夹具数据已移出 Application Support。

## 结论

macOS 子范围 PASS。Windows 原生证据按用户决定暂缓，因此本 Change 保持 verification-pending，不 Archive；完整 TM-S5 的保留、Recording 与交回仍未完成。
