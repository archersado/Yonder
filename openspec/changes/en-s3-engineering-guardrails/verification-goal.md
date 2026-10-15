# 独立 Verification Goal：EN-S3 阶段 5 工程门禁

日期：2026-10-15  
平台：macOS arm64  
结论：PASS

## 验证范围

- Story：`docs/specs/epic-EN/story-EN-S3/` 阶段 5。
- Change：`en-s3-engineering-guardrails`。
- 架构影响：none；不得改变 wire、SQLite schema/`user_version`、依赖方向、UI 或执行行为。

## 证据

1. 前端协议展示映射已由架构门禁约束到 `crates/protocol/generated/protocol.ts`；受控类型包括 `TaskStatus`、`TaskSource`、`TaskObservationResult`、`AttemptUnknownReason`、`FocusFailure`、`ArtifactAvailability` 与 `FileGrantPurpose`。缺失来源、错误来源与重复枚举拒绝样本均通过。
2. 72 个 Desktop `check-*` 验证脚本已从 `apps/desktop/` 根目录迁至 `scripts/verification/desktop/`；旧目录残留数为 0，新目录脚本数为 72。正式架构门禁会拒绝脚本回流，并要求统一目录持续存在。
3. 迁移后的 Python 脚本全部通过内建 `compile` 语法检查，Node `.mjs` 全部通过 `node --check`，Swift 脚本全部通过 `swiftc -frontend -parse`。
4. 9 个 Node 验证脚本全部实际执行 PASS：Agent 会话定向、动画预览、CUA 控制、圈选预览拖拽、圈选提交恢复、安全 Markdown、任务空间、语音回复与语音会话。
5. 4 个代表性 Python 隔离验证全部 PASS：Sky CUA worker、Sky 消息 worker、CUA 通用计划动作与 CUA 目标动作。
6. `cargo test --workspace --locked -- --test-threads=1` PASS：Adapters 87、Application 92、CLI 21、Desktop lib 39、Desktop bin 12、Domain 1、Protocol 21，共 273 项，全部通过；Doc tests 全部通过。
7. `cargo run -p yonder-protocol --example generate --locked -- --check` PASS，协议生成物保持最新。
8. `python3 scripts/test_check_architecture.py` 27/27 PASS；`python3 scripts/check_architecture.py` PASS。
9. `openspec validate en-s3-engineering-guardrails --strict` PASS。

## 复核结论

阶段 5 的前端协议单一来源门禁和 Desktop 验证资产治理均满足设计目标；迁移只改变验证脚本位置和仓库约束，不改变产品运行时行为。该 Change 可以 Archive。

Windows 原生证据按既有决定暂缓；本阶段没有平台 UI、Driver 或权限变化，不以暂缓项宣称完整 EN-S3 Done。EN-S3 阶段 4 仍受 TM-S8 验证基线门禁约束，归档后回到 `design-review`。
