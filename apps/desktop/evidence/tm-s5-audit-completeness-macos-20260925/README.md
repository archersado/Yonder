# TM-S5 macOS 原生审计验证证据

日期：2026-09-25
系统：macOS 26.5.1（25F80）
结论：PASS

## 范围与方法

- 使用正式打包的 `Yonda.app`、隔离 HOME 和独立 SQLite 数据库，不读取或展示用户任务数据。
- 仅用测试夹具准备登记 Agent 与序号 4 的已完成任务；“确认结果”通过 Task Space 的正式本机用户命令进入 Application/SQLite 事务。
- 原生 AX 验证器检查终态、待确认提示、空清单说明、时间线、确认后投影与清单版本，并以窗口级截图留证。
- Task Space 的异步刷新会改变滚动位置；`audit-pending.png` 与 `audit-confirmed.png` 选自同一隔离夹具的两次重复 PASS，以确保截图文字可独立复核。`native-result.json` 与 `database-result.json` 分别记录完整原生断言和最终重复运行的数据库结果。

## 文件

- `audit-pending.png`：显示 `已完成`、`结果待确认` 与“确认后生成首个清单”。
- `audit-confirmed.png`：显示“已确认 · 结果序号 4 · 清单版本 1”及版本 1 空清单。
- `native-result.json`：原生流程结构化结果。
- `database-result.json`：最终状态、确认、清单、事件与 Outbox 核对。

## 发现与修复

初次验证发现确认事务和时间线已成功，但详情仍显示“结果待确认”。根因是 `task.step.get` 使用不含审计数据的投影。修复后协议 1.20 返回确认与清单，协议 1.19 继续隐藏这些字段；重启应用后确认投影仍可恢复。
