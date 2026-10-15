# Proposal：EN-S3 阶段 3 TaskStore 边界拆分

## Why

Application 的 `TaskStore` 同时承担核心生命周期、执行/控制、审计和计划片段职责，SQLite 唯一实现集中在超过一万行的单文件中。Gateway 握手还逐项调用多个 `supports_*`，能力事实没有单次快照，增加实现遗漏和后续演进风险。

## What Changes

- 新增一次性 `StoreCapabilities` 探测，Gateway 握手只读取该快照。
- 将 `TaskStore` 拆为核心、执行、审计、计划片段四个子 trait，并以组合 supertrait 保持调用方 `impl TaskStore` 约束。
- 将 SQLite 实现按同一职责边界迁入 `task_store/` 子模块；不改 SQL、测试断言、schema、迁移或事务。
- 运行既有 Adapter/Application/Workspace 回归与 `PRAGMA user_version` 护栏。

关联 Story：EN-S3。Architecture Impact：none；这是不改变外部行为的 conforming 重构，无新增 ADR。

## Non-Goals

不实施宿主并发模型或前端门禁；不新增能力、协议、迁移、依赖或 UI；Windows 原生验证继续暂缓。
