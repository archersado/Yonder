# TM-S5 任务创建来源历史投影 Verification Goal

性质：独立验证记录；日期：2026-09-24。Story：TM-S5；Change：`tm-s5-historical-creation`；依据：Accepted AD-TM-20、TM5-AC01/06/07/08。

## 目标

- 只把创建事务已提交的 source payload 与不可变任务归属按原 `#1` 序号投影，不从当前快照补造缺失历史。
- 协议 1.23 隐藏新增字段，1.24 才返回；非归属 Agent 不能读取，损坏 payload 明确失败。
- Task Space 显示来源类别与 Agent 标识，不暴露描述、幂等键、凭据或完整 Agent Payload，也不暗示当前授权。

## 结果

- Adapter/Application/Gateway 回归覆盖 cloud-agent 历史、1.23/1.24 隔离、非归属 Agent `-32004`、损坏 payload 拒绝，以及删除 source 历史后仍不从当前 CloudAgent 快照回填。全仓 93 项 Rust 测试通过；Rust 生成 TypeScript 与 JSON Schema 为最新版本。
- macOS WebKit 隔离夹具显示“任务创建：云端 Agent · Agent fixture-agent”，证据在 `apps/desktop/evidence/tm-s5-historical-creation-webkit-20260924/`；不连接真实任务库。
- 独立 bundle identifier `com.yonder.creation.fixture` 的正式 Tauri 宿主从专用 SQLite 读取真实创建事务。私有 Unix Socket：1.23 返回同一条事件但零条创建来源事实；1.24 精确返回 `#1 / fixture-agent / cloud-agent`，响应没有描述或幂等键。原生 Task Space 可见并截图。Gateway 与原生 UI 证据分别位于 `apps/desktop/evidence/tm-s5-historical-creation-native-host-20260924/` 和 `apps/desktop/evidence/tm-s5-historical-creation-native-ui-20260924/`。
- 本增量无 schema 迁移。正式用户库保持 schema 19、`PRAGMA integrity_check=ok`，原有 441 条 source 历史未改写；未向用户库写入夹具任务。普通预览宿主已恢复运行。
- 71 项 OpenSpec、架构关联检查、Task Space 前端回归和 `git diff --check` 通过；隔离进程已关闭。

## 结论

macOS 子范围 PASS。Windows 原生证据按用户决定暂缓，因此本 Change 保持 verification-pending，不 Archive；完整 TM-S5 的产物、保留、Recording 和交回仍未完成。
