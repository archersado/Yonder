# TM-S2 Windows BUA 执行尝试验证

日期：2026-11-01。结论：**PASS（真实 ego-lite BUA Driver 子范围）**。

Windows Named Pipe 正式任务 `task_f956b3736988abefef2724bfd21b7b9b` 完成 `create → 6轮observe → finish`。只读核对正式 SQLite `task_attempts` 得到 8 条 attempt：`create-space@3`、`observe-1@7` 至 `observe-6@27`、`finish-space@31`；它们均绑定 `worker_instance_id=ego_bridge` 和同一 `host_session_id=host_57760`，最终 `phase=stopped`、`action_succeeded=1`、`observe_valid=1`、`unknown_reason=NULL`，每条都有独立 result sequence。

任务最终 `completed@34`，事件 34 条，外部 ego-lite Task Space `finished=true`。机器证据见 `apps/desktop/evidence/ds-s2-windows-20261101/native-task-space-result.json`，汇总见 `../ds-s2-running-pagination/verification-windows-20261101.md`。

本报告只关闭 Windows BUA Driver 的 prepared/派发/Observe/Stopped 对等链路。Windows CUA 对等验证仍按用户要求排除，不可用 BUA 代替。
