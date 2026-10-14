# 独立 Verification Goal：EN-S3 阶段 2 Gateway 能力协商与分发

日期：2026-10-14  
平台：macOS arm64  
结论：PASS

## 验证范围

- Story：`docs/specs/epic-EN/story-EN-S3/` AC2、AC3、AC8。
- Change：`en-s3-capability-routing`。
- 架构影响：none；不得改变 wire、SQLite schema/user_version、依赖方向或 UI。

## 证据

1. `capability_negotiation_minor_0_through_current_matches_golden` 对 minor 0–50 逐档锁定 hello 顶层版本与 `task.read` 版本，并锁定当前完整能力列表顺序。协议常量 1.50、Gateway 既有最高投影 1.49 的差异按等价性要求保留。
2. `request_capability_gate` 对 `Request` 全部变体使用穷举 `match`；新增变体未分类将导致编译失败。既有能力拒绝码和中文文案原样迁移。
3. `cargo test --workspace --locked` PASS：Adapters 87、Application 85、CLI 20、Desktop lib 37、Desktop bin 11、Domain 1、Protocol 20，共 261 项，全部通过；Doc tests 全部通过。
4. `cargo run -p yonder-protocol --example generate --locked -- --check` PASS，协议生成物无变化。
5. `python3 scripts/check_architecture.py` PASS；未修改 Cargo 依赖、SQLite schema/迁移、协议类型或 UI。
6. `openspec validate en-s3-capability-routing --strict` PASS。

## 复核结论

能力版本选择已从两条嵌套条件链收敛为有序 feature 表，公开能力列表由单一有序声明生成，重复的 `can_audit` 赋值已删除。现有产品行为由 0–50 golden 与全仓回归共同约束，满足阶段 2 的等价重构目标，可以 Archive。

Windows 原生证据按既有决定暂缓；本阶段没有平台 UI、Driver 或权限变化，不以暂缓项宣称完整 EN-S3 Done。
