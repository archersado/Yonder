# TM-S5 产物清单核心版本化 Verification Goal

性质：独立验证记录；日期：2026-09-25。Story：TM-S5；Change：`tm-s5-artifact-manifest-core`；依据：Accepted AD-TM-22、TM5-AC05/06/07/08。

## 目标

- 可信 Application 能力用例以完整集合发布下一不可变清单版本，任务序号、事件、Outbox、清单和全部条目同事务提交。
- 固定版本条目按稳定顺序有界分页，并服从当前任务读取权限；重复引用、非法版本、超限输入及配额不足明确拒绝。
- 用户确认绑定当时最新清单；后续包含 `changed`、`missing` 或新增引用的版本不改写旧清单与旧确认。

## 结果

- Adapter 集成测试覆盖 created 状态拒绝、非空版本 1、分页、跨 Agent 拒绝、不存在版本拒绝、确认绑定版本 1、终态发布版本 2、旧版本不变，以及条目插入失败时任务/事件/Outbox/清单/条目整体回滚。
- 配额测试使用真实临时 SQLite 文件验证容量门禁；另覆盖重复引用及 4097 条超限输入。既有空清单确认回归继续通过。
- `/Users/archersado/.cargo/bin/cargo test --offline --locked --workspace`：109 项通过（Adapter 48、Application 29、CLI 5、Desktop 18、Domain 1、Protocol 8）。
- Rust 派生协议检查、Task Space 静态检查、75 项 OpenSpec、架构检查、33 项 Python 测试和 `git diff --check` 全部通过。

## 边界与结论

本增量不新增 Gateway/协议/UI，不读取文件正文，也不改变 schema 18；因此没有新增原生 UI/Driver 行为需要取证。macOS 可独立验证的核心存储/Application 子范围 PASS。

完整 TM-S5 仍缺产物清单的 Gateway/具体能力 Adapter/UI 接线、跨 Story 的交回与 Recording 审计，以及按用户决定暂缓的 Windows 验证，因此 Story 保持 `verifying`，本 Change 不 Archive。
