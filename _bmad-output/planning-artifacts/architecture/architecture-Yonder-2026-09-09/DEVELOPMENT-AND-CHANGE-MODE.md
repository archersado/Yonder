# Yonder 研发与需求变更模式

## 主流程

小型全栈团队采用 SDD：`Epic → Story → OpenSpec Change → 实现 → Verification Goal → Archive`。BMAD 管理 Epic/Story，OpenSpec 只表达 Story 内的增量规格，避免重复事实源。

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
