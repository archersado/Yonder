# EN-S3 核心结构重构：协议来源、能力协商、端口拆分与宿主并发

Story: EN-S3  
Epic: EN  
Status: design-review  
OpenSpec: en-s3-protocol-single-source

## 设计文档

- [产品需求](product-requirements.md)
- [架构设计](architecture-design.md)
- [视觉交互设计](visual-interaction-design.md)

## 当前状态与前置条件

本 Story 是 2026-09-28 用户授权的架构审查后续：修正 TaskStore 上帝端口、协议 minor 阶梯的人肉同步税、宿主单锁长临界区、协议单一来源侵蚀、非穷举分发与前端门禁缺失。全部工作受"外部可观察行为不变"等价性约束（wire 字节、能力协商结果、SQLite schema/`user_version`、依赖方向）。

分五个独立可合阶段，顺序即风险顺序：

1. 协议单一来源（`valid_id`、`UnknownReason` 映射、版本常量下沉）——最小改动，先消除 CLI 硬编码风险。
2. Gateway 能力协商声明式能力表 + 穷举分发——以 golden 基准逐档（minor 0–31）锁定等价。
3. TaskStore 子 trait 拆分与 SQLite 实现分文件——42 项既有回归断言不得修改为护栏。
4. TaskHost 专用宿主线程 + 有界通道——AD-AG-09 连续推进的 UI 非阻塞前置；候选方案对比见架构设计。
5. 前端单一来源门禁 + `apps/desktop` 验证脚本迁移——纯工程面。

前置：无未决 ADR；阶段 4 实施前须确认当前 `dev` 分支 TM-S8 验证基线可复现（避免在验证中的产品链路上并发改宿主线程模型，届时与 TM-S8 验证串行）。

## 依赖与影响

- 下游受益：协议演进成本回落（阶段 1/2）、Windows 双平台与 BUA 多并发前的锁模型解锁（阶段 4）、CI 前端门禁补齐（阶段 5）。
- 不改产品范围、不新增协议、不动加密（AD-ST-01 延期不变）、不实现 Outbox 消费（属云端 Connector 范围）。
- Architecture Spine「React 桌宠」表述与 vanilla JS 现实的漂移随本 Story PR 顺带修正文档，不属验收范围。

## OpenSpec 与验证

[OpenSpec Change：en-s3-protocol-single-source](../../../../openspec/changes/en-s3-protocol-single-source/proposal.md)——阶段 1（协议单一来源）已按本 Change 实施并合入 dev（merge b4c3ba6），等价性证明与偏差记录（Codex 桥钉定 1.19）见其 design.md；独立 Verification Goal 待补。阶段 2–5 沿用本 Story 设计，待审阅转 ready 后按阶段另建或扩展 Change。Validation Goal 引用各阶段 AC 与对应证据类型：阶段 1–3/5 以自动化测试与门禁自测为主，阶段 4 须 macOS 原生并发回归日志；Windows 证据门禁保留、暂缓不视为通过。
