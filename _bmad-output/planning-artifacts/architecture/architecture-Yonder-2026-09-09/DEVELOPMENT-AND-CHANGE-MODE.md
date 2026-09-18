# Yonder 研发与需求变更模式

## 主流程

输入基线：`_bmad-output/planning-artifacts` 中的产品简报及补充材料定义产品目标，ARCHITECTURE-SPINE 和适用的已接受 ADR 定义架构约束，后续用户明确变更单独记录。每个 Story 产品需求必须有「需求来源」章节，映射来源章节与验收条件；Proposed ADR、实现代码和工程假设不能充当已确认产品需求。发现冲突先记录和解决，不用技术模块划分遗漏原主干链路。

依据 AD-DEV-01，采用 SDD：`技术模块 Epic 目录 → Story 三份设计 → OpenSpec Proposal → 实现 → 独立 Verification Goal → Archive`。规划唯一入口为 `docs/specs/README.md`。月份只作为里程碑，不作为 Epic。

Epic 目录 `docs/specs/epic-<模块>/` 下，每个 `story-<ID>/` 必须含 README.md（状态、依赖、Change 关联）、product-requirements.md（问题、范围、验收）、architecture-design.md（边界、状态、接口、依赖、失败和验证）、visual-interaction-design.md（入口、状态反馈、键盘/无障碍、平台证据）。无 UI 仍需写调用交互与错误表现，不能省略。

Story 按 draft → design-review → ready → implementing → verifying → done 推进，可明确 deferred。ready 前完成设计审阅和前置 ADR/Spike 门禁；工程审阅不要求用户逐步确认，但不得把未决产品问题视为已解决。先有 Story 设计，再生成 proposal/design/tasks/specs；实现后独立验证，失败回实施，通过才能归档。历史迁移文档不追认完成。

每个 Story 默认对应一个短生命周期分支 `story/<id>-<slug>`、一个 OpenSpec Change 和一个 PR。主干必须始终可构建。

## 变更分级

1. 行为变化：先有 Epic/Story，再走 OpenSpec 的 Explore、Propose、Review、Apply、Verify、Archive。
2. 恢复既有规格的缺陷：引用现有规格并提交失败测试，无须新 Proposal。
3. 改变依赖方向、状态所有者、协议、持久化、技术栈或产品边界：先更新 Architecture Decision，再建 Story/OpenSpec。

OpenSpec Change 必须引用 Story 与 AD，包含可测试 delta spec 和 Architecture Impact（none/conforming/architecture-change）、模块、依赖、协议及迁移影响。规格、代码和测试进入同一 PR。

## 完成定义

实现后必须创建独立 Verification Goal，引用 Story AC、OpenSpec、AD、环境和证据。验证者默认不得修改实现；失败返回 Apply。Goal 达成后才能 Archive 和 Story Done。

测试分层：Domain 单元测试；协议/Adapter 合约测试；SQLite/Outbox/Sidecar/Gateway 集成测试；Windows/macOS 原生 E2E。UI、Driver 或权限变更必须分别提交双平台截图、视频或结构化日志。CUA Driver 升级运行固定 Office/WPS/文件管理器任务集。

## 架构门禁

CI 检查 Cargo 依赖图、前端 import、协议单一来源、Story/Change/PR 关联和证据。Verification Goal 人工确认：无 Adapter 越层、无第二状态所有者、无隐式协议或持久化变化、无云端职责侵入。

## 架构 Spike

Spike 必须有期限、统一样本、淘汰门槛和 ADR 输出，不得以临时双实现进入长期生产。首批 Spike 是 CUA Driver 对照验证，以及 Rust/Node OOXML Adapter 对照验证。
