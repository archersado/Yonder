# TM-S5 产物清单读取与 Task Space 展示 Verification Goal

性质：独立验证记录；日期：2026-09-25。Story：TM-S5；Change：`tm-s5-artifact-list`；依据：Accepted AD-TM-22、TM5-AC05/06/08。

## 目标

- 协议 1.26 已认证会话可按固定清单版本、排他 ordinal 和 1..100 limit 读取条目；1.25 及以下稳定拒绝。
- 查询继续服从任务当前读取权限，跨 Agent 和不存在版本不泄露条目；协议只返回受控引用和四种可用性。
- Task Space 显示空清单、固定版本条目、分页、局部失败与重试；只有当前版本高于确认版本才提示变化。

## 结果

- 协议严格解析、Rust 派生 JSON Schema/TypeScript、Gateway 1.25/1.26 隔离、跨 Agent 拒绝、SQLite 固定版本分页与 CLI/MCP 工具回归通过。
- `/Users/archersado/.cargo/bin/cargo test --offline --locked --workspace`：110 项通过（Adapter 48、Application 29、CLI 5、Desktop 18、Domain 1、Protocol 9）。
- 发布契约同步为协议 1.26 / SQLite schema 19；协议生成物 `--check` 通过。
- 76 项 OpenSpec、架构关联检查、33 项 Python 测试、JavaScript 语法检查和 `git diff --check` 全部通过。
- macOS Ego Lite TaskSpace 123 的明确夹具验证空清单、四种可用性、20/21 分页、失败保留与固定版本重试、确认版本变化提示，并完成既有 Task Space 回归；同一空间已关闭。证据见 [`apps/desktop/evidence/tm-s5-artifact-list-macos-20260925`](../../../apps/desktop/evidence/tm-s5-artifact-list-macos-20260925/README.md)。
- 验证同时修正确认成功提示被刷新立即覆盖的竞态，以及既有 Task Space 夹具未实际执行时隐藏的任务计数与调用记录问题。

## 边界与结论

系统桌面验证时处于锁屏状态，因此没有发送系统输入，也不把 Ego Lite 证据冒充正式 Tauri 宿主原生操作。该增量不开放清单发布 wire、不解析路径或真实文件、不证明具体能力 Adapter 已发布产物。

macOS 可独立验证的协议/Gateway/Task Space 子范围 PASS。真实能力 Adapter、交回与 Recording 审计，以及按用户决定暂缓的 Windows 证据仍待完成；完整 TM-S5 保持 `verifying`，本 Change 不 Archive。
