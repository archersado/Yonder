# 独立 Verification Goal：TM-S5 完整审计闭环

日期：2026-09-25
关联 Story：TM-S5  
关联 Change：`tm-s5-audit-completeness`  
结论：协议、存储、应用、前端回归与 macOS 原生 UI 验证通过；Windows 验证按当前决定暂缓，Story 保持 verifying，不 Archive。

## 当前验证范围

- 协议 1.20 新增 `user_confirmation` 与 `artifact_manifest`，并验证旧客户端不读取新投影。
- SQLite schema 18 新增确认、清单与配额表，迁移保持失败回滚。
- 应用层验证本机用户确认边界、幂等、事务回滚与配额拒绝。
- 前端验证终态任务确认 UI、确认后投影与失败反馈。
- macOS 原生验证使用隔离 HOME、正式打包应用与真实 SQLite/Application 确认入口；测试夹具仅用于准备已完成任务与 Agent 登记。

## 已通过检查

- `/Users/archersado/.cargo/bin/cargo test --offline --locked --workspace`：88 项通过。
- `node apps/desktop/check-task-space.mjs`：通过。
- `openspec validate tm-s5-audit-completeness --strict` 与 `openspec validate --all --strict`：通过。
- `env PATH=/usr/bin:/bin /usr/bin/python3 -m unittest discover -s scripts -p 'test_*.py'`：22 项通过。
- `env PATH=/usr/bin:/bin /usr/bin/python3 scripts/check_architecture.py`：通过。
- `git diff --check`：通过。

## macOS 原生证据

- 系统：macOS 26.5.1（25F80）；隔离资料目录，不读取或展示用户任务数据。
- 原生 Task Space 清楚展示终态 `已完成`、`结果待确认` 与“确认后生成首个清单”；用户通过原生确认入口提交后，任务仍保持终态。
- 确认后 SQLite 当前状态序号由 4 增至 5；确认事件、空清单版本 1、事件与 Outbox 在同一提交后均可读取。
- 验证中发现并修复 `task.step.get` 丢弃审计投影的问题；补充协议 1.20/1.19 新旧能力回归后，应用重启仍能显示“已确认 · 结果序号 4 · 清单版本 1”。
- 截图、结构化结果和数据库核对见 [`apps/desktop/evidence/tm-s5-audit-completeness-macos-20260925`](../../../apps/desktop/evidence/tm-s5-audit-completeness-macos-20260925/README.md)。因 Task Space 窗口滚动位置会在异步刷新后变化，两张截图选自相同隔离夹具的两次重复 PASS；结构化断言与最终数据库核对来自完整重复运行。

## 待完成

- Windows 与完整 TM-S5 验证仍按既有决定暂缓，不能据此 Archive 或 Done。
